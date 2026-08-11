//! Models for Tauri backend
//!
//! Contains data structures used for IPC and database operations.

use rusqlite::Row;
use serde::{Deserialize, Serialize};

/// Target type for LNK shortcuts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LnkTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub path: String,
}

impl LnkTarget {
    pub fn from_path(path: &str) -> Self {
        if path.starts_with("http://") || path.starts_with("https://") {
            return LnkTarget {
                target_type: "Url".to_string(),
                path: path.to_string(),
            };
        }

        if let Ok(p) = std::path::Path::new(path).canonicalize() {
            if p.is_dir() {
                return LnkTarget {
                    target_type: "Folder".to_string(),
                    path: path.to_string(),
                };
            } else if p.is_file() {
                return LnkTarget {
                    target_type: "File".to_string(),
                    path: path.to_string(),
                };
            }
        }

        LnkTarget {
            target_type: "File".to_string(),
            path: path.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.path
    }
}

/// Represents a single entry in the LNK File Management Center
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Option<i64>,
    pub lnk_path: String,
    pub target_path: String,
    pub target_type: LnkTarget,
    pub parameters: Option<String>,
    pub working_dir: Option<String>,
    pub description: Option<String>,
    pub icon_location: Option<String>,
    pub icon_index: Option<i32>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub frequency: i32,
    pub last_opened: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub group_id: Option<i64>,
    pub expires_at: Option<i64>,
}

impl Entry {
    #[allow(dead_code)]
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
}

impl Entry {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let target_path: String = row.get::<_, Option<String>>(2)?.unwrap_or_default();
        let target_type = LnkTarget::from_path(&target_path);

        // Use .ok() for all fields to gracefully handle NULL values in the database.
        // This prevents "Failed to fetch entry" errors when older rows have NULL
        // in columns that are non-Option in the struct (e.g. frequency).
        let lnk_path: Option<String> = row.get(1).ok();
        let description: Option<String> = row.get(5).ok();
        let icon_location: Option<String> = row.get(6).ok();
        let icon_index: Option<i32> = row.get(7).ok();

        Ok(Entry {
            id: Some(row.get(0)?),
            lnk_path: lnk_path.unwrap_or_default(),
            target_path,
            target_type,
            parameters: row.get(3).ok(),
            working_dir: row.get(4).ok(),
            description,
            icon_location,
            icon_index,
            tags: row.get(8).ok(),
            notes: row.get(9).ok(),
            frequency: row.get::<_, Option<i32>>(10)?.unwrap_or(0),
            last_opened: row.get(11).ok(),
            created_at: row.get::<_, Option<i64>>(12)?.unwrap_or(0),
            updated_at: row.get::<_, Option<i64>>(13)?.unwrap_or(0),
            group_id: None,
            expires_at: row.get(14).ok(),
        })
    }
}

/// Represents a group for organizing entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Group {
    #[allow(dead_code)]
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Group {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            color: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}
