//! Tauri commands for hotkey and protocol management
//!
//! Provides IPC interface between frontend and backend for operations.

use crate::cli::CliArgs;
use crate::db;
use crate::app_scanner;
use crate::expiration::{ExpirationConfig, ExpirationManager, ExpirationStatus};
use crate::hotkey::{HotkeyConfig, HotkeyManager};
use crate::lnk;
use crate::notifications::{
    show_batch_expiration_notification, show_expired_notification, show_expiring_soon_notification,
    show_extension_notification,
};
use crate::protocol::{parse_deep_link, ProtocolAction, ProtocolRequest};
use rusqlite::OptionalExtension;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::models::Entry;

// `raw_arg` is a Windows-only extension on `std::process::Command` that allows
// appending a raw (unparsed) argument string. Used when forwarding application
// parameters verbatim to the launched executable.
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Global hotkey manager state
pub struct HotkeyState(pub Mutex<HotkeyManager>);

/// Get the application version (from Cargo.toml at compile time)
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Parse a .lnk file and return its properties for auto-completion.
/// Used by the Add Entry modal to auto-fill fields when a user uploads a .lnk file.
#[tauri::command]
pub fn parse_lnk_file(path: String) -> Result<lnk::LnkProperties, String> {
    log::info!("Parsing LNK file: {}", path);
    lnk::parse_lnk_file(&path).map_err(|e| {
        log::error!("Failed to parse LNK file '{}': {}", path, e);
        e
    })
}

/// List installed applications by scanning the Windows Start Menu.
/// Returns a list of apps with their names, target paths, and .lnk paths.
#[tauri::command]
pub fn list_installed_apps() -> Result<Vec<app_scanner::InstalledApp>, String> {
    log::info!("Scanning Start Menu for installed applications...");
    let apps = app_scanner::list_installed_apps();
    log::info!("Found {} installed applications", apps.len());
    Ok(apps)
}

/// Extract an application's icon as a base64-encoded PNG string.
/// Used by the app selector modal to display application icons.
#[tauri::command]
pub fn get_app_icon(exe_path: String) -> Result<String, String> {
    log::debug!("Extracting icon for: {}", exe_path);
    app_scanner::extract_icon_as_base64(&exe_path)
}

/// Register a global hotkey
#[tauri::command]
pub fn register_global_hotkey(hotkey: String, state: State<HotkeyState>) -> Result<(), String> {
    let manager = state.0.lock().unwrap();
    let config = HotkeyConfig::from_string(&hotkey)?;
    manager.register(&config.modifiers, &config.key)?;
    log::info!("Global hotkey registered: {}", hotkey);
    Ok(())
}

/// Unregister the global hotkey
#[tauri::command]
pub fn unregister_global_hotkey(state: State<HotkeyState>) -> Result<(), String> {
    let manager = state.0.lock().unwrap();
    manager.unregister()?;
    log::info!("Global hotkey unregistered");
    Ok(())
}

/// Update the hotkey to a new combination
#[tauri::command]
pub fn update_global_hotkey(hotkey: String, state: State<HotkeyState>) -> Result<(), String> {
    let manager = state.0.lock().unwrap();
    let config = HotkeyConfig::from_string(&hotkey)?;
    manager.update_hotkey(&config.modifiers, &config.key)?;
    log::info!("Global hotkey updated to: {}", hotkey);
    Ok(())
}

/// Check if a hotkey has a conflict
#[tauri::command]
pub fn check_hotkey_conflict(hotkey: String, state: State<HotkeyState>) -> Result<bool, String> {
    let manager = state.0.lock().unwrap();
    let config = HotkeyConfig::from_string(&hotkey)?;
    let has_conflict = manager.check_conflict(&config.modifiers, &config.key)?;
    log::info!("Hotkey conflict check for '{}': {}", hotkey, has_conflict);
    Ok(has_conflict)
}

/// Get current hotkey configuration
#[tauri::command]
pub fn get_hotkey_config(state: State<HotkeyState>) -> Result<HotkeyConfig, String> {
    let manager = state.0.lock().unwrap();
    Ok(manager.get_config())
}

/// Test a hotkey (register temporarily and immediately unregister)
#[tauri::command]
pub fn test_hotkey(hotkey: String, state: State<HotkeyState>) -> Result<bool, String> {
    let manager = state.0.lock().unwrap();
    let config = HotkeyConfig::from_string(&hotkey)?;
    let has_conflict = manager.check_conflict(&config.modifiers, &config.key)?;
    Ok(!has_conflict)
}

/// Get suggested alternative hotkeys
#[tauri::command]
pub fn get_suggested_hotkeys() -> Vec<String> {
    vec![
        "Alt+Space".to_string(),
        "Ctrl+Space".to_string(),
        "Alt+Q".to_string(),
        "Ctrl+Shift+Space".to_string(),
        "Alt+Shift+Space".to_string(),
        "Ctrl+Alt+Space".to_string(),
        "F12".to_string(),
        "Ctrl+F12".to_string(),
        "Alt+F12".to_string(),
    ]
}

/// Parse a deep link URL and return the request
#[tauri::command]
pub fn parse_protocol_url(url: String) -> Result<ProtocolRequest, String> {
    parse_deep_link(&url).map_err(|e| {
        log::error!("Failed to parse protocol URL '{}': {}", url, e);
        e.to_string()
    })
}

