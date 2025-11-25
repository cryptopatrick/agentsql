//! Error types for AgentSQL

use agentdb::AgentDbError;
use thiserror::Error;

/// Result type for AgentSQL operations
pub type Result<T> = std::result::Result<T, SqlError>;

/// SQL-specific error types
#[derive(Error, Debug)]
pub enum SqlError {
    #[error("SQLx error: {0}")]
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    AgentDb(#[from] AgentDbError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<SqlError> for AgentDbError {
    fn from(err: SqlError) -> Self {
        match err {
            SqlError::Connection(msg) => AgentDbError::Connection(msg),
            SqlError::Query(msg) => AgentDbError::Backend(msg),
            SqlError::Serialization(e) => AgentDbError::Serialization(e.to_string()),
            SqlError::Io(e) => AgentDbError::Io(e),
            SqlError::AgentDb(e) => e,
            #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
            SqlError::Sqlx(e) => AgentDbError::Backend(e.to_string()),
            SqlError::Migration(msg) => AgentDbError::Backend(msg),
        }
    }
}
