//! Group management operations
//!
//! Provides functions for managing groups and entry-group associations.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::models::{Entry, FromRow, Group};

/// Group with entry count for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithCount {
    /// Group data
    #[serde(flatten)]
    pub group: Group,
    /// Number of entries in this group
    pub entry_count: i64,
}

impl GroupWithCount {
    /// Create a new GroupWithCount
    pub fn new(group: Group, entry_count: i64) -> Self {
        Self { group, entry_count }
    }
}

/// Export format for groups (includes entries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupExport {
    /// Group metadata
    pub group: Group,
    /// Entry IDs in this group
    pub entry_ids: Vec<i64>,
    /// Export timestamp
    pub exported_at: i64,
}

impl GroupExport {
    /// Create a new export
    pub fn new(group: Group, entry_ids: Vec<i64>) -> Self {
        Self {
            group,
            entry_ids,
            exported_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Operations for managing groups
pub struct GroupOperations<'a> {
    conn: &'a Connection,
}

impl<'a> GroupOperations<'a> {
    /// Create a new GroupOperations instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new group
    ///
    /// Returns the created group with ID populated.
    pub fn create_group(&self, name: &str, color: &str) -> Result<Group> {
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT INTO groups (name, color, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![name, color, now, now],
        )
        .context("Failed to create group")?;

        let id = self.conn.last_insert_rowid();

        Ok(Group {
            id: Some(id),
            name: name.to_string(),
            color: color.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Get a group by ID
    ///
    /// Returns the group with entry count.
    pub fn get_group(&self, id: i64) -> Result<Option<GroupWithCount>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT g.id, g.name, g.color, g.created_at, g.updated_at,
                       COUNT(eg.entry_id) as entry_count
                FROM groups g
                LEFT JOIN entry_groups eg ON g.id = eg.group_id
                WHERE g.id = ?1
                GROUP BY g.id
                "#,
            )
            .context("Failed to prepare get_group statement")?;

        let result = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(GroupWithCount {
                    group: Group {
                        id: Some(row.get(0)?),
                        name: row.get(1)?,
                        color: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    },
                    entry_count: row.get(5)?,
                })
            })
            .optional()
            .context("Failed to query group")?;

        Ok(result)
    }

    /// List all groups with metadata
    ///
    /// Returns groups sorted by name, each with entry count.
    pub fn list_groups(&self) -> Result<Vec<GroupWithCount>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT g.id, g.name, g.color, g.created_at, g.updated_at,
                       COUNT(eg.entry_id) as entry_count
                FROM groups g
                LEFT JOIN entry_groups eg ON g.id = eg.group_id
                GROUP BY g.id
                ORDER BY g.name ASC
                "#,
            )
            .context("Failed to prepare list_groups statement")?;

        let groups = stmt
            .query_map([], |row| {
                Ok(GroupWithCount {
                    group: Group {
                        id: Some(row.get(0)?),
                        name: row.get(1)?,
                        color: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    },
                    entry_count: row.get(5)?,
                })
            })
            .context("Failed to map groups")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect groups")?;

