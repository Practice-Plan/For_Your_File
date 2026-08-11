//! Background timer for periodic expiration checks
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use log::{error, info, warn};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use super::{ExpirationConfig, ExpirationManager};
use crate::models::Entry;

/// Background timer that periodically checks for expired entries
pub struct ExpirationTimer {
    /// Flag to stop the timer
    running: Arc<AtomicBool>,
    /// Last check timestamp
    last_check: Arc<std::sync::Mutex<Option<DateTime<Utc>>>>,
}

impl ExpirationTimer {
    /// Create a new expiration timer
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            last_check: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Start the background timer
    pub fn start(&self, app_handle: AppHandle, config: ExpirationConfig, conn: rusqlite::Connection) {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("Expiration timer is already running");
            return;
        }

        let running = self.running.clone();
        let last_check = self.last_check.clone();
        let check_interval = Duration::from_secs(config.check_interval_hours * 3600);

        info!(
            "Starting expiration timer with interval of {} hours",
            config.check_interval_hours
        );

        tokio::spawn(async move {
            // Create manager inside the task
            let manager = ExpirationManager::with_config(conn, config);

            loop {
                if !running.load(Ordering::SeqCst) {
                    info!("Expiration timer stopped");
                    break;
                }

                // Perform expiration check
                match Self::perform_check(&manager, &app_handle) {
                    Ok(_) => {
                        // Update last check time
                        if let Ok(mut last) = last_check.lock() {
                            *last = Some(Utc::now());
                        }
                    }
                    Err(e) => {
                        error!("Expiration check failed: {}", e);
                    }
                }

                // Wait for next check interval
                sleep(check_interval).await;
            }
        });
    }

    /// Perform a single expiration check
    fn perform_check(manager: &ExpirationManager, app_handle: &AppHandle) -> anyhow::Result<()> {
        let now = Utc::now();
        info!("Performing expiration check at {}", now.format("%Y-%m-%d %H:%M:%S"));

        // Check for expired entries
        let expired_entries = manager.check_expired_entries()?;
        if !expired_entries.is_empty() {
            info!("Found {} expired entries", expired_entries.len());

            // Emit event to frontend
            app_handle.emit("entries-expired", &expired_entries)?;

            // Show notification for each expired entry
            for entry in &expired_entries {
                app_handle.emit("expiration-notification", ExpirationNotification {
                    entry_id: entry.id.unwrap_or(0),
                    entry_name: entry.lnk_path.clone(),
                    status: "expired".to_string(),
                    message: format!("Entry '{}' has expired", entry.lnk_path),
                })?;
            }
        }

        // Check for entries expiring soon
        let expiring_soon = manager.get_expiring_soon()?;
        if !expiring_soon.is_empty() {
            info!("Found {} entries expiring soon", expiring_soon.len());

            // Emit event to frontend
            app_handle.emit("entries-expiring-soon", &expiring_soon.iter().map(|(e, d)| ExpiringSoonInfo {
                entry: e.clone(),
                days_remaining: *d,
            }).collect::<Vec<_>>())?;

            // Show notification for entries expiring within 3 days
            for (entry, days) in &expiring_soon {
                if *days <= 3 {
                    app_handle.emit("expiration-notification", ExpirationNotification {
                        entry_id: entry.id.unwrap_or(0),
                        entry_name: entry.lnk_path.clone(),
                        status: "expiring_soon".to_string(),
                        message: format!("Entry '{}' expires in {} days", entry.lnk_path, days),
                    })?;
                }
            }
        }

        // Auto-delete expired entries if configured
        let config = manager.get_config();
        if config.auto_delete_expired && !expired_entries.is_empty() {
            let deleted = manager.delete_all_expired()?;
            info!("Auto-deleted {} expired entries", deleted);
        }

        Ok(())
    }

    /// Stop the background timer
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Expiration timer stopped");
    }

    /// Check if the timer is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get the last check time
    pub fn last_check_time(&self) -> Option<DateTime<Utc>> {
        self.last_check.lock().map(|l| *l).unwrap_or(None)
    }
}

impl Default for ExpirationTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification payload for expiration events
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExpirationNotification {
    /// Entry ID
    pub entry_id: i64,
    /// Entry name (lnk_path)
    pub entry_name: String,
    /// Status: "expired" or "expiring_soon"
    pub status: String,
    /// Human-readable message
    pub message: String,
}

/// Information about entries expiring soon
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExpiringSoonInfo {
    /// The entry
    pub entry: Entry,
    /// Days remaining until expiration
    pub days_remaining: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = ExpirationTimer::new();
        assert!(!timer.is_running());
    }

    #[test]
    fn test_timer_stop() {
        let timer = ExpirationTimer::new();
        timer.stop();
        assert!(!timer.is_running());
    }
}