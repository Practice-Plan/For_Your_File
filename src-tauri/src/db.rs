//! Database initialization and schema management for the Tauri backend.
//!
//! Creates all required SQLite tables, indexes, and FTS5 triggers on startup.
//! This module exists because the Tauri backend opens the database directly
//! and must ensure the schema exists before any command runs.

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

/// Get the database path inside the app data directory, creating the directory if needed.
pub fn get_database_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    Ok(app_data_dir.join("lnk_management.db"))
}

/// Initialize the database: create all tables, indexes, and triggers if they don't exist.
///
/// This must be called once during application setup before any DB-backed command runs.
/// The schema is designed to be idempotent (uses `CREATE TABLE IF NOT EXISTS`).
pub fn init_database(app_handle: &AppHandle) -> Result<(), String> {
    let db_path = get_database_path(app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {}: {}", db_path.display(), e))?;

    // Enable foreign keys so ON DELETE CASCADE works
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

    // --- Tables ---
    // entries: lnk_path is nullable to allow saving entry info without a .lnk file (Task 7).
    // UNIQUE on lnk_path is dropped to avoid conflicts with empty strings; multiple NULL/empty
    // values are permitted. Deduplication is handled at the application level.
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            lnk_path TEXT,
            target_path TEXT NOT NULL,
            parameters TEXT,
            working_dir TEXT,
            description TEXT,
            icon_location TEXT,
            icon_index INTEGER,
            tags TEXT,
            notes TEXT,
            frequency INTEGER DEFAULT 0,
            last_opened INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            expires_at INTEGER
        )
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries table: {}", e))?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT '#3B82F6',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create groups table: {}", e))?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS entry_groups (
            entry_id INTEGER NOT NULL,
            group_id INTEGER NOT NULL,
            PRIMARY KEY (entry_id, group_id),
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
        )
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entry_groups table: {}", e))?;

    // FTS5 virtual table for full-text search over entry content
    conn.execute(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            lnk_path,
            target_path,
            tags,
            notes,
            content='entries',
            content_rowid='id'
        )
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_fts table: {}", e))?;

    // --- Indexes ---
    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_entries_frequency ON entries(frequency DESC)",
        "CREATE INDEX IF NOT EXISTS idx_entries_last_opened ON entries(last_opened DESC)",
        "CREATE INDEX IF NOT EXISTS idx_entries_lnk_path ON entries(lnk_path)",
        "CREATE INDEX IF NOT EXISTS idx_entries_target_path ON entries(target_path)",
        "CREATE INDEX IF NOT EXISTS idx_entries_expires_at ON entries(expires_at)",
        "CREATE INDEX IF NOT EXISTS idx_groups_name ON groups(name)",
        "CREATE INDEX IF NOT EXISTS idx_entry_groups_entry_id ON entry_groups(entry_id)",
        "CREATE INDEX IF NOT EXISTS idx_entry_groups_group_id ON entry_groups(group_id)",
    ];
    for sql in &indexes {
        conn.execute(sql, [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
    }

    // --- FTS5 Triggers ---
    // Keep the FTS index in sync with the entries table.
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, lnk_path, target_path, tags, notes)
            VALUES (new.id, new.lnk_path, new.target_path, new.tags, new.notes)
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_ai trigger: {}", e))?;

    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, tags, notes)
            VALUES ('delete', old.id, old.lnk_path, old.target_path, old.tags, old.notes)
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_ad trigger: {}", e))?;

    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, tags, notes)
            VALUES ('delete', old.id, old.lnk_path, old.target_path, old.tags, old.notes);
            INSERT INTO entries_fts(rowid, lnk_path, target_path, tags, notes)
            VALUES (new.id, new.lnk_path, new.target_path, new.tags, new.notes)
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_au trigger: {}", e))?;

    log::info!("Database initialized at {}", db_path.display());
    Ok(())
}
