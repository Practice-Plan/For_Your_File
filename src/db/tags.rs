//! Tag management operations
//!
//! Provides functions for managing tags on entries, including
//! normalization, validation, and bulk operations.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

/// Tag operations for managing entry tags
pub struct TagOperations<'a> {
    conn: &'a Connection,
}

impl<'a> TagOperations<'a> {
    /// Create a new TagOperations instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Add tags to an entry
    ///
    /// Tags are normalized (lowercase, trimmed) and appended to existing tags.
    /// Duplicate tags are automatically removed.
    pub fn add_tags(&self, entry_id: i64, tags: &[String]) -> Result<bool> {
        if tags.is_empty() {
            return Ok(false);
        }

        // Get current entry
        let current_tags: Option<String> = self
            .conn
            .query_row(
                "SELECT tags FROM entries WHERE id = ?1",
                rusqlite::params![entry_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        // Normalize and combine tags
        let mut all_tags = self.parse_tags(current_tags);
        for tag in tags {
            let normalized = self.normalize_tag(tag);
            if !normalized.is_empty() && !all_tags.contains(&normalized) {
                all_tags.push(normalized);
            }
        }

        // Update entry
        let new_tags = all_tags.join(", ");
        let rows_affected = self.conn.execute(
            "UPDATE entries SET tags = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![new_tags, chrono::Utc::now().timestamp(), entry_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// Remove tags from an entry
    ///
    /// Tags are normalized before matching.
    pub fn remove_tags(&self, entry_id: i64, tags: &[String]) -> Result<bool> {
        if tags.is_empty() {
            return Ok(false);
        }

        // Get current entry
        let current_tags: Option<String> = self
            .conn
            .query_row(
                "SELECT tags FROM entries WHERE id = ?1",
                rusqlite::params![entry_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        let mut all_tags = self.parse_tags(current_tags);

        // Remove specified tags
        let tags_to_remove: Vec<String> = tags.iter().map(|t| self.normalize_tag(t)).collect();
        all_tags.retain(|t| !tags_to_remove.contains(t));

        // Update entry
        let new_tags = if all_tags.is_empty() {
            None
        } else {
            Some(all_tags.join(", "))
        };

        let rows_affected = self.conn.execute(
            "UPDATE entries SET tags = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![new_tags, chrono::Utc::now().timestamp(), entry_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// Get all unique tags used in the database
    ///
    /// Returns a sorted list of all tags with their usage counts.
    pub fn get_all_tags(&self) -> Result<Vec<(String, i32)>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT tags FROM entries WHERE tags IS NOT NULL AND tags != ''
                "#,
            )
            .context("Failed to prepare tags query")?;

        let tag_strings = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .context("Failed to query tags")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect tags")?;

        // Count tag occurrences
        let mut tag_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for tag_string in tag_strings {
            let tags = self.parse_tags(Some(tag_string));
            for tag in tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }

        // Sort by count descending, then alphabetically
        let mut result: Vec<(String, i32)> = tag_counts.into_iter().collect();
        result.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
        });

        Ok(result)
    }

    /// Validate tags
    ///
    /// Returns a list of validation errors for each invalid tag.
    /// Valid tags are returned in the first element.
    pub fn validate_tags(&self, tags: &[String]) -> (Vec<String>, Vec<String>) {
        let mut valid = Vec::new();
        let mut invalid = Vec::new();

        for tag in tags {
            let normalized = self.normalize_tag(tag);
            if normalized.is_empty() {
                if !tag.trim().is_empty() {
                    invalid.push(format!("Tag '{}' is empty after normalization", tag));
                }
            } else if normalized.len() > 50 {
                invalid.push(format!("Tag '{}' exceeds 50 character limit", tag));
            } else if normalized.chars().any(|c| c == ',' || c == ';') {
                invalid.push(format!("Tag '{}' contains invalid characters", tag));
            } else {
                valid.push(normalized);
            }
        }

        (valid, invalid)
    }

    /// Parse a comma-separated tag string into a vector of tags
    fn parse_tags(&self, tags: Option<String>) -> Vec<String> {
        tags
            .map(|s| {
                s.split(',')
                    .map(|t| t.trim().to_lowercase())
                    .filter(|t| !t.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Normalize a single tag (lowercase, trim)
    fn normalize_tag(&self, tag: &str) -> String {
        tag.trim().to_lowercase()
    }

    /// Get entries by tag
    pub fn get_entries_by_tag(&self, tag: &str) -> Result<Vec<i64>> {
        let normalized = self.normalize_tag(tag);
        let pattern = format!("%{}%", normalized);

        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id FROM entries WHERE tags LIKE ?1 ORDER BY frequency DESC
                "#,
            )
            .context("Failed to prepare entries by tag query")?;

        let entries = stmt
            .query_map(rusqlite::params![pattern], |row| row.get(0))
            .context("Failed to query entries by tag")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect entries by tag")?;

        Ok(entries)
    }

    /// Rename a tag across all entries
    pub fn rename_tag(&self, old_tag: &str, new_tag: &str) -> Result<i32> {
        let old_normalized = self.normalize_tag(old_tag);
        let new_normalized = self.normalize_tag(new_tag);

        if old_normalized == new_normalized {
            return Ok(0);
        }

        // Get all entries with this tag
        let entries = self.get_entries_by_tag(&old_normalized)?;
        let entry_count = entries.len();

        for entry_id in &entries {
            // Get current tags
            let current_tags: Option<String> = self
                .conn
                .query_row(
                    "SELECT tags FROM entries WHERE id = ?1",
                    rusqlite::params![entry_id],
                    |row| row.get(0),
                )
                .optional()?
                .flatten();

            let mut all_tags = self.parse_tags(current_tags);

            // Replace the tag
            if let Some(pos) = all_tags.iter().position(|t| t == &old_normalized) {
                all_tags[pos] = new_normalized.clone();
            }

            // Update entry
            let new_tags = all_tags.join(", ");
            self.conn.execute(
                "UPDATE entries SET tags = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![new_tags, chrono::Utc::now().timestamp(), entry_id],
            )?;
        }

        Ok(entry_count as i32)
    }

    /// Delete a tag from all entries
    pub fn delete_tag_from_all(&self, tag: &str) -> Result<i32> {
        let normalized = self.normalize_tag(tag);

        // Get all entries with this tag
        let entries = self.get_entries_by_tag(&normalized)?;
        let entry_count = entries.len();

        for entry_id in &entries {
            self.remove_tags(*entry_id, &[normalized.clone()])?;
        }

        Ok(entry_count as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tag() {
        let conn = Connection::open_in_memory().unwrap();
        let ops = TagOperations::new(&conn);
        assert_eq!(ops.normalize_tag("  Test Tag  "), "test tag");
        assert_eq!(ops.normalize_tag("UPPERCASE"), "uppercase");
        assert_eq!(ops.normalize_tag(""), "");
        assert_eq!(ops.normalize_tag("   "), "");
    }

    #[test]
    fn test_parse_tags() {
        let conn = Connection::open_in_memory().unwrap();
        let ops = TagOperations::new(&conn);
        let tags = ops.parse_tags(Some("tag1, Tag2, TAG3".to_string()));
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);

        let tags = ops.parse_tags(None);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_validate_tags() {
        let conn = Connection::open_in_memory().unwrap();
        let ops = TagOperations::new(&conn);

        let (valid, invalid) = ops.validate_tags(&["valid".to_string(), "Also Valid".to_string()]);
        assert_eq!(valid.len(), 2);
        assert!(invalid.is_empty());

        let (valid, invalid) = ops.validate_tags(&["test,tag".to_string()]);
        assert!(valid.is_empty());
        assert!(!invalid.is_empty());
    }
}