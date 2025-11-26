// ! Unified SQLx backend implementation for AgentDB
//!
//! Provides a single backend type that works with SQLite, PostgreSQL, and MySQL
//! using runtime dispatch based on the connection URL.

use crate::{error::Result, SqlError};
use agentdb::{
    AgentDB, BackendFamily, Capabilities, DefaultCapabilities, QueryResult, Row, ScanResult,
    Transaction, Value,
};
use async_trait::async_trait;
use sqlx::{
    any::{install_default_drivers, AnyRow},
    AnyPool, Column, Row as SqlxRow,
};

/// Configuration for SQL backend
#[derive(Debug, Clone)]
pub enum SqlBackendConfig {
    /// SQLite: file path or ":memory:"
    #[cfg(feature = "sqlite")]
    Sqlite(String),

    /// PostgreSQL: connection URL
    #[cfg(feature = "postgres")]
    Postgres(String),

    /// MySQL: connection URL
    #[cfg(feature = "mysql")]
    Mysql(String),
}

/// Unified SQL backend using SQLx
///
/// Supports SQLite, PostgreSQL, and MySQL with a single interface.
pub struct SqlBackend {
    pool: AnyPool,
    backend_type: BackendType,
    capabilities: DefaultCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendType {
    #[cfg(feature = "sqlite")]
    Sqlite,
    #[cfg(feature = "postgres")]
    Postgres,
    #[cfg(feature = "mysql")]
    Mysql,
}

impl SqlBackend {
    /// Create a new SQL backend
    ///
    /// # Arguments
    ///
    /// * `config` - Backend configuration (SQLite path, Postgres URL, or MySQL URL)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use agentsql::{SqlBackend, SqlBackendConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // SQLite
    ///     let db = SqlBackend::sqlite("agent.db").await?;
    ///
    ///     // PostgreSQL (requires "postgres" feature)
    ///     let db = SqlBackend::postgres("postgres://user:pass@localhost/db").await?;
    ///
    ///     // MySQL (requires "mysql" feature)
    ///     let db = SqlBackend::mysql("mysql://user:pass@localhost/db").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(config: SqlBackendConfig) -> Result<Self> {
        // Install default SQLx drivers
        install_default_drivers();

        let (url, backend_type, capabilities, is_memory) = match config {
            #[cfg(feature = "sqlite")]
            SqlBackendConfig::Sqlite(path) => {
                let is_memory = path == ":memory:";
                let url = if is_memory {
                    // For :memory: databases, use shared cache mode to ensure all connections
                    // see the same database, and set pool size to 1
                    "sqlite::memory:?cache=shared".to_string()
                } else {
                    format!("sqlite:{}?mode=rwc", path)
                };
                (
                    url,
                    BackendType::Sqlite,
                    DefaultCapabilities {
                        transactions: true,
                        directories: true,
                        graph_queries: false,
                        sql_queries: true,
                        indexes: true,
                        ttl: false,
                        max_key_size: Some(1024 * 1024),       // 1MB
                        max_value_size: Some(1024 * 1024 * 1024), // 1GB
                    },
                    is_memory,
                )
            }
            #[cfg(feature = "postgres")]
            SqlBackendConfig::Postgres(url) => (
                url,
                BackendType::Postgres,
                DefaultCapabilities {
                    transactions: true,
                    directories: true,
                    graph_queries: false,
                    sql_queries: true,
                    indexes: true,
                    ttl: false,
                    max_key_size: None,    // unlimited
                    max_value_size: None,  // unlimited
                },
                false,
            ),
            #[cfg(feature = "mysql")]
            SqlBackendConfig::Mysql(url) => (
                url,
                BackendType::Mysql,
                DefaultCapabilities {
                    transactions: true,
                    directories: true,
                    graph_queries: false,
                    sql_queries: true,
                    indexes: true,
                    ttl: false,
                    max_key_size: Some(255),  // VARCHAR(255) for keys
                    max_value_size: None,     // LONGBLOB
                },
                false,
            ),
        };

        // For :memory: databases, use a single connection pool
        let pool = if is_memory {
            use sqlx::pool::PoolOptions;
            PoolOptions::new()
                .max_connections(1)
                .connect(&url)
                .await
                .map_err(|e| SqlError::Connection(e.to_string()))?
        } else {
            AnyPool::connect(&url)
                .await
                .map_err(|e| SqlError::Connection(e.to_string()))?
        };

        let backend = Self {
            pool,
            backend_type,
            capabilities,
        };

        // Run migrations
        backend.migrate().await?;

        Ok(backend)
    }

