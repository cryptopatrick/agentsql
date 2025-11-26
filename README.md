<h1 align="center">
  <br>
  AGENTSQL
  <br>
</h1>

<h4 align="center">
  SQL Backend Family for AI Agent Persistence
</h4>

<p align="center">
  <a href="https://crates.io/crates/agentsql" target="_blank">
    <img src="https://img.shields.io/crates/v/agentsql" alt="Crates.io"/>
  </a>
  <a href="https://crates.io/crates/agentsql" target="_blank">
    <img src="https://img.shields.io/crates/d/agentsql" alt="Downloads"/>
  </a>
  <a href="https://docs.rs/agentsql" target="_blank">
    <img src="https://docs.rs/agentsql/badge.svg" alt="Documentation"/>
  </a>
  <a href="LICENSE" target="_blank">
    <img src="https://img.shields.io/github/license/cryptopatrick/agentsql.svg" alt="License"/>
  </a>
</p>

<b>Author's bio:</b> 👋😀 Hi, I'm CryptoPatrick! I'm currently enrolled as an
Undergraduate student in Mathematics, at Chalmers & the University of Gothenburg, Sweden. <br>
If you have any questions or need more info, then please <a href="https://discord.gg/T8EWmJZpCB">join my Discord Channel: AiMath</a>

---

<p align="center">
  <a href="#-what-is-agentsql">What is AgentSQL</a> •
  <a href="#-features">Features</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-how-to-use">How To Use</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-license">License</a>
</p>

## 🛎 Important Notices
* Supports **SQLite**, **PostgreSQL**, and **MySQL**
* Implements the **AgentDB trait** from [agentdb](../agentdb)
* Includes database migration system for schema management
* Powered by **SQLx** for type-safe SQL

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> :pushpin: Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#-what-is-agentsql">What is AgentSQL</a></li>
    <li><a href="#-features">Features</a></li>
      <ul>
        <li><a href="#-multi-database-support">Multi-Database Support</a></li>
        <li><a href="#-schema-management">Schema Management</a></li>
        <li><a href="#-production-ready">Production Ready</a></li>
      </ul>
    <li><a href="#-architecture">Architecture</a></li>
    <li><a href="#-how-to-use">How to Use</a></li>
    <li><a href="#-examples">Examples</a></li>
    <li><a href="#-testing">Testing</a></li>
    <li><a href="#-documentation">Documentation</a></li>
    <li><a href="#-author">Author</a></li>
    <li><a href="#-support">Support</a></li>
    <li><a href="#-license">License</a></li>
  </ol>
</details>

## 🤔 What is AgentSQL

`agentsql` is a production-ready SQL backend implementation for AI agent persistence. It provides a unified interface for SQLite, PostgreSQL, and MySQL databases through the `AgentDB` trait, enabling agents to store filesystems, key-value data, and tool call audit logs with a single API.

Built with SQLx for type-safe SQL, AgentSQL handles schema migrations automatically and provides seamless switching between database backends without code changes.

### Use Cases

- **Development**: Use SQLite for local development with zero configuration
- **Production**: Deploy on PostgreSQL or MySQL for multi-agent systems
- **Cloud**: Seamlessly migrate between managed database services (AWS RDS, Google Cloud SQL, Azure)
- **Testing**: Fast in-memory SQLite databases for unit tests
- **Edge**: Embedded SQLite for resource-constrained environments
- **Enterprise**: PostgreSQL/MySQL for high-availability deployments

## 📷 Features

`agentsql` provides complete SQL backend support for agent persistence with production-grade features:

### 💾 Multi-Database Support

#### **SQLite**
- **Zero Configuration**: File-based or in-memory databases
- **Single File**: Entire database in one portable file
- **Fast**: Ideal for development and embedded systems
- **ACID Compliant**: Full transaction support
- **In-Memory Mode**: Perfect for testing

#### **PostgreSQL**
- **Production Grade**: Battle-tested for high-load scenarios
- **Advanced Features**: JSONB, full-text search, concurrent access
- **Scalability**: Handles millions of records efficiently
- **Cloud Ready**: Works with AWS RDS, Google Cloud SQL, Azure Database
- **Replication**: Built-in streaming replication

