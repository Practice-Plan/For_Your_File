//! Database initialization and schema management for the Tauri backend.
//!
//! Creates all required SQLite tables, indexes, and FTS5 triggers on startup.
//! This module exists because the Tauri backend opens the database directly
//! and must ensure the schema exists before any command runs.

use rusqlite::Connection;
use tauri::AppHandle;

/// Get the database path inside the app data directory, creating the directory if needed.
///
/// The database is stored under `%APPDATA%/wang.station/app/For_Your_File/`.
pub fn get_database_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = crate::ppc_linker::resolve_data_dir(app_handle)?;
    Ok(data_dir.join("lnk_management.db"))
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

    // FTS5 virtual table for full-text search over entry content.
    // Includes `description` (the user-entered entry name) so searching by name works.
    conn.execute(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            lnk_path,
            target_path,
            description,
            tags,
            notes,
            content='entries',
            content_rowid='id'
        )
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_fts table: {}", e))?;

    // --- Migration: rebuild FTS table if it lacks the `description` column ---
    // Older versions of the app created entries_fts without `description`.
    // We detect this by checking the FTS schema and rebuild if needed.
    migrate_fts_if_needed(&conn)?;

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
    //
    // For external-content FTS5 tables (content='entries'), we follow the
    // official SQLite FTS5 documentation's recommended trigger pattern
    // (https://www.sqlite.org/fts5.html#external_content_tables):
    //
    //   - INSERT trigger: direct column insertion (FTS5 has no 'insert'
    //     command form for external-content tables).
    //   - DELETE trigger: use the 'delete' command form, which requires ALL
    //     columns to be specified (not just the rowid):
    //       INSERT INTO ft(ft, rowid, col1, col2, ...) VALUES('delete', $rowid, $old1, $old2, ...);
    //   - UPDATE trigger: 'delete' the old row (with all old column values),
    //     then insert the new row by column name.
    //
    // IMPORTANT: We DROP existing triggers before creating them (instead of
    // using `CREATE TRIGGER IF NOT EXISTS`). This ensures old, broken
    // trigger definitions (from previous app versions that used the invalid
    // `'insert'` command form or omitted column values in the 'delete'
    // command) are always replaced with the correct syntax.
    conn.execute("DROP TRIGGER IF EXISTS entries_ai", [])
        .map_err(|e| format!("Failed to drop trigger entries_ai: {}", e))?;
    conn.execute("DROP TRIGGER IF EXISTS entries_ad", [])
        .map_err(|e| format!("Failed to drop trigger entries_ad: {}", e))?;
    conn.execute("DROP TRIGGER IF EXISTS entries_au", [])
        .map_err(|e| format!("Failed to drop trigger entries_au: {}", e))?;

    conn.execute(
        r#"
        CREATE TRIGGER entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, lnk_path, target_path, description, tags, notes)
            VALUES (new.id, new.lnk_path, new.target_path, new.description, new.tags, new.notes);
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_ai trigger: {}", e))?;

    conn.execute(
        r#"
        CREATE TRIGGER entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, description, tags, notes)
            VALUES ('delete', old.id, old.lnk_path, old.target_path, old.description, old.tags, old.notes);
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_ad trigger: {}", e))?;

    conn.execute(
        r#"
        CREATE TRIGGER entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, description, tags, notes)
            VALUES ('delete', old.id, old.lnk_path, old.target_path, old.description, old.tags, old.notes);
            INSERT INTO entries_fts(rowid, lnk_path, target_path, description, tags, notes)
            VALUES (new.id, new.lnk_path, new.target_path, new.description, new.tags, new.notes);
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create entries_au trigger: {}", e))?;

    // --- FTS Index Repair ---
    // If the FTS index was previously built with broken triggers (direct
    // column form that silently failed due to PK violations), we need to
    // re-index all existing entries from the content table.
    // This uses the FTS command form to tell FTS5 to read from entries.
    let fts_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entries_fts", [], |row| row.get(0))
        .unwrap_or(0);

    let entry_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
        .unwrap_or(0);

    if fts_count != entry_count {
        log::warn!(
            "FTS index out of sync ({} indexed vs {} entries), rebuilding...",
            fts_count,
            entry_count
        );
        // Delete existing FTS index using the 'delete-all' command form.
        // This is the correct FTS5 command for clearing an external-content
        // table's index. (The previous code used 'delete' which deletes a
        // single row, not the whole index.)
        conn.execute(
            "INSERT INTO entries_fts(entries_fts) VALUES('delete-all')",
            [],
        )
        .map_err(|e| format!("Failed to clear FTS index: {}", e))?;
        // Re-index all entries by directly inserting their content into the
        // FTS table by column name. FTS5 has no 'insert' command form for
        // external-content tables, so we must provide the data directly.
        conn.execute(
            r#"
            INSERT INTO entries_fts(rowid, lnk_path, target_path, description, tags, notes)
            SELECT id, lnk_path, target_path, description, tags, notes FROM entries
            "#,
            [],
        )
        .map_err(|e| format!("Failed to rebuild FTS index: {}", e))?;
        log::info!("FTS index rebuilt from {} entries", entry_count);
    }

    log::info!("Database initialized at {}", db_path.display());
    Ok(())
}

