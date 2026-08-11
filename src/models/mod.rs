//! Data models for LNK File Management Center
//!
//! This module contains all data structures used throughout the application.

mod entry;
mod group;
mod config;

use std::collections::HashMap;

use rusqlite::Row;
use serde::{Deserialize, Serialize};

pub use entry::*;
pub use group::*;
pub use config::*;

/// Trait for converting database rows into model types
pub trait FromRow: Sized {
    /// Convert a database row into the implementing type
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self>;
}

/// Junction table entry for many-to-many relationship between entries and groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryGroup {
    /// Entry ID
    pub entry_id: i64,
    /// Group ID
    pub group_id: i64,
}

impl EntryGroup {
    /// Create a new entry-group association
    pub fn new(entry_id: i64, group_id: i64) -> Self {
        Self { entry_id, group_id }
    }
}

impl FromRow for EntryGroup {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(EntryGroup {
            entry_id: row.get(0)?,
            group_id: row.get(1)?,
        })
    }
}

/// Filter criteria for querying entries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntryFilter {
    /// Filter by tags (comma-separated, any match)
    pub tags: Option<String>,
    /// Minimum frequency threshold
    pub min_frequency: Option<i32>,
    /// Filter entries opened after this timestamp
    pub opened_after: Option<i64>,
    /// Filter entries created after this timestamp
    pub created_after: Option<i64>,
    /// Filter entries that expire before this timestamp
    pub expires_before: Option<i64>,
    /// Search query for FTS
    pub search_query: Option<String>,
    /// Maximum number of results
    pub limit: Option<i32>,
    /// Offset for pagination
    pub offset: Option<i32>,
}

impl EntryFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tag filter
    pub fn with_tags(mut self, tags: impl Into<String>) -> Self {
        self.tags = Some(tags.into());
        self
    }

    /// Set minimum frequency
    pub fn with_min_frequency(mut self, frequency: i32) -> Self {
        self.min_frequency = Some(frequency);
        self
    }

    /// Set pagination
    pub fn with_pagination(mut self, limit: i32, offset: i32) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// Usage statistics for entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryStats {
    /// Total number of entries
    pub total_entries: i64,
    /// Total number of groups
    pub total_groups: i64,
    /// Total number of opens across all entries
    pub total_opens: i64,
    /// Most frequently used entry
    pub most_used: Option<Entry>,
    /// Recently used entries
    pub recently_used: Vec<Entry>,
    /// Distribution of tags (tag -> count)
    pub tag_distribution: HashMap<String, i32>,
}

impl EntryStats {
    /// Create new stats with basic counts
    pub fn new(total_entries: i64, total_groups: i64, total_opens: i64) -> Self {
        Self {
            total_entries,
            total_groups,
            total_opens,
            most_used: None,
            recently_used: Vec::new(),
            tag_distribution: HashMap::new(),
        }
    }
}

/// Update specification for partial entry updates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntryUpdate {
    /// New target path
    pub target_path: Option<String>,
    /// New parameters
    pub parameters: Option<String>,
    /// New working directory
    pub working_dir: Option<String>,
    /// New tags
    pub tags: Option<String>,
    /// New notes
    pub notes: Option<String>,
    /// New group ID (None to unset)
    pub group_id: Option<Option<i64>>,
    /// New expiration timestamp (None to unset)
    pub expires_at: Option<Option<i64>>,
}

impl EntryUpdate {
    /// Create an empty update
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any updates are specified
    pub fn has_updates(&self) -> bool {
        self.target_path.is_some()
            || self.parameters.is_some()
            || self.working_dir.is_some()
            || self.tags.is_some()
            || self.notes.is_some()
            || self.group_id.is_some()
            || self.expires_at.is_some()
    }

    /// Set target path (builder pattern)
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target_path = Some(target.into());
        self
    }

    /// Set parameters (builder pattern)
    pub fn with_parameters(mut self, params: impl Into<String>) -> Self {
        self.parameters = Some(params.into());
        self
    }

    /// Set working directory (builder pattern)
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set tags (builder pattern)
    pub fn with_tags(mut self, tags: impl Into<String>) -> Self {
        self.tags = Some(tags.into());
        self
    }

    /// Set notes (builder pattern)
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set group ID (builder pattern)
    pub fn with_group(mut self, group_id: Option<i64>) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// Set expiration (builder pattern)
    pub fn with_expires_at(mut self, expires_at: Option<i64>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_group_new() {
        let entry_group = EntryGroup::new(1, 2);
        assert_eq!(entry_group.entry_id, 1);
        assert_eq!(entry_group.group_id, 2);
    }

    #[test]
    fn test_entry_stats_new() {
        let stats = EntryStats::new(10, 3, 50);
        assert_eq!(stats.total_entries, 10);
        assert_eq!(stats.total_groups, 3);
        assert_eq!(stats.total_opens, 50);
    }

    #[test]
    fn test_entry_update_builder() {
        let update = EntryUpdate::new()
            .with_target("new_target.exe")
            .with_notes("Updated notes");

        assert!(update.has_updates());
        assert_eq!(update.target_path, Some("new_target.exe".to_string()));
        assert_eq!(update.notes, Some("Updated notes".to_string()));
    }
}