#### **MySQL**
- **Wide Adoption**: Industry-standard database
- **Compatibility**: Works with MySQL, MariaDB, and cloud variants
- **Replication**: Built-in master-slave replication
- **Cloud Services**: Compatible with AWS Aurora, Google Cloud SQL
- **Performance**: Optimized for read-heavy workloads

### 🔧 Schema Management

- **Automatic Migrations**: Schema applied on first connection
- **Multi-Statement Support**: Complex migration scripts
- **Inode/Dentry Design**: Unix-like filesystem structure
- **Indexes**: Optimized for filesystem operations
- **Tool Call Auditing**: Built-in audit trail table
- **Version Control**: Schema versioning support

### 🚀 Production Ready

- **Connection Pooling**: Efficient connection management via SQLx
- **Async Operations**: Full async/await support with Tokio
- **Error Handling**: Detailed error messages with context
- **Type Safety**: Compile-time SQL verification (SQLx)
- **NULL Handling**: Proper handling of optional fields
- **Performance**: Optimized queries with prepared statements

## 📐 Architecture

1. 🏛 **Overall Architecture**

```diagram
┌─────────────────────────────────────────────────┐
│            AgentFS High-Level APIs              │
│   (FileSystem, KvStore, ToolRecorder)           │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│             AgentDB Trait Interface             │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│              AgentSQL (SQLx)                    │
│  • Connection pooling                           │
│  • Query builder                                │
│  • Migration system                             │
│  • Type conversions                             │
└───┬─────────────────┬────────────────┬──────────┘
    │                 │                │
┌───▼──────┐   ┌──────▼──────┐  ┌─────▼──────┐
│  SQLite  │   │ PostgreSQL  │  │   MySQL    │
│  Local   │   │  Production │  │   Cloud    │
└──────────┘   └─────────────┘  └────────────┘
```

2. 💾 **Database Schema**

```diagram
┌────────────────────────────────────────────────┐
│           SQLite / PostgreSQL / MySQL          │
│                                                │
│  ┌──────────────────────────────────────────┐ │
│  │          fs_inode (File Metadata)        │ │
│  │  - ino (PK, AUTO_INCREMENT)              │ │
│  │  - mode (permissions)                    │ │
│  │  - uid, gid                              │ │
│  │  - size, atime, mtime, ctime             │ │
│  └──────────────────────────────────────────┘ │
│                                                │
│  ┌──────────────────────────────────────────┐ │
│  │      fs_dentry (Directory Entries)       │ │
│  │  - id (PK)                               │ │
│  │  - name                                  │ │
│  │  - parent_ino (FK → fs_inode)            │ │
│  │  - ino (FK → fs_inode)                   │ │
│  │  UNIQUE(parent_ino, name)                │ │
│  └──────────────────────────────────────────┘ │
│                                                │
│  ┌──────────────────────────────────────────┐ │
│  │         fs_data (File Content)           │ │
│  │  - id (PK)                               │ │
│  │  - ino (FK → fs_inode)                   │ │
│  │  - offset                                │ │
│  │  - size                                  │ │
│  │  - data (BLOB/BYTEA)                     │ │
│  │  INDEX(ino, offset)                      │ │
│  └──────────────────────────────────────────┘ │
│                                                │
│  ┌──────────────────────────────────────────┐ │
│  │           kv_store (Key-Value)           │ │
│  │  - key (PK)                              │ │
│  │  - value (TEXT)                          │ │
│  │  - created_at, updated_at                │ │
│  └──────────────────────────────────────────┘ │
│                                                │
│  ┌──────────────────────────────────────────┐ │
│  │      tool_calls (Audit Trail)            │ │
│  │  - id (PK)                               │ │
│  │  - name                                  │ │
│  │  - parameters (JSON)                     │ │
│  │  - result (JSON)                         │ │
│  │  - error                                 │ │
│  │  - status (pending/success/error)        │ │
│  │  - started_at, completed_at              │ │
│  │  - duration_ms                           │ │
│  │  INDEX(name), INDEX(started_at)          │ │
│  └──────────────────────────────────────────┘ │
└────────────────────────────────────────────────┘
```

