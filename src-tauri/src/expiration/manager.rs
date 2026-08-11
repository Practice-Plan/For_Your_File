//! Expiration manager for tracking and managing entry expirations

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::models::Entry;

/// Expiration status for an entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpirationStatus {
    /// Entry has expired
    Expired { expired_at: i64 },
    /// Entry is expiring soon (within warning threshold)
    ExpiringSoon {
        expires_at: i64,
        days_remaining: i32,
    },
    /// Entry has no expiration or is not expiring soon
    NotExpiring,
}

/// Configuration for expiration manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpirationConfig {
    /// Days before expiration to show warning
    pub warning_days: i32,
    /// Enable automatic notifications
    pub enable_notifications: bool,
    /// Auto-delete expired entries (dangerous!)
    pub auto_delete_expired: bool,
    /// Check interval in hours
    pub check_interval_hours: u64,
}

impl Default for ExpirationConfig {
    fn default() -> Self {
        Self {
            warning_days: 7,
            enable_notifications: true,
            auto_delete_expired: false,
            check_interval_hours: 1,
        }
    }
}

/// Expiration manager for handling entry expiration logic
pub struct ExpirationManager {
    conn: Connection,
    config: ExpirationConfig,
}

impl ExpirationManager {
    /// Create a new expiration manager
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            config: ExpirationConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(conn: Connection, config: ExpirationConfig) -> Self {
        Self { conn, config }
    }

    /// Update configuration
    #[allow(dead_code)]
    pub fn set_config(&mut self, config: ExpirationConfig) {
        self.config = config;
    }

    /// Get current configuration
    #[allow(dead_code)]
    pub fn get_config(&self) -> &ExpirationConfig {
        &self.config
    }

    /// Check all entries that have expired
    pub fn check_expired_entries(&self) -> Result<Vec<Entry>> {
        let now = Utc::now().timestamp();

        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, lnk_path, target_path, parameters, working_dir, tags, notes,
                       frequency, last_opened, created_at, updated_at, expires_at
                FROM entries
                WHERE expires_at IS NOT NULL AND expires_at < ?1
                ORDER BY expires_at ASC
                "#,
            )
            .context("Failed to prepare expired entries query")?;

        let entries = stmt
            .query_map([now], Entry::from_row)
            .context("Failed to query expired entries")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect expired entries")?;