/// Handle a protocol request (sent from frontend)
#[tauri::command]
pub async fn handle_protocol_request(
    request: ProtocolRequest,
    app_handle: AppHandle,
) -> Result<(), String> {
    log::info!("Handling protocol request: {:?}", request.action);

    // Emit event to frontend
    app_handle
        .emit("protocol-request", &request)
        .map_err(|e| e.to_string())?;

    // Log the action
    match request.action {
        ProtocolAction::Add => {
            log::info!("Protocol action: Add entry from {:?}", request.path);
        }
        ProtocolAction::Open => {
            log::info!("Protocol action: Open entry {:?}", request.id);
        }
        ProtocolAction::Search => {
            log::info!("Protocol action: Search for {:?}", request.query);
        }
        ProtocolAction::Settings => {
            log::info!("Protocol action: Open settings");
        }
    }

    Ok(())
}

/// Get CLI arguments that were passed at startup
#[tauri::command]
pub fn get_cli_args() -> CliArgs {
    CliArgs::parse()
}

/// Show the main window
#[tauri::command]
pub fn show_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        log::info!("Window shown and focused");
    }
    Ok(())
}

/// Hide the main window
#[tauri::command]
pub fn hide_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        log::info!("Window hidden");
    }
    Ok(())
}

/// Minimize window to tray
#[tauri::command]
pub fn minimize_to_tray(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        log::info!("Window minimized to tray");
    }
    Ok(())
}

/// Register the shell extension for Windows Explorer context menu
#[cfg(windows)]
#[tauri::command]
pub fn register_shell_extension() -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // Get the executable path
    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

    // Get the installation script path
    let script_path = exe_path
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("install-context-menu.ps1"))
        .ok_or_else(|| "Failed to find installation script".to_string())?;

    if !script_path.exists() {
        return Err(format!(
            "Installation script not found at: {}",
            script_path.display()
        ));
    }

    // Execute PowerShell registration script
    let result = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
            "-ExePath",
            &exe_path.to_string_lossy(),
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to execute registration script: {}", e))?;

    if result.status.success() {
        Ok("Context menu registered successfully. Please restart Windows Explorer to see the changes.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("Registration failed: {}", stderr))
    }
}

/// Unregister the shell extension from Windows Explorer context menu
#[cfg(windows)]
#[tauri::command]
pub fn unregister_shell_extension() -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // Get the uninstallation script path
    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

    let script_path = exe_path
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("uninstall-context-menu.ps1"))
        .ok_or_else(|| "Failed to find uninstallation script".to_string())?;

    if !script_path.exists() {
        return Err(format!(
            "Uninstallation script not found at: {}",
            script_path.display()
        ));
    }

    // Execute PowerShell uninstallation script
    let result = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to execute uninstallation script: {}", e))?;

    if result.status.success() {
        Ok("Context menu unregistered successfully. Please restart Windows Explorer to see the changes.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("Unregistration failed: {}", stderr))
    }
}

/// Check if the shell extension is registered
#[cfg(windows)]
#[tauri::command]
pub fn is_shell_extension_registered() -> bool {
    // Check if the registry key exists
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::System::Registry::*;

    let key_path: Vec<u16> = OsStr::new("SOFTWARE\\Classes\\*\\shell\\AddToFileManagementCenter")
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut hkey: windows::Win32::System::Registry::HKEY =
        windows::Win32::System::Registry::HKEY::default();

    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            windows::core::PCWSTR(key_path.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
    };

    if result.is_ok() {
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        true
    } else {
        false
    }
}

#[cfg(not(windows))]
#[tauri::command]
pub fn register_shell_extension() -> Result<String, String> {
    Err("Shell extension is only available on Windows".to_string())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn unregister_shell_extension() -> Result<String, String> {
    Err("Shell extension is only available on Windows".to_string())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn is_shell_extension_registered() -> bool {
    false
}

// ============================================================================
// Expiration Commands
// ============================================================================

/// Check for expired entries
#[tauri::command]
pub fn check_expired_entries(app_handle: AppHandle) -> Result<Vec<Entry>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let manager = ExpirationManager::new(conn);
    manager.check_expired_entries().map_err(|e| e.to_string())
}

/// Get entries expiring soon
#[tauri::command]
pub fn get_expiring_soon(
    app_handle: AppHandle,
    warning_days: Option<i32>,
) -> Result<Vec<(Entry, i32)>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut config = ExpirationConfig::default();
    if let Some(days) = warning_days {
        config.warning_days = days;
    }

    let manager = ExpirationManager::with_config(conn, config);
    manager.get_expiring_soon().map_err(|e| e.to_string())
}

/// Set expiration date for an entry
#[tauri::command]
pub fn set_expiration(app_handle: AppHandle, entry_id: i64, expires_at: i64) -> Result<(), String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let manager = ExpirationManager::new(conn);
    let dt = chrono::DateTime::from_timestamp(expires_at, 0).unwrap_or_else(chrono::Utc::now);
    manager
        .set_expiration(entry_id, dt)
        .map_err(|e| e.to_string())
}

/// Remove expiration from an entry
#[tauri::command]
pub fn remove_expiration(app_handle: AppHandle, entry_id: i64) -> Result<(), String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let manager = ExpirationManager::new(conn);
    manager
        .remove_expiration(entry_id)
        .map_err(|e| e.to_string())
}

/// Extend expiration by N days
#[tauri::command]
pub fn extend_expiration(app_handle: AppHandle, entry_id: i64, days: i32) -> Result<(), String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let manager = ExpirationManager::new(conn);
    manager
        .extend_expiration(entry_id, days)
        .map_err(|e| e.to_string())
}

/// Get expiration status for an entry
#[tauri::command]
pub fn get_expiration_status(
    app_handle: AppHandle,
    entry: Entry,
) -> Result<ExpirationStatus, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let manager = ExpirationManager::new(conn);
    Ok(manager.get_expiration_status(&entry))
}