    /// Convenience constructor for SQLite
    #[cfg(feature = "sqlite")]
    pub async fn sqlite(path: impl Into<String>) -> Result<Self> {
        Self::new(SqlBackendConfig::Sqlite(path.into())).await
    }

    /// Convenience constructor for PostgreSQL
    #[cfg(feature = "postgres")]
    pub async fn postgres(url: impl Into<String>) -> Result<Self> {
        Self::new(SqlBackendConfig::Postgres(url.into())).await
    }

    /// Convenience constructor for MySQL
    #[cfg(feature = "mysql")]
    pub async fn mysql(url: impl Into<String>) -> Result<Self> {
        Self::new(SqlBackendConfig::Mysql(url.into())).await
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
        let sql = match self.backend_type {
            #[cfg(feature = "sqlite")]
            BackendType::Sqlite => include_str!("../migrations/sqlite.sql"),
            #[cfg(feature = "postgres")]
            BackendType::Postgres => include_str!("../migrations/postgres.sql"),
            #[cfg(feature = "mysql")]
            BackendType::Mysql => include_str!("../migrations/mysql.sql"),
        };

        // Get a connection from the pool
        // Note: We execute DDL statements directly without a transaction
        // because SQLite auto-commits DDL and wrapping in transaction can cause issues
        let mut conn = self.pool.acquire().await
            .map_err(|e| SqlError::Migration(format!("Failed to acquire connection: {}", e)))?;

        // Split SQL into individual statements and execute them one by one
        // SQLx doesn't support executing multiple statements at once with raw_sql
        for (idx, statement) in sql.split(';').enumerate() {
            // Remove comment lines and trim
            let statement: String = statement
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with("--")
                })
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            // Skip empty statements
            if statement.is_empty() {
                continue;
            }

            sqlx::query(&statement)
                .execute(&mut *conn)
                .await
                .map_err(|e| SqlError::Migration(format!("Failed to execute migration statement #{}: {} - Error: {}", idx, statement, e)))?;
        }

        Ok(())
    }

    /// Convert SQLx row to AgentDB row
    fn convert_row(&self, row: AnyRow) -> Result<Row> {
        let mut agent_row = Row::new();

        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name().to_string();

            // Try to get the value as raw and convert based on what works
            // Start with Option types to handle NULL
            let value =
                // Try Option<i64> to handle NULL integers
                row.try_get::<Option<i64>, _>(i).ok().and_then(|opt| opt.map(|v| v.to_string().into_bytes()))
                // Try Option<i32>
                .or_else(|| row.try_get::<Option<i32>, _>(i).ok().and_then(|opt| opt.map(|v| v.to_string().into_bytes())))
                // Try Option<String>
                .or_else(|| row.try_get::<Option<String>, _>(i).ok().and_then(|opt| opt.map(|v| v.into_bytes())))
                // Try Option<Vec<u8>>
                .or_else(|| row.try_get::<Option<Vec<u8>>, _>(i).ok().and_then(|opt| opt))
                // Try Option<f64>
                .or_else(|| row.try_get::<Option<f64>, _>(i).ok().and_then(|opt| opt.map(|v| v.to_string().into_bytes())))
                // Try Option<f32>
                .or_else(|| row.try_get::<Option<f32>, _>(i).ok().and_then(|opt| opt.map(|v| v.to_string().into_bytes())))
                // Try Option<bool>
                .or_else(|| row.try_get::<Option<bool>, _>(i).ok().and_then(|opt| opt.map(|v| if v { b"1".to_vec() } else { b"0".to_vec() })))
                // If all Option types return None, it's a NULL value
                .unwrap_or_else(|| vec![]);

            agent_row = agent_row.with_column(col_name, Value::new(value));
        }

        Ok(agent_row)
    }
}

#[async_trait]
impl AgentDB for SqlBackend {
    fn family(&self) -> BackendFamily {
        BackendFamily::Sql
    }

    fn capabilities(&self) -> &dyn Capabilities {
        &self.capabilities
    }

