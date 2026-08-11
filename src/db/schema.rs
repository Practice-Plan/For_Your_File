//! Database schema constants and migrations
//!
//! Contains all SQL statements for schema creation and management.

/// Current schema version
pub const SCHEMA_VERSION: i32 = 1;

/// Create entries table
pub const CREATE_ENTRIES_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS entries (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        lnk_path TEXT NOT NULL UNIQUE,
        target_path TEXT NOT NULL,
        parameters TEXT,
        working_dir TEXT,
        tags TEXT,
        notes TEXT,
        frequency INTEGER DEFAULT 0,
        last_opened INTEGER,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        expires_at INTEGER
    )
"#;

/// Create groups table
pub const CREATE_GROUPS_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS groups (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        color TEXT NOT NULL DEFAULT '#3498db',
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    )
"#;

/// Create entry_groups junction table for many-to-many relationship
pub const CREATE_ENTRY_GROUPS_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS entry_groups (
        entry_id INTEGER NOT NULL,
        group_id INTEGER NOT NULL,
        PRIMARY KEY (entry_id, group_id),
        FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
        FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
    )
"#;

/// Create FTS5 virtual table for full-text search
pub const CREATE_ENTRIES_FTS_TABLE: &str = r#"
    CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
        lnk_path,
        target_path,
        tags,
        notes,
        content='entries',
        content_rowid='id'
    )
"#;

/// Create version tracking table for migrations
pub const CREATE_VERSION_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY,
        applied_at INTEGER NOT NULL
    )
"#;

/// Indexes for entries table
pub const CREATE_IDX_ENTRIES_FREQUENCY: &str =
    "CREATE INDEX IF NOT EXISTS idx_entries_frequency ON entries(frequency DESC)";

pub const CREATE_IDX_ENTRIES_LAST_OPENED: &str =
    "CREATE INDEX IF NOT EXISTS idx_entries_last_opened ON entries(last_opened DESC)";

pub const CREATE_IDX_ENTRIES_LNK_PATH: &str =
    "CREATE INDEX IF NOT EXISTS idx_entries_lnk_path ON entries(lnk_path)";

pub const CREATE_IDX_ENTRIES_TARGET_PATH: &str =
    "CREATE INDEX IF NOT EXISTS idx_entries_target_path ON entries(target_path)";

pub const CREATE_IDX_ENTRIES_EXPIRES_AT: &str =
    "CREATE INDEX IF NOT EXISTS idx_entries_expires_at ON entries(expires_at)";

/// Indexes for groups table
pub const CREATE_IDX_GROUPS_NAME: &str =
    "CREATE INDEX IF NOT EXISTS idx_groups_name ON groups(name)";

/// Indexes for entry_groups junction table
pub const CREATE_IDX_ENTRY_GROUPS_ENTRY_ID: &str =
    "CREATE INDEX IF NOT EXISTS idx_entry_groups_entry_id ON entry_groups(entry_id)";

pub const CREATE_IDX_ENTRY_GROUPS_GROUP_ID: &str =
    "CREATE INDEX IF NOT EXISTS idx_entry_groups_group_id ON entry_groups(group_id)";

/// FTS5 trigger: after insert on entries
pub const CREATE_TRIGGER_ENTRIES_AI: &str = r#"
    CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
        INSERT INTO entries_fts(rowid, lnk_path, target_path, tags, notes)
        VALUES (new.id, new.lnk_path, new.target_path, new.tags, new.notes)
    END
"#;

/// FTS5 trigger: after delete on entries
pub const CREATE_TRIGGER_ENTRIES_AD: &str = r#"
    CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
        INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, tags, notes)
        VALUES ('delete', old.id, old.lnk_path, old.target_path, old.tags, old.notes)
    END
"#;

/// FTS5 trigger: after update on entries
pub const CREATE_TRIGGER_ENTRIES_AU: &str = r#"
    CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
        INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, tags, notes)
        VALUES ('delete', old.id, old.lnk_path, old.target_path, old.tags, old.notes);
        INSERT INTO entries_fts(rowid, lnk_path, target_path, tags, notes)
        VALUES (new.id, new.lnk_path, new.target_path, new.tags, new.notes)
    END
"#;

/// Get current schema version
pub const GET_SCHEMA_VERSION: &str = "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1";

/// Insert schema version
pub const INSERT_SCHEMA_VERSION: &str = "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)";

/// All table creation statements in order
pub const TABLE_STATEMENTS: &[&str] = &[
    CREATE_ENTRIES_TABLE,
    CREATE_GROUPS_TABLE,
    CREATE_ENTRY_GROUPS_TABLE,
    CREATE_ENTRIES_FTS_TABLE,
    CREATE_VERSION_TABLE,
];

/// All index creation statements
pub const INDEX_STATEMENTS: &[&str] = &[
    CREATE_IDX_ENTRIES_FREQUENCY,
    CREATE_IDX_ENTRIES_LAST_OPENED,
    CREATE_IDX_ENTRIES_LNK_PATH,
    CREATE_IDX_ENTRIES_TARGET_PATH,
    CREATE_IDX_ENTRIES_EXPIRES_AT,
    CREATE_IDX_GROUPS_NAME,
    CREATE_IDX_ENTRY_GROUPS_ENTRY_ID,
    CREATE_IDX_ENTRY_GROUPS_GROUP_ID,
];

/// All trigger creation statements
pub const TRIGGER_STATEMENTS: &[&str] = &[
    CREATE_TRIGGER_ENTRIES_AI,
    CREATE_TRIGGER_ENTRIES_AD,
    CREATE_TRIGGER_ENTRIES_AU,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        assert_eq!(SCHEMA_VERSION, 1);
    }

    #[test]
    fn test_table_statements_not_empty() {
        assert!(!TABLE_STATEMENTS.is_empty());
        assert_eq!(TABLE_STATEMENTS.len(), 5);
    }

    #[test]
    fn test_index_statements_not_empty() {
        assert!(!INDEX_STATEMENTS.is_empty());
        assert_eq!(INDEX_STATEMENTS.len(), 8);
    }

    #[test]
    fn test_trigger_statements_not_empty() {
        assert!(!TRIGGER_STATEMENTS.is_empty());
        assert_eq!(TRIGGER_STATEMENTS.len(), 3);
    }
}