3. 🔄 **Migration Flow**

```diagram
┌────────────────────────────────────────┐
│     SqlBackend::new(config)            │
└─────────────────┬──────────────────────┘
                  │
         ┌────────▼──────────┐
         │  Detect Backend   │
         │  SQLite/PG/MySQL  │
         └────────┬──────────┘
                  │
         ┌────────▼──────────┐
         │  Load Migration   │
         │  SQL for backend  │
         └────────┬──────────┘
                  │
         ┌────────▼──────────┐
         │  Execute Schema   │
         │  Multi-statement  │
         └────────┬──────────┘
                  │
         ┌────────▼──────────┐
         │  Initialize Root  │
         │  inode (ino=1)    │
         └────────┬──────────┘
                  │
         ┌────────▼──────────┐
         │  Ready for Use!   │
         └───────────────────┘
```

## 🚙 How to Use

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agentdb = "0.1"
agentsql = "0.1"

# Enable the backend(s) you need:
agentsql = { version = "0.1", features = ["sqlite"] }
# agentsql = { version = "0.1", features = ["postgres"] }
# agentsql = { version = "0.1", features = ["mysql"] }
# agentsql = { version = "0.1", features = ["sqlite", "postgres", "mysql"] }
```

Or install with cargo:

```bash
cargo add agentsql --features sqlite
```

### Example: SQLite (Local Development)

```rust
use agentsql::SqlBackend;
use agentdb::AgentDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create SQLite database (file-based)
    let db = SqlBackend::sqlite("agent.db").await?;

    // Or use in-memory for testing
    let db = SqlBackend::sqlite(":memory:").await?;

    // Key-value operations
    db.put("config:theme", b"dark".to_vec().into()).await?;

    let theme = db.get("config:theme").await?.unwrap();
    println!("Theme: {}", String::from_utf8_lossy(theme.as_bytes()));

    // SQL queries
    let result = db.query(
        "SELECT * FROM fs_inode WHERE ino = 1",
        vec![]
    ).await?;

    println!("Root inode: {:?}", result.rows.first());

    Ok(())
}
```

### Example: PostgreSQL (Production)

```rust
use agentsql::SqlBackend;
use agentdb::AgentDB;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PostgreSQL
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost/agentfs".to_string());

    let db = SqlBackend::postgres(database_url).await?;

    // Same API as SQLite!
    db.put("agent:status", b"running".to_vec().into()).await?;

    // Execute queries
    let result = db.query(
        "SELECT COUNT(*) as count FROM tool_calls WHERE status = 'success'",
        vec![]
    ).await?;

    Ok(())
}
```

### Example: MySQL (Cloud Deployment)

```rust
use agentsql::SqlBackend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to MySQL (e.g., AWS Aurora)
    let db = SqlBackend::mysql(
        "mysql://user:pass@aurora-cluster.region.rds.amazonaws.com/agentfs"
    ).await?;

    // Scan with prefix
    let sessions = db.scan("session:").await?;
    println!("Found {} active sessions", sessions.keys.len());

    Ok(())
}
```

## 🧪 Examples

See the [agentfs](../agentfs) crate for complete examples demonstrating:
- Basic SQLite usage
- PostgreSQL multi-agent systems
- MySQL cloud deployments

## 🧪 Testing

Run the test suite:

```bash
# Run all tests (SQLite)
cargo test

# Test with PostgreSQL
cargo test --features postgres

# Test with MySQL
cargo test --features mysql

# Run with output
cargo test -- --nocapture
```

## 📚 Documentation

Comprehensive documentation is available at [docs.rs/agentsql](https://docs.rs/agentsql), including:
- API reference for `SqlBackend`
- Migration system details
- Database-specific configuration
- Performance tuning guides
- Connection pooling best practices

## 🖊 Author

<a href="https://x.com/cryptopatrick">CryptoPatrick</a>

Keybase Verification:
https://keybase.io/cryptopatrick/sigs/8epNh5h2FtIX1UNNmf8YQ-k33M8J-Md4LnAN

## 🐣 Support

Leave a ⭐ if you think this project is cool.

## 🗄 License

This project is licensed under MIT. See [LICENSE](LICENSE) for details.