/// Get expiration counts (expired and expiring soon)
#[tauri::command]
pub fn get_expiration_counts(app_handle: AppHandle) -> Result<ExpirationCounts, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let manager = ExpirationManager::new(conn);

    Ok(ExpirationCounts {
        expired: manager.count_expired().map_err(|e| e.to_string())?,
        expiring_soon: manager.count_expiring_soon().map_err(|e| e.to_string())?,
    })
}

/// Delete all expired entries
#[tauri::command]
pub fn delete_expired_entries(app_handle: AppHandle) -> Result<usize, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let manager = ExpirationManager::new(conn);
    manager.delete_all_expired().map_err(|e| e.to_string())
}

/// Get expiration configuration
#[tauri::command]
pub fn get_expiration_config(_app_handle: AppHandle) -> Result<ExpirationConfig, String> {
    // TODO: Load from app configuration
    Ok(ExpirationConfig::default())
}

/// Update expiration configuration
#[tauri::command]
pub fn update_expiration_config(
    _app_handle: AppHandle,
    config: ExpirationConfig,
) -> Result<(), String> {
    // TODO: Save to app configuration
    log::info!("Expiration config updated: {:?}", config);
    Ok(())
}

/// Show expiration notification manually
#[tauri::command]
pub fn show_expiration_notification(
    app_handle: AppHandle,
    notification_type: String,
    entry_name: String,
    entry_id: i64,
    days_remaining: Option<i32>,
) -> Result<(), String> {
    match notification_type.as_str() {
        "expired" => {
            show_expired_notification(&app_handle, &entry_name, entry_id)
                .map_err(|e| e.to_string())?;
        }
        "expiring_soon" => {
            let days = days_remaining.unwrap_or(7);
            show_expiring_soon_notification(&app_handle, &entry_name, days, entry_id)
                .map_err(|e| e.to_string())?;
        }
        "batch" => {
            let expired_count = if entry_id > 0 { entry_id as usize } else { 0 };
            let expiring_count = days_remaining.unwrap_or(0) as usize;
            show_batch_expiration_notification(&app_handle, expired_count, expiring_count)
                .map_err(|e| e.to_string())?;
        }
        "extended" => {
            let days = days_remaining.unwrap_or(7);
            show_extension_notification(&app_handle, &entry_name, days)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            return Err(format!("Unknown notification type: {}", notification_type));
        }
    }
    Ok(())
}

/// Expiration counts response
#[derive(Debug, serde::Serialize)]
pub struct ExpirationCounts {
    /// Number of expired entries
    pub expired: i64,
    /// Number of entries expiring soon
    pub expiring_soon: i64,
}

// ============================================================================
// Group Commands
// ============================================================================

/// Group model for API responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupResponse {
    /// Unique identifier
    pub id: Option<i64>,
    /// Group name
    pub name: String,
    /// Group color (hex format)
    pub color: String,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
}

/// Group with entry count
#[derive(Debug, Clone, serde::Serialize)]
pub struct GroupWithCountResponse {
    /// Group data
    #[serde(flatten)]
    pub group: GroupResponse,
    /// Number of entries in this group
    pub entry_count: i64,
}

/// Create a new group
#[tauri::command]
pub fn create_group(
    app_handle: AppHandle,
    name: String,
    color: String,
) -> Result<GroupResponse, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, color, now, now],
    )
    .map_err(|e| format!("Failed to create group: {}", e))?;

    let id = conn.last_insert_rowid();

    Ok(GroupResponse {
        id: Some(id),
        name,
        color,
        created_at: now,
        updated_at: now,
    })
}

