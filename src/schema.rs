//! Database schema definitions for AgentSQL
//!
//! Based on the Agent Filesystem Specification (SPEC.md).
//! All schemas use the inode/dentry design for Unix-like filesystem semantics.

/// File type constants for mode field
pub mod mode {
    pub const S_IFMT: u32 = 0o170000;   // File type mask
    pub const S_IFREG: u32 = 0o100000;  // Regular file
    pub const S_IFDIR: u32 = 0o040000;  // Directory
    pub const S_IFLNK: u32 = 0o120000;  // Symbolic link

    // Default permissions
    pub const DEFAULT_FILE_MODE: u32 = S_IFREG | 0o644; // Regular file, rw-r--r--
    pub const DEFAULT_DIR_MODE: u32 = S_IFDIR | 0o755;  // Directory, rwxr-xr-x
}

/// Root inode number (always 1)
pub const ROOT_INO: i64 = 1;
