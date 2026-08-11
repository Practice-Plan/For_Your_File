//! Batch operations for entries
//!
//! Provides efficient bulk operations for creating, updating, and deleting
//! multiple entries at once.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

use crate::models::{Entry, EntryUpdate};

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub failed_count: usize,
    /// IDs of successfully processed items
    pub successful_ids: Vec<i64>,
    /// Error messages for failed operations
    pub errors: Vec<String>,
}

impl BatchResult {
    /// Create a new empty batch result
    pub fn new() -> Self {
        Self {
            success_count: 0,
            failed_count: 0,
            successful_ids: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if all operations succeeded
    pub fn is_complete_success(&self) -> bool {
        self.failed_count == 0
    }

    /// Check if all operations failed
    pub fn is_complete_failure(&self) -> bool {
        self.success_count == 0 && self.failed_count > 0
    }
}

impl Default for BatchResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch operations for entries
pub struct BatchOperations<'a> {
    conn: &'a Connection,
}

impl<'a> BatchOperations<'a> {
    /// Create a new BatchOperations instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create multiple entries in a single transaction
    ///
    /// This is more efficient than creating entries one by one as it
    /// uses a single transaction for all insertions.
    pub fn batch_create(&self, entries: &[Entry]) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        for entry in entries {
            match self.create_entry_internal(entry) {
                Ok(id) => {
                    result.success_count += 1;
                    result.successful_ids.push(id);
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!(
                        "Failed to create entry '{}': {}",
                        entry.lnk_path, e
                    ));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Delete multiple entries in a single transaction
    ///
    /// Returns the result of the batch deletion including which entries
    /// were successfully deleted.
    pub fn batch_delete(&self, ids: &[i64]) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        for id in ids {
            match self.delete_entry_internal(*id) {
                Ok(deleted) => {
                    if deleted {
                        result.success_count += 1;
                        result.successful_ids.push(*id);
                    } else {
                        result.failed_count += 1;
                        result.errors.push(format!("Entry {} not found", id));
                    }
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!("Failed to delete entry {}: {}", id, e));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Update multiple entries in a single transaction
    ///
    /// Uses partial updates to only modify specified fields.
    pub fn batch_update(&self, updates: &[(i64, EntryUpdate)]) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        for (id, update) in updates {
            match self.update_entry_internal(*id, update) {
                Ok(updated) => {
                    if updated {
                        result.success_count += 1;
                        result.successful_ids.push(*id);
                    } else {
                        result.failed_count += 1;
                        result.errors.push(format!("Entry {} not found", id));
                    }
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!("Failed to update entry {}: {}", id, e));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Add tags to multiple entries
    pub fn batch_add_tags(&self, ids: &[i64], tags: &[String]) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        let tags_str = tags
            .iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(", ");

        for id in ids {
            match self.add_tags_internal(*id, &tags_str) {
                Ok(_) => {
                    result.success_count += 1;
                    result.successful_ids.push(*id);
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!("Failed to add tags to entry {}: {}", id, e));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Move multiple entries to a group
    pub fn batch_move_to_group(&self, ids: &[i64], group_id: Option<i64>) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        let now = chrono::Utc::now().timestamp();

        for id in ids {
            match self.conn.execute(
                "UPDATE entries SET group_id = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![group_id, now, id],
            ) {
                Ok(rows) => {
                    if rows > 0 {
                        result.success_count += 1;
                        result.successful_ids.push(*id);
                    } else {
                        result.failed_count += 1;
                        result.errors.push(format!("Entry {} not found", id));
                    }
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!("Failed to move entry {}: {}", id, e));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Increment frequency for multiple entries
    pub fn batch_increment_frequency(&self, ids: &[i64]) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        let now = chrono::Utc::now().timestamp();

        for id in ids {
            match self.conn.execute(
                r#"
                UPDATE entries
                SET frequency = frequency + 1,
                    last_opened = ?1,
                    updated_at = ?1
                WHERE id = ?2
                "#,
                rusqlite::params![now, id],
            ) {
                Ok(rows) => {
                    if rows > 0 {
                        result.success_count += 1;
                        result.successful_ids.push(*id);
                    } else {
                        result.failed_count += 1;
                        result.errors.push(format!("Entry {} not found", id));
                    }
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!("Failed to increment entry {}: {}", id, e));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Add multiple entries to a group
    ///
    /// Creates associations between entries and a group.
    /// Uses INSERT OR IGNORE to handle duplicates gracefully.
    pub fn batch_add_to_group(&self, entry_ids: &[i64], group_id: i64) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        for entry_id in entry_ids {
            match self.conn.execute(
                "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
                rusqlite::params![entry_id, group_id],
            ) {
                Ok(rows) => {
                    if rows > 0 {
                        result.success_count += 1;
                        result.successful_ids.push(*entry_id);
                    } else {
                        // Already in group
                        result.success_count += 1;
                        result.successful_ids.push(*entry_id);
                    }
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!(
                        "Failed to add entry {} to group {}: {}",
                        entry_id, group_id, e
                    ));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Remove multiple entries from a group
    ///
    /// Removes associations between entries and a group.
    pub fn batch_remove_from_group(&self, entry_ids: &[i64], group_id: i64) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        for entry_id in entry_ids {
            match self.conn.execute(
                "DELETE FROM entry_groups WHERE entry_id = ?1 AND group_id = ?2",
                rusqlite::params![entry_id, group_id],
            ) {
                Ok(rows) => {
                    if rows > 0 {
                        result.success_count += 1;
                        result.successful_ids.push(*entry_id);
                    } else {
                        result.failed_count += 1;
                        result.errors.push(format!(
                            "Entry {} was not in group {}",
                            entry_id, group_id
                        ));
                    }
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!(
                        "Failed to remove entry {} from group {}: {}",
                        entry_id, group_id, e
                    ));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Remove entries from all groups
    ///
    /// Removes all group associations for the specified entries.
    pub fn batch_remove_from_all_groups(&self, entry_ids: &[i64]) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        for entry_id in entry_ids {
            match self.conn.execute(
                "DELETE FROM entry_groups WHERE entry_id = ?1",
                rusqlite::params![entry_id],
            ) {
                Ok(_) => {
                    result.success_count += 1;
                    result.successful_ids.push(*entry_id);
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!(
                        "Failed to remove entry {} from all groups: {}",
                        entry_id, e
                    ));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    /// Move entries from one group to another
    ///
    /// Removes from source group and adds to target group.
    pub fn batch_move_between_groups(
        &self,
        entry_ids: &[i64],
        from_group_id: i64,
        to_group_id: i64,
    ) -> Result<BatchResult> {
        let mut result = BatchResult::new();

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction")?;

        for entry_id in entry_ids {
            // Remove from old group
            self.conn.execute(
                "DELETE FROM entry_groups WHERE entry_id = ?1 AND group_id = ?2",
                rusqlite::params![entry_id, from_group_id],
            )?;

            // Add to new group
            match self.conn.execute(
                "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
                rusqlite::params![entry_id, to_group_id],
            ) {
                Ok(_) => {
                    result.success_count += 1;
                    result.successful_ids.push(*entry_id);
                }
                Err(e) => {
                    result.failed_count += 1;
                    result.errors.push(format!(
                        "Failed to move entry {} from group {} to {}: {}",
                        entry_id, from_group_id, to_group_id, e
                    ));
                }
            }
        }

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }

    // Internal helper methods

    fn create_entry_internal(&self, entry: &Entry) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO entries (lnk_path, target_path, parameters, working_dir, tags, notes,
                                 frequency, last_opened, created_at, updated_at, group_id, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
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
                entry.group_id,
                entry.expires_at,
            ],
        )
        .context("Failed to insert entry")?;

        Ok(self.conn.last_insert_rowid())
    }

    fn delete_entry_internal(&self, id: i64) -> Result<bool> {
        // Check if entry exists and get lnk_path
        let lnk_path: Option<String> = self
            .conn
            .query_row("SELECT lnk_path FROM entries WHERE id = ?1", rusqlite::params![id], |row| {
                row.get(0)
            })
            .optional()?;

        if let Some(path) = lnk_path {
            // Delete .lnk file if it exists
            if std::path::Path::new(&path).exists() {
                std::fs::remove_file(&path).ok(); // Ignore errors for file deletion
            }

            // Delete from database
            self.conn
                .execute("DELETE FROM entries WHERE id = ?1", rusqlite::params![id])
                .context("Failed to delete entry")?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn update_entry_internal(&self, id: i64, update: &EntryUpdate) -> Result<bool> {
        if !update.has_updates() {
            return Ok(true);
        }

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

        updates.push("updated_at = ?".to_string());
        params.push(Box::new(chrono::Utc::now().timestamp()));

        params.push(Box::new(id));

        let sql = format!("UPDATE entries SET {} WHERE id = ?", updates.join(", "));
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = self
            .conn
            .execute(&sql, params_refs.as_slice())
            .context("Failed to update entry")?;

        Ok(rows > 0)
    }

    fn add_tags_internal(&self, id: i64, new_tags: &str) -> Result<()> {
        let current_tags: Option<String> = self
            .conn
            .query_row("SELECT tags FROM entries WHERE id = ?1", rusqlite::params![id], |row| {
                row.get(0)
            })
            .optional()?
            .flatten();

        let mut all_tags = current_tags
            .map(|s| {
                s.split(',')
                    .map(|t| t.trim().to_lowercase())
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for tag in new_tags.split(',') {
            let tag = tag.trim().to_lowercase();
            if !tag.is_empty() && !all_tags.contains(&tag) {
                all_tags.push(tag);
            }
        }

        let combined_tags = all_tags.join(", ");
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "UPDATE entries SET tags = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![combined_tags, now, id],
        )
        .context("Failed to add tags")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_result() {
        let mut result = BatchResult::new();
        assert!(result.is_complete_success());
        assert!(!result.is_complete_failure());

        result.failed_count = 1;
        assert!(!result.is_complete_success());
        assert!(!result.is_complete_failure());

        result.success_count = 0;
        assert!(result.is_complete_failure());
    }
}