/// List all groups with entry counts
#[tauri::command]
pub fn list_groups(app_handle: AppHandle) -> Result<Vec<GroupWithCountResponse>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT g.id, g.name, g.color, g.created_at, g.updated_at,
                   COUNT(eg.entry_id) as entry_count
            FROM groups g
            LEFT JOIN entry_groups eg ON g.id = eg.group_id
            GROUP BY g.id
            ORDER BY g.name ASC
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let groups = stmt
        .query_map([], |row| {
            Ok(GroupWithCountResponse {
                group: GroupResponse {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                },
                entry_count: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query groups: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect groups: {}", e))?;

    Ok(groups)
}

/// Get a group by ID
#[tauri::command]
pub fn get_group(app_handle: AppHandle, id: i64) -> Result<Option<GroupWithCountResponse>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT g.id, g.name, g.color, g.created_at, g.updated_at,
                   COUNT(eg.entry_id) as entry_count
            FROM groups g
            LEFT JOIN entry_groups eg ON g.id = eg.group_id
            WHERE g.id = ?1
            GROUP BY g.id
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok(GroupWithCountResponse {
                group: GroupResponse {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                },
                entry_count: row.get(5)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query group: {}", e))?;

    Ok(result)
}

/// Update a group
#[tauri::command]
pub fn update_group(
    app_handle: AppHandle,
    id: i64,
    name: Option<String>,
    color: Option<String>,
) -> Result<Option<GroupResponse>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Check if group exists
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM groups WHERE id = ?1",
            rusqlite::params![id],
            |_| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Failed to check group: {}", e))?
        .is_some();

    if !exists {
        return Ok(None);
    }

    let now = chrono::Utc::now().timestamp();
    let mut updates: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(n) = name {
        updates.push("name = ?".to_string());
        params.push(Box::new(n));
    }
    if let Some(c) = color {
        updates.push("color = ?".to_string());
        params.push(Box::new(c));
    }

    if !updates.is_empty() {
        updates.push("updated_at = ?".to_string());
        params.push(Box::new(now));
        params.push(Box::new(id));

        let sql = format!("UPDATE groups SET {} WHERE id = ?", updates.join(", "));
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        conn.execute(&sql, params_refs.as_slice())
            .map_err(|e| format!("Failed to update group: {}", e))?;
    }

    // Fetch updated group
    let mut stmt = conn
        .prepare("SELECT id, name, color, created_at, updated_at FROM groups WHERE id = ?1")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok(GroupResponse {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to fetch updated group: {}", e))?;

    Ok(result)
}

/// Delete a group
#[tauri::command]
pub fn delete_group(app_handle: AppHandle, id: i64) -> Result<bool, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Delete associations first
    conn.execute(
        "DELETE FROM entry_groups WHERE group_id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("Failed to delete associations: {}", e))?;

    // Delete the group
    let rows = conn
        .execute("DELETE FROM groups WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| format!("Failed to delete group: {}", e))?;

    Ok(rows > 0)
}

/// Add an entry to a group
#[tauri::command]
pub fn add_entry_to_group(
    app_handle: AppHandle,
    entry_id: i64,
    group_id: i64,
) -> Result<bool, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute(
        "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        rusqlite::params![entry_id, group_id],
    )
    .map_err(|e| format!("Failed to add entry to group: {}", e))?;

    Ok(true)
}

/// Remove an entry from a group
#[tauri::command]
pub fn remove_entry_from_group(
    app_handle: AppHandle,
    entry_id: i64,
    group_id: i64,
) -> Result<bool, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let rows = conn
        .execute(
            "DELETE FROM entry_groups WHERE entry_id = ?1 AND group_id = ?2",
            rusqlite::params![entry_id, group_id],
        )
        .map_err(|e| format!("Failed to remove entry from group: {}", e))?;

    Ok(rows > 0)
}

/// Get all entries in a group
#[tauri::command]
pub fn get_group_entries(app_handle: AppHandle, group_id: i64) -> Result<Vec<Entry>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
                   e.description, e.icon_location, e.icon_index,
                   e.tags, e.notes, e.frequency, e.last_opened,
                   e.created_at, e.updated_at, e.expires_at
            FROM entries e
            INNER JOIN entry_groups eg ON e.id = eg.entry_id
            WHERE eg.group_id = ?1
            ORDER BY e.frequency DESC
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let entries = stmt
        .query_map(rusqlite::params![group_id], Entry::from_row)
        .map_err(|e| format!("Failed to query entries: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect entries: {}", e))?;

    Ok(entries)
}

/// Get all groups for an entry
#[tauri::command]
pub fn get_entry_groups(
    app_handle: AppHandle,
    entry_id: i64,
) -> Result<Vec<GroupResponse>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT g.id, g.name, g.color, g.created_at, g.updated_at
            FROM groups g
            INNER JOIN entry_groups eg ON g.id = eg.group_id
            WHERE eg.entry_id = ?1
            ORDER BY g.name ASC
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let groups = stmt
        .query_map(rusqlite::params![entry_id], |row| {
            Ok(GroupResponse {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to query groups: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect groups: {}", e))?;

    Ok(groups)
}

/// Group export response
#[derive(Debug, serde::Serialize)]
pub struct GroupExportResponse {
    /// Group data
    pub group: GroupResponse,
    /// Entry IDs in this group
    pub entry_ids: Vec<i64>,
    /// Export timestamp
    pub exported_at: i64,
}

/// Export a group as JSON
#[tauri::command]
pub fn export_group(
    app_handle: AppHandle,
    group_id: i64,
) -> Result<Option<GroupExportResponse>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Get group
    let group = get_group(app_handle.clone(), group_id)?;
    if group.is_none() {
        return Ok(None);
    }

    // Get entry IDs
    let mut stmt = conn
        .prepare("SELECT entry_id FROM entry_groups WHERE group_id = ?1")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let entry_ids: Vec<i64> = stmt
        .query_map(rusqlite::params![group_id], |row| row.get(0))
        .map_err(|e| format!("Failed to query entry IDs: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect entry IDs: {}", e))?;

    Ok(Some(GroupExportResponse {
        group: group.unwrap().group,
        entry_ids,
        exported_at: chrono::Utc::now().timestamp(),
    }))
}

/// Import a group from JSON
#[tauri::command]
pub fn import_group(
    app_handle: AppHandle,
    name: String,
    color: String,
    entry_ids: Vec<i64>,
) -> Result<GroupResponse, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Create the group
    let group = create_group(app_handle.clone(), name, color)?;

    // Associate entries (skip non-existent)
    for entry_id in entry_ids {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM entries WHERE id = ?1",
                rusqlite::params![entry_id],
                |_| Ok(true),
            )
            .optional()
            .map_err(|e| format!("Failed to check entry: {}", e))?
            .is_some();

        if exists && group.id.is_some() {
            conn.execute(
                "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
                rusqlite::params![entry_id, group.id],
            )
            .map_err(|e| format!("Failed to add entry to group: {}", e))?;
        }
    }

    Ok(group)
}

/// Batch add entries to a group
#[tauri::command]
pub fn batch_add_to_group(
    app_handle: AppHandle,
    entry_ids: Vec<i64>,
    group_id: i64,
) -> Result<usize, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut count = 0;
    for entry_id in entry_ids {
        let rows = conn
            .execute(
                "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
                rusqlite::params![entry_id, group_id],
            )
            .map_err(|e| format!("Failed to add entry to group: {}", e))?;
        if rows > 0 {
            count += 1;
        }
    }

    Ok(count)
}

/// Batch remove entries from a group
#[tauri::command]
pub fn batch_remove_from_group(
    app_handle: AppHandle,
    entry_ids: Vec<i64>,
    group_id: i64,
) -> Result<usize, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut count = 0;
    for entry_id in entry_ids {
        let rows = conn
            .execute(
                "DELETE FROM entry_groups WHERE entry_id = ?1 AND group_id = ?2",
                rusqlite::params![entry_id, group_id],
            )
            .map_err(|e| format!("Failed to remove entry from group: {}", e))?;
        if rows > 0 {
            count += 1;
        }
    }

    Ok(count)
}

// ============================================================================
// Entry Commands
// ============================================================================

/// Get an entry by ID
#[tauri::command]
pub fn get_entry(app_handle: AppHandle, id: i64) -> Result<Option<Entry>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, lnk_path, target_path, parameters, working_dir,
                   description, icon_location, icon_index,
                   tags, notes, frequency, last_opened,
                   created_at, updated_at, expires_at
            FROM entries WHERE id = ?1
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let result = stmt
        .query_row(rusqlite::params![id], Entry::from_row)
        .optional()
        .map_err(|e| format!("Failed to query entry: {}", e))?;

    Ok(result)
}

