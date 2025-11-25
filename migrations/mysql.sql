-- MySQL Schema for AgentFS
-- Based on Agent Filesystem Specification (SPEC.md)
-- Optimized for MySQL/MariaDB deployments

-- Inode table: Stores file and directory metadata
CREATE TABLE IF NOT EXISTS fs_inode (
    ino BIGINT PRIMARY KEY AUTO_INCREMENT,
    mode INTEGER NOT NULL,
    uid INTEGER NOT NULL DEFAULT 0,
    gid INTEGER NOT NULL DEFAULT 0,
    size BIGINT NOT NULL DEFAULT 0,
    atime BIGINT NOT NULL,
    mtime BIGINT NOT NULL,
    ctime BIGINT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Directory entry table: Maps names to inodes
CREATE TABLE IF NOT EXISTS fs_dentry (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    parent_ino BIGINT NOT NULL,
    ino BIGINT NOT NULL,
    UNIQUE KEY unique_parent_name (parent_ino, name),
    INDEX idx_fs_dentry_parent (parent_ino, name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- File data table: Stores file content in chunks
CREATE TABLE IF NOT EXISTS fs_data (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    ino BIGINT NOT NULL,
    `offset` BIGINT NOT NULL,
    size BIGINT NOT NULL,
    data LONGBLOB NOT NULL,
    INDEX idx_fs_data_ino_offset (ino, `offset`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Symbolic link table: Stores symlink targets
CREATE TABLE IF NOT EXISTS fs_symlink (
    ino BIGINT PRIMARY KEY,
    target TEXT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Key-value store table
CREATE TABLE IF NOT EXISTS kv_store (
    `key` VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    created_at BIGINT DEFAULT (UNIX_TIMESTAMP()),
    updated_at BIGINT DEFAULT (UNIX_TIMESTAMP()),
    INDEX idx_kv_store_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Tool calls audit table
CREATE TABLE IF NOT EXISTS tool_calls (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    parameters TEXT,
    result TEXT,
    error TEXT,
    started_at BIGINT NOT NULL,
    completed_at BIGINT NOT NULL,
    duration_ms BIGINT NOT NULL,
    INDEX idx_tool_calls_name (name),
    INDEX idx_tool_calls_started_at (started_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Initialize root directory (ino=1) if it doesn't exist
-- MySQL doesn't support INSERT IGNORE with specific values in AUTO_INCREMENT column
-- We use a stored procedure approach
INSERT INTO fs_inode (ino, mode, uid, gid, size, atime, mtime, ctime)
SELECT 1, 16877, 0, 0, 0, UNIX_TIMESTAMP(), UNIX_TIMESTAMP(), UNIX_TIMESTAMP()
WHERE NOT EXISTS (SELECT 1 FROM fs_inode WHERE ino = 1);

-- Set AUTO_INCREMENT to start after root inode
ALTER TABLE fs_inode AUTO_INCREMENT = 2;
