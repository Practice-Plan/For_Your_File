//! Database CRUD operations
//!
//! Provides comprehensive CRUD operations for entries with proper metadata management.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

use crate::lnk::{create_lnk_file, WindowState};
use crate::models::{Entry, EntryFilter, EntryUpdate, FromRow, Group};

/// Operations for managing entries in the database
pub struct EntryOperations<'a> {
    conn: &'a Connection,
}

impl<'a> EntryOperations<'a> {
    /// Create a new EntryOperations instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new entry with associated .lnk file
    ///
    /// This will:
    /// 1. Create the .lnk file using LnkManager
    /// 2. Insert metadata into entries table
    /// 3. Insert tags and notes into FTS5 table (via trigger)
    /// 4. Return the created entry with ID
    pub fn create_entry(&self, entry: &Entry) -> Result<Entry> {
        // Create the .lnk file
        #[cfg(windows)]
        {
            create_lnk_file(
                &entry.lnk_path,
                &entry.target_path,
                entry.parameters.as_deref(),
                entry.working_dir.as_deref(),
                entry.description.as_deref(),
                entry.icon_location.as_deref(),
                entry.icon_index,
                WindowState::Normal,
            )
            .context(format!(
                "Failed to create .lnk file at: {}",
                entry.lnk_path
            ))?;
        }

        // Insert into database
        self.conn.execute(
            r#"
            INSERT INTO entries (lnk_path, target_path, parameters, working_dir, tags, notes,
                                 frequency, last_opened, created_at, updated_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            rusqlite::params![
                entry.lnk_path,
                entry.target_path,
                entry.parameters,
                entry.working_dir,
                entry.tags,
                entry.notes,
                entry.frequency,
                entry.last_opened,
                entry.created_at,
                entry.updated_at,
                entry.expires_at,
            ],
        )
        .context("Failed to insert entry into database")?;

        let id = self.conn.last_insert_rowid();

        // Return the created entry with ID
        let mut created_entry = entry.clone();
        created_entry.id = Some(id);
        Ok(created_entry)
    }

    /// Read an entry by ID
    ///
    /// Returns the entry with all metadata fields populated
    pub fn read_entry(&self, id: i64) -> Result<Option<Entry>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, lnk_path, target_path, parameters, working_dir, tags, notes,
                       frequency, last_opened, created_at, updated_at, expires_at
                FROM entries WHERE id = ?1
                "#,
            )
            .context("Failed to prepare read statement")?;

        let result = stmt
            .query_row(rusqlite::params![id], |row| Entry::from_row(row))
            .optional()
            .context("Failed to query entry")?;

