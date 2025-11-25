-- SQLite Schema for AgentFS
-- Based on Agent Filesystem Specification (SPEC.md)
-- Optimized for embedded, single-file storage

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

-- Inode table: Stores file and directory metadata
CREATE TABLE IF NOT EXISTS fs_inode (
    ino INTEGER PRIMARY KEY AUTOINCREMENT,
    mode INTEGER NOT NULL,
    uid INTEGER NOT NULL DEFAULT 0,
    gid INTEGER NOT NULL DEFAULT 0,
    size INTEGER NOT NULL DEFAULT 0,
    atime INTEGER NOT NULL,
    mtime INTEGER NOT NULL,
    ctime INTEGER NOT NULL
);

-- Directory entry table: Maps names to inodes
CREATE TABLE IF NOT EXISTS fs_dentry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parent_ino INTEGER NOT NULL,
    ino INTEGER NOT NULL,
    UNIQUE(parent_ino, name)
);

CREATE INDEX IF NOT EXISTS idx_fs_dentry_parent ON fs_dentry(parent_ino, name);

-- File data table: Stores file content in chunks
CREATE TABLE IF NOT EXISTS fs_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ino INTEGER NOT NULL,
    offset INTEGER NOT NULL,
    size INTEGER NOT NULL,
    data BLOB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fs_data_ino_offset ON fs_data(ino, offset);

-- Symbolic link table: Stores symlink targets
CREATE TABLE IF NOT EXISTS fs_symlink (
    ino INTEGER PRIMARY KEY,
    target TEXT NOT NULL
);

-- Key-value store table
CREATE TABLE IF NOT EXISTS kv_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at INTEGER DEFAULT (unixepoch()),
    updated_at INTEGER DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_kv_store_created_at ON kv_store(created_at);

-- Tool calls audit table
CREATE TABLE IF NOT EXISTS tool_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parameters TEXT,
    result TEXT,
    error TEXT,
    started_at INTEGER NOT NULL,
    completed_at INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tool_calls_name ON tool_calls(name);
CREATE INDEX IF NOT EXISTS idx_tool_calls_started_at ON tool_calls(started_at);

-- Initialize root directory (ino=1) if it doesn't exist
INSERT OR IGNORE INTO fs_inode (ino, mode, uid, gid, size, atime, mtime, ctime)
VALUES (1, 16877, 0, 0, 0, unixepoch(), unixepoch(), unixepoch());