        Ok(entries)
    }

    /// Get entries expiring within the warning period
    pub fn get_expiring_soon(&self) -> Result<Vec<(Entry, i32)>> {
        let now = Utc::now();
        let warning_threshold = now + Duration::days(self.config.warning_days as i64);
        let warning_ts = warning_threshold.timestamp();
        let now_ts = now.timestamp();

        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, lnk_path, target_path, parameters, working_dir, tags, notes,
                       frequency, last_opened, created_at, updated_at, expires_at
                FROM entries
                WHERE expires_at IS NOT NULL 
                  AND expires_at >= ?1 
                  AND expires_at <= ?2
                ORDER BY expires_at ASC
                "#,
            )
            .context("Failed to prepare expiring soon query")?;

        let entries = stmt
            .query_map([now_ts, warning_ts], |row| {
                let entry = Entry::from_row(row)?;
                let expires_at = entry.expires_at.unwrap_or(0);
                let days_remaining = ((expires_at - now_ts) / 86400).max(0) as i32;
                Ok((entry, days_remaining))
            })
            .context("Failed to query expiring soon entries")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect expiring soon entries")?;

        Ok(entries)
    }

    /// Set expiration date for an entry
    pub fn set_expiration(&self, entry_id: i64, expires_at: DateTime<Utc>) -> Result<()> {
        let ts = expires_at.timestamp();
        let now = Utc::now().timestamp();

        self.conn
            .execute(
                "UPDATE entries SET expires_at = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![ts, now, entry_id],
            )
            .context("Failed to set expiration")?;

        Ok(())
    }

    /// Remove expiration date from an entry
    pub fn remove_expiration(&self, entry_id: i64) -> Result<()> {
        let now = Utc::now().timestamp();

        self.conn
            .execute(
                "UPDATE entries SET expires_at = NULL, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, entry_id],
            )
            .context("Failed to remove expiration")?;

        Ok(())
    }

    /// Extend expiration by N days
    pub fn extend_expiration(&self, entry_id: i64, days: i32) -> Result<()> {
        let now = Utc::now().timestamp();

        // Get current expiration or use now as base
        let current_expires_at: Option<i64> = self
            .conn
            .query_row(
                "SELECT expires_at FROM entries WHERE id = ?1",
                [entry_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        let new_expires_at = match current_expires_at {
            Some(ts) => ts + (days as i64 * 86400),
            None => now + (days as i64 * 86400),
        };

        self.conn
            .execute(
                "UPDATE entries SET expires_at = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![new_expires_at, now, entry_id],
            )
            .context("Failed to extend expiration")?;

        Ok(())
    }

    /// Get expiration status for an entry
    pub fn get_expiration_status(&self, entry: &Entry) -> ExpirationStatus {
        match entry.expires_at {
            Some(expires_at) => {
                let now = Utc::now().timestamp();

                if expires_at < now {
                    ExpirationStatus::Expired {
                        expired_at: expires_at,
                    }
                } else {
                    let days_remaining = ((expires_at - now) / 86400) as i32;

                    if days_remaining <= self.config.warning_days {
                        ExpirationStatus::ExpiringSoon {
                            expires_at,
                            days_remaining,
                        }
                    } else {
                        ExpirationStatus::NotExpiring
                    }
                }
            }
            None => ExpirationStatus::NotExpiring,
        }
    }

    /// Get count of expired entries
    pub fn count_expired(&self) -> Result<i64> {
        let now = Utc::now().timestamp();

        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM entries WHERE expires_at IS NOT NULL AND expires_at < ?1",
                [now],
                |row| row.get(0),
            )
            .context("Failed to count expired entries")?;

        Ok(count)
    }

    /// Get count of entries expiring soon
    pub fn count_expiring_soon(&self) -> Result<i64> {
        let now = Utc::now();
        let warning_threshold = now + Duration::days(self.config.warning_days as i64);

        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM entries WHERE expires_at IS NOT NULL AND expires_at >= ?1 AND expires_at <= ?2",
                [now.timestamp(), warning_threshold.timestamp()],
                |row| row.get(0),
            )
            .context("Failed to count expiring soon entries")?;

        Ok(count)
    }

    /// Delete all expired entries
    pub fn delete_all_expired(&self) -> Result<usize> {
        let now = Utc::now().timestamp();

        let affected = self
            .conn
            .execute(
                "DELETE FROM entries WHERE expires_at IS NOT NULL AND expires_at < ?1",
                [now],
            )
            .context("Failed to delete expired entries")?;

        Ok(affected)
    }

    /// Get the underlying connection (for use in timer)
    #[allow(dead_code)]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// Helper function to format remaining time
#[allow(dead_code)]
pub fn format_remaining_time(seconds: i64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{} days", days)
    } else if hours > 0 {
        format!("{} hours", hours)
    } else if minutes > 0 {
        format!("{} minutes", minutes)
    } else {
        "less than a minute".to_string()
    }
}

/// Helper function to format expiration date
#[allow(dead_code)]
pub fn format_expiration_date(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);
    dt.format("%Y-%m-%d %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_remaining_time() {
        assert_eq!(format_remaining_time(172800), "2 days"); // 2 days
        assert_eq!(format_remaining_time(3600), "1 hours"); // 1 hour
        assert_eq!(format_remaining_time(120), "2 minutes"); // 2 minutes
    }

    #[test]
    fn test_expiration_config_default() {
        let config = ExpirationConfig::default();
        assert_eq!(config.warning_days, 7);
        assert!(config.enable_notifications);
        assert!(!config.auto_delete_expired);
    }
}
