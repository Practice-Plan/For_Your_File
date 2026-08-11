//! Tauri commands for hotkey and protocol management
//!
//! Provides IPC interface between frontend and backend for operations.

use crate::cli::CliArgs;
use crate::db;
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
        .spawn()
        .map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(())
}

/// Search entries by query with pagination
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

    // First get total count
    let total_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH ?1",
            rusqlite::params![query],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn
        .prepare(
            r#"
            SELECT e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
                   e.description, e.icon_location, e.icon_index,
                   e.tags, e.notes, e.frequency, e.last_opened,
                   e.created_at, e.updated_at, e.expires_at
            FROM entries e
            JOIN entries_fts fts ON e.id = fts.rowid
            WHERE entries_fts MATCH ?1
            ORDER BY bm25(entries_fts) DESC, e.frequency DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let entries = stmt
        .query_map(rusqlite::params![query, limit_val, offset_val], |row| {
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