/// Create a new entry
#[tauri::command]
pub fn create_entry(app_handle: AppHandle, entry: Entry) -> Result<Entry, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    // Convert empty lnk_path to NULL (allows saving entries without a .lnk file)
    let lnk_path_val: Option<&str> = if entry.lnk_path.is_empty() {
        None
    } else {
        Some(&entry.lnk_path)
    };
    conn.execute(
        r#"
        INSERT INTO entries (lnk_path, target_path, parameters, working_dir,
                             description, icon_location, icon_index,
                             tags, notes, frequency, last_opened,
                             created_at, updated_at, expires_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        "#,
        rusqlite::params![
            lnk_path_val,
            entry.target_path,
            entry.parameters,
            entry.working_dir,
            entry.description,
            entry.icon_location,
            entry.icon_index,
            entry.tags,
            entry.notes,
            entry.frequency,
            entry.last_opened,
            now,
            now,
            entry.expires_at,
        ],
    )
    .map_err(|e| format!("Failed to create entry: {}", e))?;

    let id = conn.last_insert_rowid();

    let mut created_entry = entry;
    created_entry.id = Some(id);
    created_entry.created_at = now;
    created_entry.updated_at = now;
    Ok(created_entry)
}

/// Update an existing entry
#[tauri::command]
pub fn update_entry(app_handle: AppHandle, id: i64, entry: Entry) -> Result<Entry, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    // Convert empty lnk_path to NULL (allows saving entries without a .lnk file)
    let lnk_path_val: Option<&str> = if entry.lnk_path.is_empty() {
        None
    } else {
        Some(&entry.lnk_path)
    };
    conn.execute(
        r#"
        UPDATE entries SET
            lnk_path = ?1, target_path = ?2, parameters = ?3, working_dir = ?4,
            description = ?5, icon_location = ?6, icon_index = ?7,
            tags = ?8, notes = ?9, frequency = ?10, last_opened = ?11,
            updated_at = ?12, expires_at = ?13
        WHERE id = ?14
        "#,
        rusqlite::params![
            lnk_path_val,
            entry.target_path,
            entry.parameters,
            entry.working_dir,
            entry.description,
            entry.icon_location,
            entry.icon_index,
            entry.tags,
            entry.notes,
            entry.frequency,
            entry.last_opened,
            now,
            entry.expires_at,
            id,
        ],
    )
    .map_err(|e| format!("Failed to update entry: {}", e))?;

    let mut updated_entry = entry;
    updated_entry.id = Some(id);
    updated_entry.updated_at = now;
    Ok(updated_entry)
}

/// Delete an entry by ID
#[tauri::command]
pub fn delete_entry(app_handle: AppHandle, id: i64) -> Result<(), String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute("DELETE FROM entries WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| format!("Failed to delete entry: {}", e))?;

    Ok(())
}

/// Get all entries
#[tauri::command]
pub fn get_all_entries(app_handle: AppHandle) -> Result<Vec<Entry>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, lnk_path, target_path, parameters, working_dir,
                   description, icon_location, icon_index,
                   tags, notes, frequency, last_opened,
                   created_at, updated_at, expires_at
            FROM entries
            ORDER BY updated_at DESC
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let entries = stmt
        .query_map([], Entry::from_row)
        .map_err(|e| format!("Failed to query entries: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect entries: {}", e))?;

    Ok(entries)
}

/// Paginated search response
#[derive(Debug, serde::Serialize)]
pub struct PaginatedEntries {
    /// Search results
    pub results: Vec<Entry>,
    /// Total count of matching entries
    pub total_count: i64,
    /// Current offset
    pub offset: i64,
    /// Page limit
    pub limit: i64,
}

/// Open a file or folder using the default application.
/// Accepts either a .lnk file path or a direct target path (for entries without a .lnk file).
#[tauri::command]
pub fn open_lnk_file(path: String) -> Result<(), String> {
    if path.is_empty() {
        return Err("Cannot open: path is empty".to_string());
    }
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &path])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(())
}

/// Parameters describing how to open an entry. Used by `open_entry`.
/// For application mode, `parameters` is the raw command-line argument string
/// (or empty). For file/folder modes, `parameters` is a JSON string with
/// shape: {"openMethod": "explorer"|"app"|"custom", "app": "<path>", "customCommand": "<fmt>"}
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenEntryParams {
    pub entry_id: Option<i64>,
    pub lnk_path: String,
    pub target_path: String,
    /// "File" | "Folder" | "Url" | "Unknown"
    pub target_type: String,
    pub parameters: Option<String>,
    pub working_dir: Option<String>,
}