        Ok(result)
    }

    /// Update an existing entry
    ///
    /// This will:
    /// 1. Validate the entry exists
    /// 2. Update metadata in entries table
    /// 3. Update FTS5 index (via trigger)
    /// 4. Return the updated entry
    pub fn update_entry(&self, id: i64, update: &EntryUpdate) -> Result<Option<Entry>> {
        // First check if entry exists
        let existing = self
            .read_entry(id)?
            .context(format!("Entry with ID {} not found", id))?;

        if !update.has_updates() {
            return Ok(Some(existing));
        }

        // Build dynamic update query
        let mut updates: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref target_path) = update.target_path {
            updates.push("target_path = ?".to_string());
            params.push(Box::new(target_path.clone() as String));
        }
        if let Some(ref parameters) = update.parameters {
            updates.push("parameters = ?".to_string());
            params.push(Box::new(parameters.clone() as String));
        }
        if let Some(ref working_dir) = update.working_dir {
            updates.push("working_dir = ?".to_string());
            params.push(Box::new(working_dir.clone() as String));
        }
        if let Some(ref tags) = update.tags {
            updates.push("tags = ?".to_string());
            params.push(Box::new(tags.clone() as String));
        }
        if let Some(ref notes) = update.notes {
            updates.push("notes = ?".to_string());
            params.push(Box::new(notes.clone() as String));
        }
        if let Some(group_id) = update.group_id {
            updates.push("group_id = ?".to_string());
            params.push(Box::new(group_id));
        }
        if let Some(expires_at) = update.expires_at {
            updates.push("expires_at = ?".to_string());
            params.push(Box::new(expires_at));
        }

        if updates.is_empty() {
            return Ok(Some(existing));
        }

        // Add updated_at timestamp
        let now = chrono::Utc::now().timestamp();
        updates.push("updated_at = ?".to_string());
        params.push(Box::new(now));

        // Add ID for WHERE clause
        params.push(Box::new(id));

        let sql = format!(
            "UPDATE entries SET {} WHERE id = ?",
            updates.join(", ")
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.conn
            .execute(&sql, params_refs.as_slice())
            .context("Failed to update entry")?;

        // Return the updated entry
        self.read_entry(id)
    }

    /// Delete an entry by ID
    ///
    /// This will:
    /// 1. Delete the .lnk file
    /// 2. Delete metadata from entries table
    /// 3. Delete from FTS5 table (via trigger)
    /// 4. Return success status
    pub fn delete_entry(&self, id: i64) -> Result<bool> {
        // Get the entry to find the .lnk file path
        let entry = self.read_entry(id)?;

        if let Some(entry) = entry {
            // Delete the .lnk file
            if std::path::Path::new(&entry.lnk_path).exists() {
                std::fs::remove_file(&entry.lnk_path).ok(); // Ignore errors for file deletion
            }

            // Delete from database
            self.conn
                .execute("DELETE FROM entries WHERE id = ?1", rusqlite::params![id])
                .context("Failed to delete entry from database")?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List entries with filtering and pagination
    ///
    /// Supports:
    /// - Filtering by group, tags, frequency, dates
    /// - Full-text search
    /// - Pagination with limit/offset
    pub fn list_entries(&self, filter: &EntryFilter) -> Result<Vec<Entry>> {
        // Handle FTS search separately
        if let Some(ref query) = filter.search_query {
            return self.search_entries(query, filter.limit, filter.offset);
        }

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref tags) = filter.tags {
            conditions.push("tags LIKE ?".to_string());
            params.push(Box::new(format!("%{}%", tags) as String));
        }
        if let Some(min_freq) = filter.min_frequency {
            conditions.push("frequency >= ?".to_string());
            params.push(Box::new(min_freq));
        }
        if let Some(opened_after) = filter.opened_after {
            conditions.push("last_opened > ?".to_string());
            params.push(Box::new(opened_after));
        }
        if let Some(created_after) = filter.created_after {
            conditions.push("created_at > ?".to_string());
            params.push(Box::new(created_after));
        }
        if let Some(expires_before) = filter.expires_before {
            conditions.push("(expires_at IS NOT NULL AND expires_at < ?)".to_string());
            params.push(Box::new(expires_before));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let limit = filter.limit.unwrap_or(100);
        params.push(Box::new(limit));

        let offset = filter.offset.unwrap_or(0);
        params.push(Box::new(offset));

        let sql = format!(
            r#"
            SELECT id, lnk_path, target_path, parameters, working_dir, tags, notes,
                   frequency, last_opened, created_at, updated_at, expires_at
            FROM entries
            {}
            ORDER BY updated_at DESC
            LIMIT ? OFFSET ?
            "#,
            where_clause
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self
            .conn
            .prepare(&sql)
            .context("Failed to prepare list statement")?;

        let entries = stmt
            .query_map(params_refs.as_slice(), |row| Entry::from_row(row))
            .context("Failed to map entries")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect entries")?;

        Ok(entries)
    }

    /// Search entries using FTS5
    fn search_entries(&self, query: &str, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Entry>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
                       e.tags, e.notes, e.frequency, e.last_opened,
                       e.created_at, e.updated_at, e.expires_at
                FROM entries e
                JOIN entries_fts fts ON e.id = fts.rowid
                WHERE entries_fts MATCH ?
                ORDER BY bm25(entries_fts) DESC, e.frequency DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .context("Failed to prepare search statement")?;

        let entries = stmt
            .query_map(rusqlite::params![query, limit, offset], |row| Entry::from_row(row))
            .context("Failed to map search results")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect search results")?;

        Ok(entries)
    }

    /// Get the total count of entries matching a filter
    pub fn count_entries(&self, filter: &EntryFilter) -> Result<i64> {
        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref tags) = filter.tags {
            conditions.push("tags LIKE ?".to_string());
            params.push(Box::new(format!("%{}%", tags) as String));
        }
        if let Some(min_freq) = filter.min_frequency {
            conditions.push("frequency >= ?".to_string());
            params.push(Box::new(min_freq));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let sql = format!("SELECT COUNT(*) FROM entries{}", where_clause);

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let count: i64 = self
            .conn
            .query_row(&sql, params_refs.as_slice(), |row| row.get(0))
            .context("Failed to count entries")?;

        Ok(count)
    }
}

/// Insert a new entry into the database (legacy function)
pub fn insert_entry(conn: &Connection, entry: &Entry) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO entries (lnk_path, target_path, parameters, working_dir, tags, notes,
                             frequency, last_opened, created_at, updated_at, expires_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        rusqlite::params![
            entry.lnk_path,
            entry.target_path,
            entry.parameters,
            entry.working_dir,
            entry.tags,
            entry.notes,
            entry.frequency,
            entry.last_opened,
            entry.created_at,
            entry.updated_at,
            entry.expires_at,
        ],
    )
    .context("Failed to insert entry")?;

    Ok(conn.last_insert_rowid())
}

/// Get an entry by ID using FromRow trait
pub fn get_entry_by_id(conn: &Connection, id: i64) -> Result<Option<Entry>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, lnk_path, target_path, parameters, working_dir, tags, notes,
                   frequency, last_opened, created_at, updated_at, expires_at
            FROM entries WHERE id = ?1
            "#,
        )
        .context("Failed to prepare statement")?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| Entry::from_row(row))
        .optional()
        .context("Failed to query entry")?;

    Ok(result)
}