/// Check whether the `entries_fts` table includes the `description` column.
/// If it doesn't (older schema), drop and rebuild the FTS table + triggers
/// so that searching by entry name (description) works correctly.
fn migrate_fts_if_needed(conn: &Connection) -> Result<(), String> {
    // Query the FTS5 table schema to check for the `description` column.
    // FTS5 exposes columns via `pragma_table_info` on the virtual table.
    let has_description: bool = {
        let mut stmt = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='entries_fts'")
            .map_err(|e| format!("Failed to query FTS schema: {}", e))?;
        let sql_text: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        sql_text
            .map(|s| s.to_lowercase().contains("description"))
            .unwrap_or(false)
    };

    if has_description {
        // Schema already includes description — no migration needed
        return Ok(());
    }

    log::info!("Migrating entries_fts: adding description column (rebuilding FTS table)...");

    // Drop old triggers first (they reference the old FTS schema)
    conn.execute("DROP TRIGGER IF EXISTS entries_ai", [])
        .map_err(|e| format!("Failed to drop trigger entries_ai: {}", e))?;
    conn.execute("DROP TRIGGER IF EXISTS entries_ad", [])
        .map_err(|e| format!("Failed to drop trigger entries_ad: {}", e))?;
    conn.execute("DROP TRIGGER IF EXISTS entries_au", [])
        .map_err(|e| format!("Failed to drop trigger entries_au: {}", e))?;

    // Drop and recreate the FTS table with the new schema
    conn.execute("DROP TABLE IF EXISTS entries_fts", [])
        .map_err(|e| format!("Failed to drop entries_fts: {}", e))?;

    conn.execute(
        r#"
        CREATE VIRTUAL TABLE entries_fts USING fts5(
            lnk_path,
            target_path,
            description,
            tags,
            notes,
            content='entries',
            content_rowid='id'
        )
        "#,
        [],
    )
    .map_err(|e| format!("Failed to recreate entries_fts: {}", e))?;

    // Rebuild the FTS index from existing entries by directly inserting
    // content by column name. FTS5 has no 'insert' command form for
    // external-content tables, so direct column insertion is required.
    conn.execute(
        r#"
        INSERT INTO entries_fts(rowid, lnk_path, target_path, description, tags, notes)
        SELECT id, lnk_path, target_path, description, tags, notes FROM entries
        "#,
        [],
    )
    .map_err(|e| format!("Failed to rebuild FTS index: {}", e))?;

    log::info!("FTS migration complete: entries_fts now includes description");
    Ok(())
}