/// Parsed file/folder open-method configuration stored in `Entry::parameters`.
/// Field names use camelCase to match the JSON shape produced by the frontend
/// (see AddEntryModal.buildParametersJson): {"openMethod", "app", "customCommand"}.
#[derive(Debug)]
struct FileFolderOpenConfig {
    open_method: String,
    app: String,
    custom_command: String,
}

/// Manual serde deserialization mapping camelCase JSON keys to snake_case Rust
/// fields, so the frontend JSON shape ({openMethod, app, customCommand}) is
/// accepted.
impl<'de> serde::Deserialize<'de> for FileFolderOpenConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Helper {
            open_method: String,
            #[serde(default)]
            app: String,
            #[serde(default)]
            custom_command: String,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(FileFolderOpenConfig {
            open_method: h.open_method,
            app: h.app,
            custom_command: h.custom_command,
        })
    }
}

/// Open an entry according to its mode:
/// - Application: launch target_path with parameters (raw arguments) in working_dir
/// - File/Folder: parse parameters JSON for openMethod (explorer | app | custom)
///
/// Also increments frequency and updates last_opened if entry_id is provided.
#[tauri::command]
pub fn open_entry(app_handle: AppHandle, params: OpenEntryParams) -> Result<(), String> {
    // If we have a .lnk file, the simplest and most faithful approach is to
    // launch it directly — Windows Shell resolves target, arguments, working
    // directory, icon, etc. from the .lnk itself.
    if !params.lnk_path.is_empty() {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &params.lnk_path])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to open LNK file: {}", e))?;
        increment_open_stats(&app_handle, params.entry_id)?;
        return Ok(());
    }

    if params.target_path.is_empty() {
        return Err("Cannot open entry: target path is empty".to_string());
    }

    // Determine if this is file/folder mode by trying to parse parameters as
    // the FileFolderOpenConfig JSON. If it parses and contains an open_method
    // field, treat as file/folder mode. Otherwise, treat as application mode
    // (parameters is raw command-line arguments).
    let is_folder = params.target_type.eq_ignore_ascii_case("Folder");
    let file_folder_cfg = params
        .parameters
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(|s| serde_json::from_str::<FileFolderOpenConfig>(s).ok());

    if let Some(cfg) = file_folder_cfg {
        // File/Folder mode
        match cfg.open_method.as_str() {
            "explorer" => {
                std::process::Command::new("cmd")
                    .args(["/C", "start", "", &params.target_path])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .spawn()
                    .map_err(|e| format!("Failed to open: {}", e))?;
            }
            "app" => {
                if cfg.app.is_empty() {
                    return Err("Open with application: no application specified".to_string());
                }
                std::process::Command::new(&cfg.app)
                    .arg(&params.target_path)
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .spawn()
                    .map_err(|e| format!("Failed to launch application: {}", e))?;
            }
            "custom" => {
                if cfg.custom_command.is_empty() {
                    return Err("Custom command: no command specified".to_string());
                }
                run_custom_command(&cfg.custom_command, &params.target_path, is_folder)?;
            }
            other => {
                return Err(format!("Unknown open method: {}", other));
            }
        }
    } else {
        // Application mode: target_path is the executable; parameters are raw
        // command-line arguments (may be empty).
        let mut cmd = std::process::Command::new(&params.target_path);
        if let Some(args) = params.parameters.as_deref().filter(|s| !s.is_empty()) {
            cmd.raw_arg(args);
        }
        if let Some(wd) = params.working_dir.as_deref().filter(|s| !s.is_empty()) {
            cmd.current_dir(wd);
        }
        cmd.creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to launch application: {}", e))?;
    }

    increment_open_stats(&app_handle, params.entry_id)?;
    Ok(())
}

/// Execute a custom command for file/folder entries.
/// The command format uses `_` as a placeholder for the entry path.
/// - For folders, the command runs after `cd <folder>` in PowerShell.
/// - For cmd/powershell commands, runs directly.
/// - For other apps, PowerShell runs `<app> <command>`.
fn run_custom_command(command: &str, target_path: &str, is_folder: bool) -> Result<(), String> {
    let resolved = command.replace('_', target_path);
    let ps_script = if is_folder {
        format!("cd '{}'; {}", target_path, resolved)
    } else {
        resolved
    };
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Failed to run custom command: {}", e))?;
    Ok(())
}

/// Increment an entry's frequency and update last_opened timestamp.
/// Silently no-ops if entry_id is None or the database is unavailable,
/// so a stats update failure never blocks opening the entry.
fn increment_open_stats(app_handle: &AppHandle, entry_id: Option<i64>) -> Result<(), String> {
    let id = match entry_id {
        Some(id) => id,
        None => return Ok(()),
    };
    let db_path = db::get_database_path(app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE entries SET frequency = frequency + 1, last_opened = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![now, now, id],
    )
    .map_err(|e| format!("Failed to update entry stats: {}", e))?;
    Ok(())
}

/// Open an external URL in the system's default browser.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if url.is_empty() {
        return Err("Cannot open: URL is empty".to_string());
    }
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &url])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