/// Update an existing entry (legacy function)
pub fn update_entry(conn: &Connection, entry: &Entry) -> Result<()> {
    conn.execute(
        r#"
        UPDATE entries SET
            lnk_path = ?1, target_path = ?2, parameters = ?3, working_dir = ?4,
            tags = ?5, notes = ?6, frequency = ?7, last_opened = ?8,
            updated_at = ?9, expires_at = ?10
        WHERE id = ?11
        "#,
        rusqlite::params![
            entry.lnk_path,
            entry.target_path,
            entry.parameters,
            entry.working_dir,
            entry.tags,
            entry.notes,
            entry.frequency,
            entry.last_opened,
            entry.updated_at,
            entry.expires_at,
            entry.id,
        ],
    )
    .context("Failed to update entry")?;

    Ok(())
}

/// Delete an entry by ID (legacy function)
pub fn delete_entry(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", rusqlite::params![id])
        .context("Failed to delete entry")?;

    Ok(())
}

/// Insert a new group
pub fn insert_group(conn: &Connection, group: &Group) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO groups (name, color, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        rusqlite::params![group.name, group.color, group.created_at, group.updated_at],
    )
    .context("Failed to insert group")?;

    Ok(conn.last_insert_rowid())
}

/// Get all groups
pub fn get_all_groups(conn: &Connection) -> Result<Vec<Group>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, name, color, created_at, updated_at FROM groups
            ORDER BY name
            "#,
        )
        .context("Failed to prepare groups query")?;

    let groups = stmt
        .query_map([], |row| {
            Ok(Group {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .context("Failed to map groups")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect groups")?;

    Ok(groups)
}

/// Get group by ID
pub fn get_group_by_id(conn: &Connection, id: i64) -> Result<Option<Group>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, name, color, created_at, updated_at FROM groups WHERE id = ?1
            "#,
        )
        .context("Failed to prepare group query")?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok(Group {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .optional()
        .context("Failed to query group")?;

    Ok(result)
}

/// Delete a group by ID
pub fn delete_group(conn: &Connection, id: i64) -> Result<bool> {
    let rows_affected = conn
        .execute("DELETE FROM groups WHERE id = ?1", rusqlite::params![id])
        .context("Failed to delete group")?;

    Ok(rows_affected > 0)
}