//! Entry model representing a single LNK shortcut entry

use crate::lnk::LnkProperties;
use crate::models::FromRow;
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Target type for LNK shortcuts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LnkTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub path: String,
}

impl LnkTarget {
    /// Create a new LnkTarget from a path string
    pub fn from_path(path: impl Into<String>) -> Self {
        let path_str = path.into();

        // Check if it's a URL
        if path_str.starts_with("http://") || path_str.starts_with("https://") {
            return LnkTarget {
                target_type: "Url".to_string(),
                path: path_str,
            };
        }

        // Try to determine if it's a file or folder
        if let Ok(path) = std::path::Path::new(&path_str).canonicalize() {
            if path.is_dir() {
                return LnkTarget {
                    target_type: "Folder".to_string(),
                    path: path_str,
                };
            } else if path.is_file() {
                return LnkTarget {
                    target_type: "File".to_string(),
                    path: path_str,
                };
            }
        }

        // Default to file if we can't determine
        LnkTarget {
            target_type: "File".to_string(),
            path: path_str,
        }
    }

    /// Get the target path as a string
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Check if this is a URL target
    pub fn is_url(&self) -> bool {
        self.target_type == "Url"
    }

    /// Check if this is a file target
    pub fn is_file(&self) -> bool {
        self.target_type == "File"
    }

    /// Check if this is a folder target
    pub fn is_folder(&self) -> bool {
        self.target_type == "Folder"
    }

    /// Check if the target exists
    pub fn exists(&self) -> bool {
        if self.is_url() {
            true // URLs are considered to always exist
        } else {
            Path::new(&self.path).exists()
        }
    }
}

/// Metadata extracted from a LNK file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LnkMetadata {
    /// Target path the shortcut points to
    pub target_path: String,
    /// Target type (file, folder, URL)
    pub target_type: LnkTarget,
    /// Command line arguments
    pub arguments: Option<String>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Icon location
    pub icon_location: Option<String>,
    /// Icon index
    pub icon_index: Option<i32>,
    /// Show command (1=normal, 3=maximized, 7=minimized)
    pub show_command: Option<i32>,
}

impl LnkMetadata {
    /// Create new metadata from a target path
    pub fn new(target_path: impl Into<String>) -> Self {
        let path = target_path.into();
        let target_type = LnkTarget::from_path(&path);
        Self {
            target_path: path,
            target_type,
            arguments: None,
            working_directory: None,
            description: None,
            icon_location: None,
            icon_index: None,
            show_command: Some(1), // Normal window
        }
    }

    /// Create metadata from LnkProperties
    pub fn from_properties(props: LnkProperties) -> Self {
        let target_type = LnkTarget::from_path(&props.target_path);
        Self {
            target_path: props.target_path,
            target_type,
            arguments: props.arguments,
            working_directory: props.working_directory,
            description: props.description,
            icon_location: props.icon_location,
            icon_index: props.icon_index,
            show_command: props.show_command,
        }
    }

    /// Set arguments (builder pattern)
    pub fn with_arguments(mut self, args: impl Into<String>) -> Self {
        self.arguments = Some(args.into());
        self
    }

    /// Set working directory (builder pattern)
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Set description (builder pattern)
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set icon (builder pattern)
    pub fn with_icon(mut self, location: impl Into<String>, index: i32) -> Self {
        self.icon_location = Some(location.into());
        self.icon_index = Some(index);
        self
    }

    /// Convert to LnkProperties
    pub fn to_properties(&self) -> LnkProperties {
        LnkProperties {
            target_path: self.target_path.clone(),
            arguments: self.arguments.clone(),
            working_directory: self.working_directory.clone(),
            description: self.description.clone(),
            icon_location: self.icon_location.clone(),
            icon_index: self.icon_index,
            show_command: self.show_command,
        }
    }
}

impl From<LnkProperties> for LnkMetadata {
    fn from(props: LnkProperties) -> Self {
        Self::from_properties(props)
    }
}

impl From<LnkMetadata> for LnkProperties {
    fn from(meta: LnkMetadata) -> Self {
        meta.to_properties()
    }
}

/// Represents a single entry in the LNK File Management Center
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Unique identifier
    pub id: Option<i64>,
    /// Path to the .lnk file
    pub lnk_path: String,
    /// Target path that the shortcut points to
    pub target_path: String,
    /// Target type
    pub target_type: LnkTarget,
    /// Command line parameters
    pub parameters: Option<String>,
    /// Working directory for the shortcut
    pub working_dir: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Icon location
    pub icon_location: Option<String>,
    /// Icon index
    pub icon_index: Option<i32>,
    /// Tags associated with this entry (comma-separated)
    pub tags: Option<String>,
    /// User notes for this entry
    pub notes: Option<String>,
    /// Number of times this entry has been opened
    pub frequency: i32,
    /// Timestamp of last open action
    pub last_opened: Option<i64>,
    /// Creation timestamp (Unix epoch)
    pub created_at: i64,
    /// Last update timestamp (Unix epoch)
    pub updated_at: i64,
    /// Group ID if this entry belongs to a group
    pub group_id: Option<i64>,
    /// Expiration timestamp (Unix epoch), if set
    pub expires_at: Option<i64>,
}