        Ok(groups)
    }

    /// Update group name and/or color
    ///
    /// Returns the updated group, or None if not found.
    pub fn update_group(&self, id: i64, name: Option<&str>, color: Option<&str>) -> Result<Option<Group>> {
        // Check if group exists
        let existing = self.get_group(id)?;
        if existing.is_none() {
            return Ok(None);
        }

        let now = chrono::Utc::now().timestamp();

        // Build dynamic update
        let mut updates: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(n) = name {
            updates.push("name = ?".to_string());
            params.push(Box::new(n.to_string()));
        }
        if let Some(c) = color {
            updates.push("color = ?".to_string());
            params.push(Box::new(c.to_string()));
        }

        if updates.is_empty() {
            return Ok(existing.map(|g| g.group));
        }

        updates.push("updated_at = ?".to_string());
        params.push(Box::new(now));

        params.push(Box::new(id));

        let sql = format!("UPDATE groups SET {} WHERE id = ?", updates.join(", "));
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        self.conn
            .execute(&sql, params_refs.as_slice())
            .context("Failed to update group")?;

        // Return updated group
        self.get_group(id).map(|opt| opt.map(|g| g.group))
    }

    /// Delete a group by ID
    ///
    /// This removes the group and all entry-group associations,
    /// but keeps the entries themselves intact.
    /// Returns true if the group was deleted, false if not found.
    pub fn delete_group(&self, id: i64) -> Result<bool> {
        // Delete entry-group associations first (should cascade, but be explicit)
        self.conn
            .execute("DELETE FROM entry_groups WHERE group_id = ?1", rusqlite::params![id])
            .context("Failed to delete entry-group associations")?;

        // Delete the group
        let rows_affected = self
            .conn
            .execute("DELETE FROM groups WHERE id = ?1", rusqlite::params![id])
            .context("Failed to delete group")?;

        Ok(rows_affected > 0)
    }

    /// Add an entry to a group
    ///
    /// Creates an association between the entry and group.
    /// Returns true if the association was created, false if already exists.
    pub fn add_entry_to_group(&self, entry_id: i64, group_id: i64) -> Result<bool> {
        // Use INSERT OR IGNORE to handle duplicates gracefully
        self.conn
            .execute(
                "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
                rusqlite::params![entry_id, group_id],
            )
            .context("Failed to add entry to group")?;

        // Check if a row was actually inserted
        let changes = self.conn.execute("SELECT changes()", [])?;
        Ok(changes > 0)
    }

    /// Remove an entry from a group
    ///
    /// Removes the association between entry and group.
    /// Returns true if the association was removed, false if not found.
    pub fn remove_entry_from_group(&self, entry_id: i64, group_id: i64) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute(
                "DELETE FROM entry_groups WHERE entry_id = ?1 AND group_id = ?2",
                rusqlite::params![entry_id, group_id],
            )
            .context("Failed to remove entry from group")?;

        Ok(rows_affected > 0)
    }

    /// Get all entries in a group
    ///
    /// Returns entries sorted by frequency (most used first).
    pub fn get_group_entries(&self, group_id: i64) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
                       e.tags, e.notes, e.frequency, e.last_opened,
                       e.created_at, e.updated_at, e.expires_at
                FROM entries e
                INNER JOIN entry_groups eg ON e.id = eg.entry_id
                WHERE eg.group_id = ?1
                ORDER BY e.frequency DESC, e.updated_at DESC
                "#,
            )
            .context("Failed to prepare get_group_entries statement")?;

        let entries = stmt
            .query_map(rusqlite::params![group_id], |row| Entry::from_row(row))
            .context("Failed to map entries")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect entries")?;

        Ok(entries)
    }

    /// Get all groups for an entry
    ///
    /// Returns groups the entry belongs to.
    pub fn get_entry_groups(&self, entry_id: i64) -> Result<Vec<Group>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT g.id, g.name, g.color, g.created_at, g.updated_at
                FROM groups g
                INNER JOIN entry_groups eg ON g.id = eg.group_id
                WHERE eg.entry_id = ?1
                ORDER BY g.name ASC
                "#,
            )
            .context("Failed to prepare get_entry_groups statement")?;

        let groups = stmt
            .query_map(rusqlite::params![entry_id], |row| {
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

    /// Export a group as JSON
    ///
    /// Creates an export with group metadata and entry IDs.
    pub fn export_group(&self, group_id: i64) -> Result<Option<GroupExport>> {
        let group_with_count = self.get_group(group_id)?;

        if let Some(gwc) = group_with_count {
            let entries = self.get_group_entries(group_id)?;
            let entry_ids: Vec<i64> = entries.iter().filter_map(|e| e.id).collect();

            Ok(Some(GroupExport::new(gwc.group, entry_ids)))
        } else {
            Ok(None)
        }
    }

    /// Import a group from JSON export
    ///
    /// Creates a new group and optionally associates entries.
    /// If entries don't exist, they are skipped.
    /// Returns the created group with ID.
    pub fn import_group(
        &self,
        name: &str,
        color: &str,
        entry_ids: &[i64],
    ) -> Result<Group> {
        // Create the group
        let group = self.create_group(name, color)?;

        // Associate entries (skip non-existent)
        if let Some(group_id) = group.id {
            for entry_id in entry_ids {
                // Check if entry exists
                let exists: bool = self
                    .conn
                    .query_row(
                        "SELECT 1 FROM entries WHERE id = ?1",
                        rusqlite::params![entry_id],
                        |_| Ok(true),
                    )
                    .optional()?
                    .is_some();

                if exists {
                    self.add_entry_to_group(*entry_id, group_id)?;
                }
            }
        }

        Ok(group)
    }

    /// Remove all entries from a group
    ///
    /// Removes all associations but keeps the group and entries.
    /// Returns the number of associations removed.
    pub fn clear_group(&self, group_id: i64) -> Result<usize> {
        let rows_affected = self
            .conn
            .execute(
                "DELETE FROM entry_groups WHERE group_id = ?1",
                rusqlite::params![group_id],
            )
            .context("Failed to clear group")?;

        Ok(rows_affected)
    }

    /// Rename a group
    ///
    /// Convenience method for updating just the name.
    pub fn rename_group(&self, id: i64, new_name: &str) -> Result<Option<Group>> {
        self.update_group(id, Some(new_name), None)
    }

    /// Change group color
    ///
    /// Convenience method for updating just the color.
    pub fn set_group_color(&self, id: i64, color: &str) -> Result<Option<Group>> {
        self.update_group(id, None, Some(color))
    }

    /// Check if a group name already exists
    ///
    /// Case-insensitive check for duplicate names.
    pub fn group_name_exists(&self, name: &str, exclude_id: Option<i64>) -> Result<bool> {
        let exists: bool = match exclude_id {
            Some(id) => {
                self.conn
                    .query_row(
                        "SELECT 1 FROM groups WHERE LOWER(name) = LOWER(?1) AND id != ?2",
                        rusqlite::params![name, id],
                        |_| Ok(true),
                    )
                    .optional()?
                    .is_some()
            }
            None => {
                self.conn
                    .query_row(
                        "SELECT 1 FROM groups WHERE LOWER(name) = LOWER(?1)",
                        rusqlite::params![name],
                        |_| Ok(true),
                    )
                    .optional()?
                    .is_some()
            }
        };

        Ok(exists)
    }

    /// Get groups with their entries (for batch operations)
    ///
    /// Returns groups with entry counts, optionally filtered.
    pub fn get_groups_summary(&self) -> Result<Vec<(Group, i64)>> {
        let groups = self.list_groups()?;
        Ok(groups.into_iter().map(|g| (g.group, g.entry_count)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create tables
        conn.execute_batch(
            r#"
            CREATE TABLE entries (
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
            );
            CREATE TABLE groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#3498db',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE entry_groups (
                entry_id INTEGER NOT NULL,
                group_id INTEGER NOT NULL,
                PRIMARY KEY (entry_id, group_id),
                FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
                FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
            );
            "#,
        )
        .unwrap();

        conn
    }

    fn create_test_entry(conn: &Connection, lnk_path: &str) -> i64 {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![lnk_path, "C:\\test.exe", now, now],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn test_create_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let group = ops.create_group("Work", "#FF5733").unwrap();
        assert!(group.id.is_some());
        assert_eq!(group.name, "Work");
        assert_eq!(group.color, "#FF5733");
    }

    #[test]
    fn test_get_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let created = ops.create_group("Personal", "#00FF00").unwrap();
        let id = created.id.unwrap();

        let fetched = ops.get_group(id).unwrap().unwrap();
        assert_eq!(fetched.group.name, "Personal");
        assert_eq!(fetched.group.color, "#00FF00");
        assert_eq!(fetched.entry_count, 0);
    }

    #[test]
    fn test_list_groups() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        ops.create_group("B Group", "#111111").unwrap();
        ops.create_group("A Group", "#222222").unwrap();

        let groups = ops.list_groups().unwrap();
        assert_eq!(groups.len(), 2);
        // Should be sorted by name
        assert_eq!(groups[0].group.name, "A Group");
        assert_eq!(groups[1].group.name, "B Group");
    }

    #[test]
    fn test_update_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let created = ops.create_group("Old Name", "#000000").unwrap();
        let id = created.id.unwrap();

        let updated = ops.update_group(id, Some("New Name"), Some("#FFFFFF")).unwrap().unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.color, "#FFFFFF");
    }

    #[test]
    fn test_delete_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let created = ops.create_group("To Delete", "#000000").unwrap();
        let id = created.id.unwrap();

        let deleted = ops.delete_group(id).unwrap();
        assert!(deleted);

        let fetched = ops.get_group(id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_add_entry_to_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let entry_id = create_test_entry(&conn, "C:\\test.lnk");
        let group = ops.create_group("Test Group", "#3498db").unwrap();
        let group_id = group.id.unwrap();

        let added = ops.add_entry_to_group(entry_id, group_id).unwrap();
        assert!(added);

        let entries = ops.get_group_entries(group_id).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, Some(entry_id));
    }

    #[test]
    fn test_remove_entry_from_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let entry_id = create_test_entry(&conn, "C:\\test.lnk");
        let group = ops.create_group("Test Group", "#3498db").unwrap();
        let group_id = group.id.unwrap();

        ops.add_entry_to_group(entry_id, group_id).unwrap();

        let removed = ops.remove_entry_from_group(entry_id, group_id).unwrap();
        assert!(removed);

        let entries = ops.get_group_entries(group_id).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_entry_groups() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let entry_id = create_test_entry(&conn, "C:\\test.lnk");
        let group1 = ops.create_group("Group 1", "#111111").unwrap();
        let group2 = ops.create_group("Group 2", "#222222").unwrap();

        ops.add_entry_to_group(entry_id, group1.id.unwrap()).unwrap();
        ops.add_entry_to_group(entry_id, group2.id.unwrap()).unwrap();

        let groups = ops.get_entry_groups(entry_id).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_export_import_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let entry_id = create_test_entry(&conn, "C:\\test.lnk");
        let group = ops.create_group("Export Test", "#123456").unwrap();
        let group_id = group.id.unwrap();

        ops.add_entry_to_group(entry_id, group_id).unwrap();

        // Export
        let export = ops.export_group(group_id).unwrap().unwrap();
        assert_eq!(export.group.name, "Export Test");
        assert_eq!(export.entry_ids.len(), 1);

        // Delete original
        ops.delete_group(group_id).unwrap();

        // Import
        let imported = ops.import_group(&export.group.name, &export.group.color, &export.entry_ids).unwrap();
        assert!(imported.id.is_some());

        // Verify
        let imported_entries = ops.get_group_entries(imported.id.unwrap()).unwrap();
        assert_eq!(imported_entries.len(), 1);
    }

    #[test]
    fn test_group_name_exists() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        ops.create_group("Existing", "#000000").unwrap();

        assert!(ops.group_name_exists("Existing", None).unwrap());
        assert!(ops.group_name_exists("EXISTING", None).unwrap()); // Case insensitive
        assert!(!ops.group_name_exists("Not Existing", None).unwrap());
    }

    #[test]
    fn test_clear_group() {
        let conn = setup_test_db();
        let ops = GroupOperations::new(&conn);

        let entry1 = create_test_entry(&conn, "C:\\test1.lnk");
        let entry2 = create_test_entry(&conn, "C:\\test2.lnk");
        let group = ops.create_group("Test", "#000000").unwrap();
        let group_id = group.id.unwrap();

        ops.add_entry_to_group(entry1, group_id).unwrap();
        ops.add_entry_to_group(entry2, group_id).unwrap();

        let cleared = ops.clear_group(group_id).unwrap();
        assert_eq!(cleared, 2);

        let entries = ops.get_group_entries(group_id).unwrap();
        assert!(entries.is_empty());
    }
}