    async fn put(&self, key: &str, value: Value) -> agentdb::Result<()> {
        let query = match self.backend_type {
            #[cfg(feature = "sqlite")]
            BackendType::Sqlite => {
                "INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?1, ?2, datetime('now'))"
            }
            #[cfg(feature = "postgres")]
            BackendType::Postgres => {
                "INSERT INTO kv_store (key, value, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()"
            }
            #[cfg(feature = "mysql")]
            BackendType::Mysql => {
                "INSERT INTO kv_store (`key`, value, updated_at) VALUES (?, ?, NOW()) ON DUPLICATE KEY UPDATE value = ?, updated_at = NOW()"
            }
        };

        #[cfg(feature = "mysql")]
        if matches!(self.backend_type, BackendType::Mysql) {
            sqlx::query(query)
                .bind(key)
                .bind(value.as_bytes())
                .bind(value.as_bytes())
                .execute(&self.pool)
                .await
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;
        } else {
            sqlx::query(query)
                .bind(key)
                .bind(value.as_bytes())
                .execute(&self.pool)
                .await
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;
        }

        #[cfg(not(feature = "mysql"))]
        sqlx::query(query)
            .bind(key)
            .bind(value.as_bytes())
            .execute(&self.pool)
            .await
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, key: &str) -> agentdb::Result<Option<Value>> {
        let query = match self.backend_type {
            #[cfg(feature = "mysql")]
            BackendType::Mysql => "SELECT value FROM kv_store WHERE `key` = ?",
            _ => "SELECT value FROM kv_store WHERE key = ?",
        };

        let row: Option<AnyRow> = sqlx::query(query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        match row {
            Some(row) => {
                let bytes: Vec<u8> = row
                    .try_get(0)
                    .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;
                Ok(Some(Value::new(bytes)))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, key: &str) -> agentdb::Result<()> {
        let query = match self.backend_type {
            #[cfg(feature = "mysql")]
            BackendType::Mysql => "DELETE FROM kv_store WHERE `key` = ?",
            _ => "DELETE FROM kv_store WHERE key = ?",
        };

        let result = sqlx::query(query)
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(agentdb::AgentDbError::NotFound(key.to_string()));
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> agentdb::Result<bool> {
        let query = match self.backend_type {
            #[cfg(feature = "mysql")]
            BackendType::Mysql => "SELECT 1 FROM kv_store WHERE `key` = ? LIMIT 1",
            _ => "SELECT 1 FROM kv_store WHERE key = ? LIMIT 1",
        };

        let row: Option<AnyRow> = sqlx::query(query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        Ok(row.is_some())
    }

    async fn query(&self, query_str: &str, _params: Vec<Value>) -> agentdb::Result<QueryResult> {
        // Check if this is a SELECT query
        let is_select = query_str.trim().to_uppercase().starts_with("SELECT");

        if is_select {
            let rows: Vec<AnyRow> = sqlx::query(query_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

            let agent_rows: Vec<Row> = rows
                .into_iter()
                .map(|row| self.convert_row(row))
                .collect::<Result<Vec<_>>>()
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

            Ok(QueryResult::new(agent_rows, 0))
        } else {
            let result = sqlx::query(query_str)
                .execute(&self.pool)
                .await
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

            Ok(QueryResult::new(Vec::new(), result.rows_affected() as usize))
        }
    }

    async fn scan(&self, prefix: &str) -> agentdb::Result<ScanResult> {
        let (query, pattern) = match self.backend_type {
            #[cfg(feature = "mysql")]
            BackendType::Mysql => (
                "SELECT `key` FROM kv_store WHERE `key` LIKE ? ORDER BY `key`",
                format!("{}%", prefix),
            ),
            _ => (
                "SELECT key FROM kv_store WHERE key LIKE ? ORDER BY key",
                format!("{}%", prefix),
            ),
        };

        let rows: Vec<AnyRow> = sqlx::query(query)
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        let keys: Vec<String> = rows
            .into_iter()
            .map(|row| {
                row.try_get(0)
                    .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(ScanResult::new(keys))
    }

    async fn begin(&self) -> agentdb::Result<Box<dyn Transaction>> {
        Err(agentdb::AgentDbError::Unsupported(
            "Transactions not yet implemented for SQL backend".to_string(),
        ))
    }

    async fn close(&self) -> agentdb::Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_operations() {
        let db = SqlBackend::sqlite(":memory:").await.unwrap();

        // Test put/get
        db.put("test_key", b"test_value".to_vec().into()).await.unwrap();
        let value = db.get("test_key").await.unwrap().unwrap();
        assert_eq!(value.as_bytes(), b"test_value");

        // Test exists
        assert!(db.exists("test_key").await.unwrap());
        assert!(!db.exists("nonexistent").await.unwrap());

        // Test scan
        db.put("prefix_1", b"v1".to_vec().into()).await.unwrap();
        db.put("prefix_2", b"v2".to_vec().into()).await.unwrap();
        let result = db.scan("prefix").await.unwrap();
        assert_eq!(result.keys.len(), 2);

        // Test delete
        db.delete("test_key").await.unwrap();
        assert!(!db.exists("test_key").await.unwrap());
    }
}