/// Open the parent directory of a path in Windows File Explorer.
/// If the path is a folder, opens that folder directly. If it is a file,
/// opens the containing folder with the file selected.
#[tauri::command]
pub fn open_working_directory(path: String) -> Result<(), String> {
    if path.is_empty() {
        return Err("Cannot open working directory: path is empty".to_string());
    }
    let p = std::path::Path::new(&path);
    let (dir, file_name) = if p.is_dir() {
        (path.clone(), None)
    } else {
        let parent = p
            .parent()
            .ok_or_else(|| "Cannot resolve parent directory".to_string())?;
        (parent.to_string_lossy().to_string(), p.file_name().map(|n| n.to_string_lossy().to_string()))
    };

    let mut args = vec!["/C".to_string(), "explorer".to_string()];
    if let Some(name) = &file_name {
        args.push("/select,".to_string());
        // build "<dir>\<file>" as a single argument for /select
        let full = std::path::Path::new(&dir).join(name);
        args.push(full.to_string_lossy().to_string());
    } else {
        args.push(dir);
    }

    std::process::Command::new("cmd")
        .args(&args)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Failed to open working directory: {}", e))?;
    Ok(())
}

/// Payload for batch entry creation. Each item mirrors the fields a single
/// `create_entry` would accept, minus id/timestamps (assigned by the backend).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BatchEntryInput {
    pub lnk_path: String,
    pub target_path: String,
    /// Sent by the frontend for validation; not stored in the DB (type is
    /// auto-inferred at open time via LnkTarget::from_path).
    #[allow(dead_code)]
    pub target_type: String,
    pub parameters: Option<String>,
    pub working_dir: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
}

/// Result of a single batch-create item.
#[derive(Debug, serde::Serialize)]
pub struct BatchCreateResult {
    pub success: bool,
    pub error: Option<String>,
    pub entry_id: Option<i64>,
    pub target_path: String,
}

/// Create multiple entries in a single transaction. Duplicates (by target_path)
/// are skipped with an error message. Progress is reported via Tauri events
/// so the frontend can show a progress bar for large batches.
#[tauri::command]
pub async fn batch_create_entries(
    app_handle: AppHandle,
    entries: Vec<BatchEntryInput>,
) -> Result<Vec<BatchCreateResult>, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Enable foreign keys (consistent with init_database)
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

    // Use WAL mode for better concurrent read performance during writes.
    // PRAGMA journal_mode returns a row (the new mode), so use query_row not execute.
    let _: String = conn
        .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
        .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    let total = entries.len();
    let mut results = Vec::with_capacity(total);

    // Pre-load all existing target_paths for O(1) duplicate lookup.
    // This is much faster than querying per-entry during the loop.
    let mut existing_paths: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT target_path FROM entries")
            .map_err(|e| format!("Failed to prepare duplicate check: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                Ok(path.to_lowercase())
            })
            .map_err(|e| format!("Failed to query existing paths: {}", e))?;
        let mut set = std::collections::HashSet::new();
        for row in rows {
            if let Ok(p) = row {
                set.insert(p);
            }
        }
        set
    };

    // Use a transaction for atomicity and performance (all inserts commit together)
    let tx = conn.transaction()
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    for (index, input) in entries.into_iter().enumerate() {
        let target_path = input.target_path.clone();
        let target_path_lower = target_path.to_lowercase();

        // Check for duplicates (case-insensitive on Windows)
        if existing_paths.contains(&target_path_lower) {
            // Emit progress event before moving target_path into result
            let _ = app_handle.emit("batch-import-progress", serde_json::json!({
                "current": index + 1,
                "total": total,
                "target_path": &target_path,
                "status": "duplicate"
            }));
            results.push(BatchCreateResult {
                success: false,
                error: Some(format!("目标路径已存在，跳过重复条目: {}", target_path)),
                entry_id: None,
                target_path,
            });
            continue;
        }

        let lnk_path_val: Option<&str> = if input.lnk_path.is_empty() {
            None
        } else {
            Some(&input.lnk_path)
        };

        let insert_result = tx.execute(
            r#"
            INSERT INTO entries (lnk_path, target_path, parameters, working_dir,
                                 description, icon_location, icon_index,
                                 tags, notes, frequency, last_opened,
                                 created_at, updated_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL, ?6, ?7, 0, NULL, ?8, ?9, NULL)
            "#,
            rusqlite::params![
                lnk_path_val,
                input.target_path,
                input.parameters,
                input.working_dir,
                input.description,
                input.tags,
                input.notes,
                now,
                now,
            ],
        );

        match insert_result {
            Ok(_) => {
                let id = tx.last_insert_rowid();
                // Track the new path to prevent duplicates within the same batch
                existing_paths.insert(target_path_lower);
                // Emit progress event before moving target_path into result
                let _ = app_handle.emit("batch-import-progress", serde_json::json!({
                    "current": index + 1,
                    "total": total,
                    "target_path": &target_path,
                    "status": "done"
                }));
                results.push(BatchCreateResult {
                    success: true,
                    error: None,
                    entry_id: Some(id),
                    target_path,
                });
            }
            Err(e) => {
                // Emit progress event before moving target_path into result
                let _ = app_handle.emit("batch-import-progress", serde_json::json!({
                    "current": index + 1,
                    "total": total,
                    "target_path": &target_path,
                    "status": "error"
                }));
                results.push(BatchCreateResult {
                    success: false,
                    error: Some(format!("插入失败: {}", e)),
                    entry_id: None,
                    target_path,
                });
            }
        }
    }

    // Commit all inserts atomically
    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    // Emit completion event
    let _ = app_handle.emit("batch-import-complete", serde_json::json!({
        "total": total,
        "success": results.iter().filter(|r| r.success).count(),
        "failed": results.iter().filter(|r| !r.success).count(),
    }));

    Ok(results)
}

