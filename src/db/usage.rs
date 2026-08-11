//! Usage tracking operations
//!
//! Provides functions for tracking entry usage, including frequency counting,
//! last opened timestamps, and statistics queries.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

use crate::models::{Entry, EntryStats, FromRow};

/// Usage tracking operations
pub struct UsageOperations<'a> {
    conn: &'a Connection,
}

impl<'a> UsageOperations<'a> {
    /// Create a new UsageOperations instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Increment the frequency counter for an entry
    ///
    /// Also updates the last_opened timestamp.
    pub fn increment_frequency(&self, entry_id: i64) -> Result<bool> {
        let now = chrono::Utc::now().timestamp();
        let rows_affected = self
            .conn
            .execute(
                r#"
                UPDATE entries
                SET frequency = frequency + 1,
                    last_opened = ?1,
                    updated_at = ?1
                WHERE id = ?2
                "#,
                rusqlite::params![now, entry_id],
            )
            .context("Failed to increment frequency")?;

        Ok(rows_affected > 0)
    }

    /// Update the last opened timestamp for an entry
    ///
    /// This is called when an entry is opened without incrementing frequency.
    pub fn update_last_opened(&self, entry_id: i64) -> Result<bool> {
        let now = chrono::Utc::now().timestamp();
        let rows_affected = self
            .conn
            .execute(
                r#"
                UPDATE entries
                SET last_opened = ?1,
                    updated_at = ?1
                WHERE id = ?2
                "#,
                rusqlite::params![now, entry_id],
            )
            .context("Failed to update last opened")?;

        Ok(rows_affected > 0)
    }

    /// Get the most frequently used entries
    ///
    /// Returns top N entries sorted by frequency (descending).
    pub fn get_most_used(&self, limit: i32) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, lnk_path, target_path, parameters, working_dir,
                       tags, notes, frequency, last_opened,
                       created_at, updated_at, group_id, expires_at
                FROM entries
                ORDER BY frequency DESC, last_opened DESC
                LIMIT ?1
                "#,
            )
            .context("Failed to prepare most used query")?;

        let entries = stmt
            .query_map(rusqlite::params![limit], |row| Entry::from_row(row))
            .context("Failed to map most used entries")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect most used entries")?;

        Ok(entries)
    }

    /// Get the most recently used entries
    ///
    /// Returns top N entries sorted by last_opened (descending).
    pub fn get_recently_used(&self, limit: i32) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, lnk_path, target_path, parameters, working_dir,
                       tags, notes, frequency, last_opened,
                       created_at, updated_at, group_id, expires_at
                FROM entries
                WHERE last_opened IS NOT NULL
                ORDER BY last_opened DESC
                LIMIT ?1
                "#,
            )
            .context("Failed to prepare recently used query")?;

        let entries = stmt
            .query_map(rusqlite::params![limit], |row| Entry::from_row(row))
            .context("Failed to map recently used entries")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect recently used entries")?;

        Ok(entries)
    }

    /// Get usage statistics for all entries
    ///
    /// Returns comprehensive statistics including totals, averages,
    /// most used entry, recently used entries, and tag distribution.
    pub fn get_stats(&self) -> Result<EntryStats> {
        // Get basic counts
        let total_entries: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .context("Failed to count entries")?;

        let total_groups: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM groups", [], |row| row.get(0))
            .context("Failed to count groups")?;

        let total_opens: i64 = self
            .conn
            .query_row("SELECT COALESCE(SUM(frequency), 0) FROM entries", [], |row| row.get(0))
            .context("Failed to sum frequency")?;

        let mut stats = EntryStats::new(total_entries, total_groups, total_opens);

        // Get most used entry
        let most_used = self.get_most_used(1)?;
        stats.most_used = most_used.into_iter().next();

        // Get recently used entries
        stats.recently_used = self.get_recently_used(10)?;

        // Get tag distribution
        stats.tag_distribution = self.get_tag_distribution()?;

        Ok(stats)
    }

    /// Get tag distribution across all entries
    ///
    /// Returns a map of tag to count.
    fn get_tag_distribution(&self) -> Result<std::collections::HashMap<String, i32>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tags FROM entries WHERE tags IS NOT NULL AND tags != ''")
            .context("Failed to prepare tag distribution query")?;

        let tag_strings = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .context("Failed to query tag distribution")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect tag distribution")?;

        let mut distribution: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

        for tag_string in tag_strings {
            for tag in tag_string.split(',') {
                let tag = tag.trim().to_lowercase();
                if !tag.is_empty() {
                    *distribution.entry(tag).or_insert(0) += 1;
                }
            }
        }

        Ok(distribution)
    }

    /// Reset frequency for an entry
    pub fn reset_frequency(&self, entry_id: i64) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute(
                "UPDATE entries SET frequency = 0, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![chrono::Utc::now().timestamp(), entry_id],
            )
            .context("Failed to reset frequency")?;

        Ok(rows_affected > 0)
    }

    /// Reset last opened timestamp for an entry
    pub fn reset_last_opened(&self, entry_id: i64) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute(
                "UPDATE entries SET last_opened = NULL, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![chrono::Utc::now().timestamp(), entry_id],
            )
            .context("Failed to reset last opened")?;

        Ok(rows_affected > 0)
    }

    /// Reset all usage data for an entry
    pub fn reset_usage(&self, entry_id: i64) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute(
                "UPDATE entries SET frequency = 0, last_opened = NULL, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![chrono::Utc::now().timestamp(), entry_id],
            )
            .context("Failed to reset usage")?;

        Ok(rows_affected > 0)
    }

    /// Get entries not used in the last N days
    pub fn get_unused_entries(&self, days: i32) -> Result<Vec<Entry>> {
        let cutoff = chrono::Utc::now().timestamp() - (days as i64 * 86400);

        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, lnk_path, target_path, parameters, working_dir,
                       tags, notes, frequency, last_opened,
                       created_at, updated_at, group_id, expires_at
                FROM entries
                WHERE last_opened IS NULL OR last_opened < ?1
                ORDER BY frequency ASC, created_at DESC
                "#,
            )
            .context("Failed to prepare unused entries query")?;

        let entries = stmt
            .query_map(rusqlite::params![cutoff], |row| Entry::from_row(row))
            .context("Failed to map unused entries")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect unused entries")?;

        Ok(entries)
    }

    /// Get usage summary for a specific entry
    pub fn get_entry_usage(&self, entry_id: i64) -> Result<Option<EntryUsage>> {
        let result = self
            .conn
            .query_row(
                r#"
                SELECT frequency, last_opened, created_at
                FROM entries WHERE id = ?1
                "#,
                rusqlite::params![entry_id],
                |row| {
                    Ok(EntryUsage {
                        entry_id,
                        frequency: row.get(0)?,
                        last_opened: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .optional()
            .context("Failed to get entry usage")?;

        Ok(result)
    }
}

/// Usage summary for a single entry
#[derive(Debug, Clone)]
pub struct EntryUsage {
    /// Entry ID
    pub entry_id: i64,
    /// Number of times opened
    pub frequency: i32,
    /// Last opened timestamp
    pub last_opened: Option<i64>,
    /// Creation timestamp
    pub created_at: i64,
}

impl EntryUsage {
    /// Calculate average uses per day
    pub fn uses_per_day(&self) -> f64 {
        let now = chrono::Utc::now().timestamp();
        let age_days = ((now - self.created_at) as f64 / 86400.0).max(1.0);
        self.frequency as f64 / age_days
    }
}