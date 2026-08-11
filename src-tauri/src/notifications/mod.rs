//! Notification system for expiration reminders
//!
//! Uses tauri-plugin-notification for cross-platform notifications.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Notification types for expiration events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum NotificationType {
    /// Entry has expired
    Expired,
    /// Entry is expiring soon
    ExpiringSoon { days_remaining: i32 },
    /// Expiration reminder (scheduled)
    Reminder,
}

/// Notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ExpirationNotificationPayload {
    /// Entry ID
    pub entry_id: i64,
    /// Entry name
    pub entry_name: String,
    /// Notification type
    pub notification_type: NotificationType,
    /// Message to display
    pub message: String,
}

/// Show a notification for an expired entry
#[cfg(windows)]
pub fn show_expired_notification(app: &AppHandle, entry_name: &str, entry_id: i64) -> Result<()> {
    let title = "LNK File Expired";
    let body = format!("'{}' has expired and may need attention.", entry_name);

    app.notification()
        .builder()
        .title(title)
        .body(&body)
        .icon("icon")
        .show()?;

    log::info!("Shown expired notification for entry {}", entry_id);
    Ok(())
}

/// Show a notification for an entry expiring soon
#[cfg(windows)]
pub fn show_expiring_soon_notification(
    app: &AppHandle,
    entry_name: &str,
    days_remaining: i32,
    entry_id: i64,
) -> Result<()> {
    let title = "LNK File Expiring Soon";
    let body = format!(
        "'{}' will expire in {} day(s).",
        entry_name, days_remaining
    );

    app.notification()
        .builder()
        .title(title)
        .body(&body)
        .icon("icon")
        .show()?;

    log::info!(
        "Shown expiring soon notification for entry {} ({} days)",
        entry_id,
        days_remaining
    );
    Ok(())
}

/// Show a batch notification for multiple expirations
#[cfg(windows)]
pub fn show_batch_expiration_notification(
    app: &AppHandle,
    expired_count: usize,
    expiring_soon_count: usize,
) -> Result<()> {
    let title = "LNK File Expiration Summary";
    let body = if expired_count > 0 && expiring_soon_count > 0 {
        format!(
            "{} file(s) have expired, {} will expire soon.",
            expired_count, expiring_soon_count
        )
    } else if expired_count > 0 {
        format!("{} file(s) have expired.", expired_count)
    } else if expiring_soon_count > 0 {
        format!("{} file(s) will expire soon.", expiring_soon_count)
    } else {
        return Ok(());
    };

    app.notification()
        .builder()
        .title(title)
        .body(&body)
        .icon("icon")
        .show()?;

    log::info!(
        "Shown batch notification: {} expired, {} expiring soon",
        expired_count,
        expiring_soon_count
    );
    Ok(())
}

/// Show a notification that an entry was successfully extended
#[cfg(windows)]
pub fn show_extension_notification(app: &AppHandle, entry_name: &str, days: i32) -> Result<()> {
    let title = "Expiration Extended";
    let body = format!("'{}' expiration extended by {} day(s).", entry_name, days);

    app.notification()
        .builder()
        .title(title)
        .body(&body)
        .icon("icon")
        .show()?;

    log::info!("Shown extension notification for entry '{}'", entry_name);
    Ok(())
}

#[cfg(not(windows))]
pub fn show_expired_notification(app: &AppHandle, entry_name: &str, entry_id: i64) -> Result<()> {
    log::info!("Expired notification (non-Windows): {} - {}", entry_name, entry_id);
    Ok(())
}

#[cfg(not(windows))]
pub fn show_expiring_soon_notification(
    app: &AppHandle,
    entry_name: &str,
    days_remaining: i32,
    entry_id: i64,
) -> Result<()> {
    log::info!(
        "Expiring soon notification (non-Windows): {} - {} days",
        entry_name,
        days_remaining
    );
    Ok(())
}

#[cfg(not(windows))]
pub fn show_batch_expiration_notification(
    app: &AppHandle,
    expired_count: usize,
    expiring_soon_count: usize,
) -> Result<()> {
    log::info!(
        "Batch notification (non-Windows): {} expired, {} expiring soon",
        expired_count,
        expiring_soon_count
    );
    Ok(())
}

#[cfg(not(windows))]
pub fn show_extension_notification(app: &AppHandle, entry_name: &str, days: i32) -> Result<()> {
    log::info!(
        "Extension notification (non-Windows): {} - {} days",
        entry_name,
        days
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_payload_creation() {
        let payload = ExpirationNotificationPayload {
            entry_id: 123,
            entry_name: "test.lnk".to_string(),
            notification_type: NotificationType::Expired,
            message: "Entry has expired".to_string(),
        };

        assert_eq!(payload.entry_id, 123);
        assert_eq!(payload.entry_name, "test.lnk");
    }

    #[test]
    fn test_notification_type_serialization() {
        let nt = NotificationType::ExpiringSoon { days_remaining: 5 };
        let json = serde_json::to_string(&nt).unwrap();
        assert!(json.contains("ExpiringSoon"));
        assert!(json.contains("5"));
    }
}