/// Rebuild the FTS5 full-text search index from the entries table.
/// This is a maintenance command useful when the FTS index becomes out of sync
/// with the entries table (e.g., after database corruption, schema migration,
/// or if triggers were missing/not firing correctly).
#[tauri::command]
pub fn rebuild_fts_index(app_handle: AppHandle) -> Result<usize, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Drop existing triggers (they will be recreated by init_database on next startup,
    // but we recreate them here too for immediate effect)
    conn.execute("DROP TRIGGER IF EXISTS entries_ai", [])
        .map_err(|e| format!("Failed to drop trigger entries_ai: {}", e))?;
    conn.execute("DROP TRIGGER IF EXISTS entries_ad", [])
        .map_err(|e| format!("Failed to drop trigger entries_ad: {}", e))?;
    conn.execute("DROP TRIGGER IF EXISTS entries_au", [])
        .map_err(|e| format!("Failed to drop trigger entries_au: {}", e))?;

    // Drop and recreate the FTS table
    conn.execute("DROP TABLE IF EXISTS entries_fts", [])
        .map_err(|e| format!("Failed to drop entries_fts: {}", e))?;

    conn.execute(
        r#"
        CREATE VIRTUAL TABLE entries_fts USING fts5(
            lnk_path,
            target_path,
            description,
            tags,
            notes,
            content='entries',
            content_rowid='id'
        )
        "#,
        [],
    )
    .map_err(|e| format!("Failed to recreate entries_fts: {}", e))?;

    // Rebuild the FTS index from existing entries
    let count: usize = conn
        .execute(
            r#"
            INSERT INTO entries_fts(rowid, lnk_path, target_path, description, tags, notes)
            SELECT id, lnk_path, target_path, description, tags, notes FROM entries
            "#,
            [],
        )
        .map_err(|e| format!("Failed to rebuild FTS index: {}", e))?;

    // Recreate triggers
    conn.execute(
        r#"
        CREATE TRIGGER entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, lnk_path, target_path, description, tags, notes)
            VALUES (new.id, new.lnk_path, new.target_path, new.description, new.tags, new.notes)
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create trigger entries_ai: {}", e))?;

    conn.execute(
        r#"
        CREATE TRIGGER entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, description, tags, notes)
            VALUES ('delete', old.id, old.lnk_path, old.target_path, old.description, old.tags, old.notes)
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create trigger entries_ad: {}", e))?;

    conn.execute(
        r#"
        CREATE TRIGGER entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, lnk_path, target_path, description, tags, notes)
            VALUES ('delete', old.id, old.lnk_path, old.target_path, old.description, old.tags, old.notes);
            INSERT INTO entries_fts(rowid, lnk_path, target_path, description, tags, notes)
            VALUES (new.id, new.lnk_path, new.target_path, new.description, new.tags, new.notes)
        END
        "#,
        [],
    )
    .map_err(|e| format!("Failed to create trigger entries_au: {}", e))?;

    log::info!("FTS index rebuilt: {} entries indexed", count);
    Ok(count)
}

/// Search entries by query with pagination.
///
/// The query is sanitized for FTS5 MATCH syntax: each whitespace-separated
/// token is treated as a prefix search (token*), and special FTS5 characters
/// are escaped. This allows searching for file paths, names with dots, etc.
#[tauri::command]
pub fn search_entries(
    app_handle: AppHandle,
    query: String,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<PaginatedEntries, String> {
    let db_path = db::get_database_path(&app_handle)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let offset_val = offset.unwrap_or(0).max(0);
    let limit_val = limit.unwrap_or(50).clamp(1, 100);

    // Sanitize the query for FTS5 MATCH syntax.
    // Split by whitespace, wrap each token in double quotes (escaping internal
    // quotes), and append * for prefix matching. This handles special characters
    // like . \ / - * ( ) : that FTS5 would otherwise treat as operators.
    let fts_query: String = query
        .split_whitespace()
        .map(|token| {
            let escaped = token.replace('"', "\"\"");
            format!("\"{}\"*", escaped)
        })
        .collect::<Vec<_>>()
        .join(" ");

    // If the sanitized query is empty, return no results
    if fts_query.is_empty() {
        return Ok(PaginatedEntries {
            results: Vec::new(),
            total_count: 0,
            offset: offset_val,
            limit: limit_val,
        });
    }

    // First get total count
    let total_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH ?1",
            rusqlite::params![&fts_query],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Note: bm25() in SQLite FTS5 returns lower (more negative) values for BETTER
    // matches. Using ASC puts the best matches first. The previous DESC ordering
    // caused the best matches to appear last (or be cut off by pagination).
    //
    // IMPORTANT: bm25() and MATCH must reference the FTS5 table by its real name
    // (entries_fts), not a FROM alias — SQLite FTS5 requires this.
    let mut stmt = conn
        .prepare(
            r#"
            SELECT e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
                   e.description, e.icon_location, e.icon_index,
                   e.tags, e.notes, e.frequency, e.last_opened,
                   e.created_at, e.updated_at, e.expires_at
            FROM entries_fts
            JOIN entries e ON e.id = entries_fts.rowid
            WHERE entries_fts MATCH ?1
            ORDER BY bm25(entries_fts) ASC, e.frequency DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let entries = stmt
        .query_map(rusqlite::params![fts_query, limit_val, offset_val], |row| {
            Entry::from_row(row)
        })
        .map_err(|e| format!("Failed to search entries: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect search results: {}", e))?;

    Ok(PaginatedEntries {
        results: entries,
        total_count,
        offset: offset_val,
        limit: limit_val,
    })
}