impl Entry {
    /// Create a new entry with the given LNK path and target path
    pub fn new(lnk_path: String, target_path: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        let target_type = LnkTarget::from_path(&target_path);
        Self {
            id: None,
            lnk_path,
            target_path,
            target_type,
            parameters: None,
            working_dir: None,
            description: None,
            icon_location: None,
            icon_index: None,
            tags: None,
            notes: None,
            frequency: 0,
            last_opened: None,
            created_at: now,
            updated_at: now,
            group_id: None,
            expires_at: None,
        }
    }

    /// Create an entry from metadata
    pub fn from_metadata(lnk_path: impl Into<String>, metadata: LnkMetadata) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            lnk_path: lnk_path.into(),
            target_path: metadata.target_path,
            target_type: metadata.target_type,
            parameters: metadata.arguments,
            working_dir: metadata.working_directory,
            description: metadata.description,
            icon_location: metadata.icon_location,
            icon_index: metadata.icon_index,
            tags: None,
            notes: None,
            frequency: 0,
            last_opened: None,
            created_at: now,
            updated_at: now,
            group_id: None,
            expires_at: None,
        }
    }

    /// Create an entry from LnkProperties
    pub fn from_properties(lnk_path: impl Into<String>, props: LnkProperties) -> Self {
        let metadata = LnkMetadata::from_properties(props);
        Self::from_metadata(lnk_path, metadata)
    }

    /// Get metadata from this entry
    pub fn to_metadata(&self) -> LnkMetadata {
        LnkMetadata {
            target_path: self.target_path.clone(),
            target_type: self.target_type.clone(),
            arguments: self.parameters.clone(),
            working_directory: self.working_dir.clone(),
            description: self.description.clone(),
            icon_location: self.icon_location.clone(),
            icon_index: self.icon_index,
            show_command: Some(1), // Default to normal window
        }
    }

    /// Record that this entry was opened
    pub fn record_open(&mut self) {
        self.frequency += 1;
        self.last_opened = Some(chrono::Utc::now().timestamp());
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Add tags to this entry
    pub fn add_tags(&mut self, new_tags: &[&str]) {
        let existing: Vec<String> = self
            .tags
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let mut all_tags = existing;
        for tag in new_tags {
            if !all_tags.contains(&tag.to_string()) {
                all_tags.push(tag.to_string());
            }
        }

        self.tags = Some(all_tags.join(","));
        self.updated_at = chrono::Utc::now().timestamp();
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

    /// Set parameters (builder pattern)
    pub fn with_parameters(mut self, parameters: impl Into<String>) -> Self {
        self.parameters = Some(parameters.into());
        self
    }

    /// Set working directory (builder pattern)
    pub fn with_working_dir(mut self, working_dir: impl Into<String>) -> Self {
        self.working_dir = Some(working_dir.into());
        self
    }
}

impl FromRow for Entry {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        // Get the target path and derive target_type
        let target_path: String = row.get(2)?;
        let target_type = LnkTarget::from_path(&target_path);

        Ok(Entry {
            id: Some(row.get(0)?),
            lnk_path: row.get(1)?,
            target_path,
            target_type,
            parameters: row.get(3)?,
            working_dir: row.get(4)?,
            description: None, // Not in current schema
            icon_location: None,
            icon_index: None,
            tags: row.get(5)?,
            notes: row.get(6)?,
            frequency: row.get(7)?,
            last_opened: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            group_id: None, // Not in current schema
            expires_at: row.get(11)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lnk_target_url() {
        let target = LnkTarget::from_path("https://example.com");
        assert!(target.is_url());
        assert!(target.exists());
    }

    #[test]
    fn test_lnk_target_nonexistent() {
        let target = LnkTarget::from_path("/nonexistent/path");
        assert!(!target.exists());
    }

    #[test]
    fn test_entry_creation() {
        let entry = Entry::new("/path/to/shortcut.lnk".to_string(), "C:\\Program.exe".to_string());
        assert_eq!(entry.lnk_path, "/path/to/shortcut.lnk");
        assert_eq!(entry.target_path, "C:\\Program.exe");
        assert_eq!(entry.frequency, 0);
    }

    #[test]
    fn test_entry_from_metadata() {
        let metadata = LnkMetadata::new("C:\\Program.exe")
            .with_arguments("--flag")
            .with_working_directory("C:\\")
            .with_description("Test shortcut");

        let entry = Entry::from_metadata("/path/to/shortcut.lnk", metadata);
        assert_eq!(entry.target_path, "C:\\Program.exe");
        assert_eq!(entry.parameters, Some("--flag".to_string()));
        assert_eq!(entry.working_dir, Some("C:\\".to_string()));
    }

    #[test]
    fn test_entry_record_open() {
        let mut entry = Entry::new("C:/test.lnk".to_string(), "C:/target.exe".to_string());
        entry.record_open();
        assert_eq!(entry.frequency, 1);
        assert!(entry.last_opened.is_some());
    }

    #[test]
    fn test_entry_add_tags() {
        let mut entry = Entry::new("C:/test.lnk".to_string(), "C:/target.exe".to_string());
        entry.add_tags(&["work", "important"]);
        assert!(entry.tags.is_some());
        let tags = entry.tags.unwrap();
        assert!(tags.contains("work"));
        assert!(tags.contains("important"));
    }
}