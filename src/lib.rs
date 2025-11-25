//! # AgentSQL - SQL Backend Implementation for AgentDB
//!
//! AgentSQL provides SQL-based implementations of the AgentDB trait,
//! supporting SQLite, PostgreSQL, and MySQL backends via SQLx.
//!
//! ## Backends
//!
//! - **SQLite**: Embedded, single-file database (async via sqlx)
//! - **PostgreSQL**: Remote database (async via sqlx)
//! - **MySQL**: Remote database (async via sqlx)
//!
//! ## Example
//!
//! ```rust,ignore
//! use agentsql::{SqlBackend, SqlBackendConfig};
//! use agentdb::AgentDB;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // SQLite
//!     let db = SqlBackend::sqlite("agent.db").await?;
//!     db.put("key", b"value".into()).await?;
//!
//!     // PostgreSQL
//!     let db = SqlBackend::postgres("postgres://user:pass@localhost/db").await?;
//!
//!     // MySQL
//!     let db = SqlBackend::mysql("mysql://user:pass@localhost/db").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod backend;
pub mod error;
pub mod schema;

pub use backend::{SqlBackend, SqlBackendConfig};
pub use error::{Result, SqlError};
