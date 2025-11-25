//! SQLite backend implementation for AgentDB

use crate::{error::Result, schema::SCHEMA};
use agentdb::{
    AgentDB, BackendFamily, Capabilities, DefaultCapabilities, QueryResult, Row, ScanResult,
    Transaction, Value,
};
use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

/// SQLite backend for AgentDB
///
/// Provides embedded, file-based storage with ACID guarantees.
/// Uses `rusqlite` for synchronous operations wrapped in async interfaces.
pub struct SqliteBackend {
    conn: Arc<Mutex<Connection>>,
    capabilities: DefaultCapabilities,
}

impl SqliteBackend {
    /// Create a new SQLite backend
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file, or ":memory:" for in-memory
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())?;

        // Enable foreign keys and WAL mode for better concurrency
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        conn.execute("PRAGMA journal_mode = WAL", [])?;

        let backend = Self {
            conn: Arc::new(Mutex::new(conn)),
            capabilities: DefaultCapabilities {
                transactions: true,
                directories: true,
                graph_queries: false,
                sql_queries: true,
                indexes: true,
                ttl: false,
                max_key_size: Some(1024 * 1024), // 1MB
                max_value_size: Some(1024 * 1024 * 1024), // 1GB
            },
        };

        // Initialize schema
        backend.migrate().await?;

        Ok(backend)
    }

    /// Migrate database schema
    async fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute_batch(SCHEMA)?;
        Ok(())
    }
}

#[async_trait]
impl AgentDB for SqliteBackend {
    fn family(&self) -> BackendFamily {
        BackendFamily::Sql
    }

    fn capabilities(&self) -> &dyn Capabilities {
        &self.capabilities
    }

    async fn put(&self, key: &str, value: Value) -> agentdb::Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            params![key, value.as_bytes()],
        )
        .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, key: &str) -> agentdb::Result<Option<Value>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT value FROM kv_store WHERE key = ?1")
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        let result = stmt
            .query_row(params![key], |row| {
                let bytes: Vec<u8> = row.get(0)?;
                Ok(Value::new(bytes))
            })
            .optional()
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, key: &str) -> agentdb::Result<()> {
        let conn = self.conn.lock().await;
        let affected = conn
            .execute("DELETE FROM kv_store WHERE key = ?1", params![key])
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        if affected == 0 {
            return Err(agentdb::AgentDbError::NotFound(key.to_string()));
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> agentdb::Result<bool> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT 1 FROM kv_store WHERE key = ?1")
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        let exists = stmt
            .exists(params![key])
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        Ok(exists)
    }

    async fn query(&self, query: &str, _params: Vec<Value>) -> agentdb::Result<QueryResult> {
        let conn = self.conn.lock().await;

        // Check if this is a SELECT query
        let query_upper = query.trim().to_uppercase();
        if query_upper.starts_with("SELECT") {
            let mut stmt = conn
                .prepare(query)
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

            let column_names: Vec<String> = stmt
                .column_names()
                .iter()
                .map(|s| s.to_string())
                .collect();

            // For SELECT queries, we need to handle parameters differently
            // For now, we'll use a simplified approach without parameters
            let rows = stmt
                .query_map([], |sql_row| {
                    let mut row = Row::new();
                    for (i, col_name) in column_names.iter().enumerate() {
                        // Try to get as blob first, fallback to text
                        let value: Vec<u8> = sql_row.get(i).or_else(|_: rusqlite::Error| {
                            let text: String = sql_row.get(i)?;
                            Ok::<Vec<u8>, rusqlite::Error>(text.into_bytes())
                        })?;
                        row = row.with_column(col_name.clone(), Value::new(value));
                    }
                    Ok(row)
                })
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e: rusqlite::Error| agentdb::AgentDbError::Backend(e.to_string()))?;

            Ok(QueryResult::new(rows, 0))
        } else {
            // For non-SELECT queries (INSERT, UPDATE, DELETE)
            let affected = conn
                .execute(query, [])
                .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

            Ok(QueryResult::new(Vec::new(), affected))
        }
    }

    async fn scan(&self, prefix: &str) -> agentdb::Result<ScanResult> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT key FROM kv_store WHERE key LIKE ?1 || '%' ORDER BY key")
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?;

        let keys = stmt
            .query_map(params![prefix], |row| row.get(0))
            .map_err(|e| agentdb::AgentDbError::Backend(e.to_string()))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e: rusqlite::Error| agentdb::AgentDbError::Backend(e.to_string()))?;

        Ok(ScanResult::new(keys))
    }

    async fn begin(&self) -> agentdb::Result<Box<dyn Transaction>> {
        Err(agentdb::AgentDbError::Unsupported(
            "Transactions not yet implemented for SQLite backend".to_string(),
        ))
    }

    async fn close(&self) -> agentdb::Result<()> {
        // SQLite connections are closed when dropped
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_basic_operations() {
        let db = SqliteBackend::new(":memory:").await.unwrap();

        // Test put/get
        db.put("test_key", b"test_value".into()).await.unwrap();
        let value = db.get("test_key").await.unwrap().unwrap();
        assert_eq!(value.as_bytes(), b"test_value");

        // Test exists
        assert!(db.exists("test_key").await.unwrap());
        assert!(!db.exists("nonexistent").await.unwrap());

        // Test scan
        db.put("prefix_1", b"v1".into()).await.unwrap();
        db.put("prefix_2", b"v2".into()).await.unwrap();
        let result = db.scan("prefix").await.unwrap();
        assert_eq!(result.keys.len(), 2);

        // Test delete
        db.delete("test_key").await.unwrap();
        assert!(!db.exists("test_key").await.unwrap());
    }
}
