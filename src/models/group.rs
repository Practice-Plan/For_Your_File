//! Group model for organizing entries

use rusqlite::Row;
use serde::{Deserialize, Serialize};

use super::FromRow;

/// Represents a group for organizing multiple entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Unique identifier
    pub id: Option<i64>,
    /// Group name
    pub name: String,
    /// Group color (hex format, e.g., "#FF5733")
    pub color: String,
    /// Creation timestamp (Unix epoch)
    pub created_at: i64,
    /// Last update timestamp (Unix epoch)
    pub updated_at: i64,
}

impl Group {
    /// Create a new group with the given name and color
    pub fn new(name: String, color: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            name,
            color,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new group with default color
    pub fn with_name(name: String) -> Self {
        Self::new(name, "#3498db".to_string())
    }

    /// Update the group's color
    pub fn set_color(&mut self, color: String) {
        self.color = color;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Update the group's name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

impl FromRow for Group {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Group {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            color: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_new() {
        let group = Group::new("Work".to_string(), "#FF5733".to_string());
        assert!(group.id.is_none());
        assert_eq!(group.name, "Work");
        assert_eq!(group.color, "#FF5733");
    }

    #[test]
    fn test_group_with_name() {
        let group = Group::with_name("Personal".to_string());
        assert_eq!(group.name, "Personal");
        assert_eq!(group.color, "#3498db"); // Default color
    }

    #[test]
    fn test_group_set_color() {
        let mut group = Group::new("Work".to_string(), "#FF5733".to_string());
        group.set_color("#00FF00".to_string());
        assert_eq!(group.color, "#00FF00");
    }
}