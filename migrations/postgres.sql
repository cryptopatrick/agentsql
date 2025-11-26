-- PostgreSQL Schema for AgentFS
-- Based on Agent Filesystem Specification (SPEC.md)
-- Optimized for production deployments with JSONB support

-- Inode table: Stores file and directory metadata
CREATE TABLE IF NOT EXISTS fs_inode (
    ino BIGSERIAL PRIMARY KEY,
    mode INTEGER NOT NULL,
    uid INTEGER NOT NULL DEFAULT 0,
    gid INTEGER NOT NULL DEFAULT 0,
    size BIGINT NOT NULL DEFAULT 0,
    atime BIGINT NOT NULL,
    mtime BIGINT NOT NULL,
    ctime BIGINT NOT NULL
);

-- Directory entry table: Maps names to inodes
CREATE TABLE IF NOT EXISTS fs_dentry (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    parent_ino BIGINT NOT NULL,
    ino BIGINT NOT NULL,
    UNIQUE(parent_ino, name)
);

CREATE INDEX IF NOT EXISTS idx_fs_dentry_parent ON fs_dentry(parent_ino, name);

-- File data table: Stores file content in chunks
CREATE TABLE IF NOT EXISTS fs_data (
    id BIGSERIAL PRIMARY KEY,
    ino BIGINT NOT NULL,
    offset BIGINT NOT NULL,
    size BIGINT NOT NULL,
    data BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fs_data_ino_offset ON fs_data(ino, offset);

-- Symbolic link table: Stores symlink targets
CREATE TABLE IF NOT EXISTS fs_symlink (
    ino BIGINT PRIMARY KEY,
    target TEXT NOT NULL
);

-- Key-value store table
CREATE TABLE IF NOT EXISTS kv_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    updated_at BIGINT DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

CREATE INDEX IF NOT EXISTS idx_kv_store_created_at ON kv_store(created_at);

-- Tool calls audit table
CREATE TABLE IF NOT EXISTS tool_calls (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    parameters TEXT,
    result TEXT,
    error TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at BIGINT NOT NULL,
    completed_at BIGINT,
    duration_ms BIGINT
);

CREATE INDEX IF NOT EXISTS idx_tool_calls_name ON tool_calls(name);
CREATE INDEX IF NOT EXISTS idx_tool_calls_started_at ON tool_calls(started_at);

-- Initialize root directory (ino=1) if it doesn't exist
-- Note: PostgreSQL doesn't support INSERT OR IGNORE, use ON CONFLICT instead
INSERT INTO fs_inode (ino, mode, uid, gid, size, atime, mtime, ctime)
VALUES (1, 16877, 0, 0, 0, EXTRACT(EPOCH FROM NOW())::BIGINT, EXTRACT(EPOCH FROM NOW())::BIGINT, EXTRACT(EPOCH FROM NOW())::BIGINT)
ON CONFLICT (ino) DO NOTHING;

-- Reset sequence to start after root inode
SELECT setval('fs_inode_ino_seq', GREATEST(1, (SELECT MAX(ino) FROM fs_inode)), true);
