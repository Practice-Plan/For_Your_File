//! Global hotkey management for Tauri
//!
//! Handles registration, unregistration, and monitoring of global hotkeys
//! that work across all applications on Windows.
//!
//! IMPORTANT: RegisterHotKey and the message loop MUST run on the same thread.
//! When HWND is NULL, WM_HOTKEY messages are posted to the calling thread's
//! queue, so GetMessageW/PeekMessageW must be called from that same thread.
//! This module uses a command channel to send registration requests from
//! any thread to the dedicated listener thread.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

/// Hotkey identifier for Windows API
#[cfg(windows)]
const HOTKEY_ID: i32 = 1;

/// Default hotkey combination
#[allow(dead_code)]
pub const DEFAULT_HOTKEY: &str = "Alt+Space";

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Modifier keys (e.g., "Alt", "Ctrl+Shift", "Alt+Ctrl")
    pub modifiers: String,
    /// Main key (e.g., "Space", "A", "F1")
    pub key: String,
    /// Whether hotkey is currently registered
    pub registered: bool,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: "Alt".to_string(),
            key: "Space".to_string(),
            registered: false,
        }
    }
}

impl HotkeyConfig {
    /// Create a new hotkey config from a string like "Alt+Space"
    pub fn from_string(hotkey_str: &str) -> Result<Self, String> {
        let trimmed = hotkey_str.trim();
        if trimmed.is_empty() {
            return Err("Invalid hotkey format: empty string".to_string());
        }

        let parts: Vec<&str> = trimmed
            .split('+')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            return Err("Invalid hotkey format: no keys".to_string());
        }

        // Last part is the key, rest are modifiers
        let key = parts.last().unwrap().to_string();
        let modifiers = parts[..parts.len() - 1].join("+");

        Ok(Self {
            modifiers,
            key,
            registered: false,
        })
    }

    /// Convert to string representation
    #[allow(dead_code)]
    pub fn to_string_repr(&self) -> String {
        if self.modifiers.is_empty() {
            self.key.clone()
        } else {
            format!("{}+{}", self.modifiers, self.key)
        }
    }
}

/// Commands sent to the listener thread via channel
enum HotkeyCommand {
    /// Register a new hotkey (unregister old first)
    Register(String, String),
    /// Unregister current hotkey
    Unregister,
    /// Check if a hotkey combination conflicts with another app
    CheckConflict(String, String, Sender<Result<bool, String>>),
}

/// Global hotkey manager
pub struct HotkeyManager {
    /// Whether the hotkey listener is running
    running: Arc<AtomicBool>,
    /// Current hotkey configuration (shared with listener thread)
    config: Arc<Mutex<HotkeyConfig>>,
    /// Channel sender for sending commands to the listener thread
    cmd_sender: Mutex<Option<Sender<HotkeyCommand>>>,
    /// Handle to the Tauri app
    #[allow(dead_code)]
    app_handle: Option<AppHandle<tauri::Wry>>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(HotkeyConfig::default())),
            cmd_sender: Mutex::new(None),
            app_handle: None,
        }
    }

    /// Set the app handle for emitting events and config persistence
    pub fn set_app_handle(&mut self, handle: AppHandle<tauri::Wry>) {
        self.app_handle = Some(handle);
    }

    /// Register a global hotkey.
    ///
    /// If the listener thread is running, sends a command to register in that thread.
    /// If not, just stores the config (listener will register when it starts).
    pub fn register(&self, modifiers: &str, key: &str) -> Result<(), String> {
        // Send command to listener thread if running
        let sender = self.cmd_sender.lock().unwrap();
        if let Some(sender) = sender.as_ref() {
            sender
                .send(HotkeyCommand::Register(
                    modifiers.to_string(),
                    key.to_string(),
                ))
                .map_err(|e| format!("Failed to send register command: {}", e))?;
        }

        // Update config
        let mut config = self.config.lock().unwrap();
        config.modifiers = modifiers.to_string();
        config.key = key.to_string();
        config.registered = true;

        log::info!("Hotkey register requested: {} + {}", modifiers, key);
        Ok(())
    }

    /// Unregister the global hotkey.
    pub fn unregister(&self) -> Result<(), String> {
        let sender = self.cmd_sender.lock().unwrap();
        if let Some(sender) = sender.as_ref() {
            let _ = sender.send(HotkeyCommand::Unregister);
        }

        let mut config = self.config.lock().unwrap();
        config.registered = false;

        log::info!("Hotkey unregister requested");
        Ok(())
    }

    /// Update the hotkey to a new combination.
    ///
    /// NOTE: This method must NOT hold the config lock while calling unregister(),
    /// because unregister() also locks config. Otherwise, this causes a deadlock.
    pub fn update_hotkey(&self, modifiers: &str, key: &str) -> Result<(), String> {
        // Check if registered WITHOUT holding the lock
        let is_registered = {
            let config = self.config.lock().unwrap();
            config.registered
        };

        if is_registered {
            self.unregister()?;
        }

        self.register(modifiers, key)?;
        self.save_config()?;

        Ok(())
    }

    /// Check if the current hotkey has a conflict.
    ///
    /// Sends a command to the listener thread and waits for the reply.
    pub fn check_conflict(&self, modifiers: &str, key: &str) -> Result<bool, String> {
        let sender_guard = self.cmd_sender.lock().unwrap();
        if let Some(sender) = sender_guard.as_ref() {
            let (reply_tx, reply_rx) = channel();
            sender
                .send(HotkeyCommand::CheckConflict(
                    modifiers.to_string(),
                    key.to_string(),
                    reply_tx,
                ))
                .map_err(|e| format!("Failed to send check conflict command: {}", e))?;
            // Drop the sender guard before blocking on recv to avoid holding the mutex
            drop(sender_guard);
            reply_rx
                .recv()
                .map_err(|e| format!("Failed to receive conflict check result: {}", e))?
        } else {
            // Listener not running, assume no conflict
            Ok(false)
        }
    }

    /// Start listening for hotkey events.
    ///
    /// This spawns a dedicated thread that:
    /// 1. Registers the initial hotkey (in this thread, so messages come here)
    /// 2. Processes commands from the channel (register/unregister/check_conflict)
    /// 3. Runs a message loop using PeekMessageW (non-blocking)
    pub fn start_listener(&mut self, app_handle: AppHandle<tauri::Wry>) -> Result<(), String> {
        self.app_handle = Some(app_handle.clone());
        self.running.store(true, Ordering::SeqCst);

        let (tx, rx) = channel::<HotkeyCommand>();
        *self.cmd_sender.lock().unwrap() = Some(tx);

        let running = self.running.clone();
        let config_arc = self.config.clone();
        let initial_config = self.config.lock().unwrap().clone();
        let app_handle_clone = self.app_handle.clone();

        thread::spawn(move || {
            #[cfg(windows)]
            {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::Input::KeyboardAndMouse::{
                    RegisterHotKey, UnregisterHotKey,
                };
                use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, MSG, PM_REMOVE, WM_HOTKEY};

                // Register initial hotkey in THIS thread (so messages come to this thread)
                if initial_config.registered {
                    let mod_flags = parse_modifiers(&initial_config.modifiers);
                    match key_to_vk_code(&initial_config.key) {
                        Ok(vk) => {
                            unsafe {
                                if let Err(e) =
                                    RegisterHotKey(HWND::default(), HOTKEY_ID, mod_flags, vk)
                                {
                                    log::error!(
                                        "Failed to register initial hotkey in listener thread: {}",
                                        e
                                    );
                                } else {
                                    log::info!(
                                        "Initial hotkey registered in listener thread: {} + {}",
                                        initial_config.modifiers,
                                        initial_config.key
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Invalid key '{}': {}", initial_config.key, e);
                        }
                    }
                }

                let mut msg = MSG::default();

                while running.load(Ordering::SeqCst) {
                    // Process pending commands (non-blocking)
                    while let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            HotkeyCommand::Register(modifiers, key) => {
                                // Unregister old hotkey first
                                unsafe {
                                    let _ = UnregisterHotKey(HWND::default(), HOTKEY_ID);
                                }
                                let mod_flags = parse_modifiers(&modifiers);
                                match key_to_vk_code(&key) {
                                    Ok(vk) => {
                                        unsafe {
                                            if let Err(e) = RegisterHotKey(
                                                HWND::default(),
                                                HOTKEY_ID,
                                                mod_flags,
                                                vk,
                                            ) {
                                                log::error!("Failed to register hotkey: {}", e);
                                            } else {
                                                log::info!(
                                                    "Hotkey registered: {} + {}",
                                                    modifiers,
                                                    key
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => log::error!("Invalid key '{}': {}", key, e),
                                }
                            }
                            HotkeyCommand::Unregister => {
                                unsafe {
                                    let _ = UnregisterHotKey(HWND::default(), HOTKEY_ID);
                                    log::info!("Hotkey unregistered");
                                }
                            }
                            HotkeyCommand::CheckConflict(modifiers, key, reply) => {
                                // Temporarily unregister our own hotkey to avoid
                                // false conflict with ourselves
                                unsafe {
                                    let _ = UnregisterHotKey(HWND::default(), HOTKEY_ID);
                                }

                                let mod_flags = parse_modifiers(&modifiers);
                                let result = match key_to_vk_code(&key) {
                                    Ok(vk) => {
                                        let register_result = unsafe {
                                            RegisterHotKey(
                                                HWND::default(),
                                                HOTKEY_ID + 1,
                                                mod_flags,
                                                vk,
                                            )
                                        };
                                        if register_result.is_ok() {
                                            unsafe {
                                                let _ =
                                                    UnregisterHotKey(HWND::default(), HOTKEY_ID + 1);
                                            }
                                            Ok(false) // No conflict
                                        } else {
                                            Ok(true) // Conflict
                                        }
                                    }
                                    Err(e) => Err(e),
                                };
                                let _ = reply.send(result);

                                // Re-register our own hotkey from config
                                let config = config_arc.lock().unwrap();
                                if config.registered {
                                    let mod_flags = parse_modifiers(&config.modifiers);
                                    if let Ok(vk) = key_to_vk_code(&config.key) {
                                        unsafe {
                                            let _ = RegisterHotKey(
                                                HWND::default(),
                                                HOTKEY_ID,
                                                mod_flags,
                                                vk,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Peek for window messages (non-blocking)
                    let has_msg =
                        unsafe { PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE) };
                    if has_msg.as_bool() {
                        if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
                            log::info!("Global hotkey pressed!");
                            if let Some(handle) = &app_handle_clone {
                                if let Err(e) = handle.emit("hotkey-pressed", ()) {
                                    log::error!("Failed to emit hotkey event: {}", e);
                                }
                            }
                        }
                    } else {
                        // No message, sleep briefly to avoid busy-loop
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }

                // Cleanup: unregister hotkey before exiting
                unsafe {
                    let _ = UnregisterHotKey(HWND::default(), HOTKEY_ID);
                }
                log::info!("Hotkey listener thread exiting");
            }

            #[cfg(not(windows))]
            {
                let _ = running;
                let _ = rx;
                let _ = app_handle_clone;
                let _ = config_arc;
                log::info!("Hotkey listener running (non-Windows, no-op)");
            }
        });

        log::info!("Hotkey listener started");
        Ok(())
    }

    /// Stop listening for hotkey events
    #[allow(dead_code)]
    pub fn stop_listener(&self) {
        self.running.store(false, Ordering::SeqCst);
        log::info!("Hotkey listener stopped");
    }

    /// Get current hotkey configuration
    pub fn get_config(&self) -> HotkeyConfig {
        self.config.lock().unwrap().clone()
    }

    /// Save configuration to a JSON file in the app data directory.
    ///
    /// Called automatically by `update_hotkey` whenever the user changes
    /// the hotkey. Uses `self.app_handle` (set by `start_listener`).
    fn save_config(&self) -> Result<(), String> {
        let app_handle = self
            .app_handle
            .as_ref()
            .ok_or_else(|| "AppHandle not set (start_listener not called yet)".to_string())?;

        let config = self.config.lock().unwrap();
        let json = serde_json::to_string_pretty(&*config)
            .map_err(|e| format!("Failed to serialize hotkey config: {}", e))?;

        let config_path = get_config_path(app_handle)?;
        std::fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write hotkey config file: {}", e))?;

        log::info!("Hotkey config saved to {}", config_path.display());
        Ok(())
    }

    /// Load configuration from a JSON file in the app data directory.
    ///
    /// Called at startup (before `start_listener`), so the `app_handle`
    /// must be passed as a parameter.
    pub fn load_config(&self, app_handle: &AppHandle<tauri::Wry>) -> Result<HotkeyConfig, String> {
        let config_path = get_config_path(app_handle)?;

        if !config_path.exists() {
            log::info!("Hotkey config file not found, using default");
            return Ok(HotkeyConfig::default());
        }

        let json = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read hotkey config file: {}", e))?;

        let config: HotkeyConfig = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse hotkey config: {}", e))?;

        log::info!("Hotkey config loaded from {}", config_path.display());
        Ok(config)
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the hotkey config file path inside the app data directory.
///
/// The config is stored as `hotkey_config.json` in the app data directory
/// (same directory as the SQLite database).
fn get_config_path(app_handle: &AppHandle<tauri::Wry>) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    Ok(app_data_dir.join("hotkey_config.json"))
}

// ============================================================================
// Standalone helper functions (used by the listener thread)
// ============================================================================

/// Parse modifier string (e.g., "Alt+Ctrl") to Windows HOT_KEY_MODIFIERS flags
#[cfg(windows)]
fn parse_modifiers(modifiers: &str) -> windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS {
    use windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS;

    let mut mod_flags = HOT_KEY_MODIFIERS(0);
    for mod_str in modifiers.split('+').map(|s| s.trim().to_uppercase()) {
        match mod_str.as_str() {
            "ALT" => mod_flags |= HOT_KEY_MODIFIERS(1), // MOD_ALT
            "CTRL" | "CONTROL" => mod_flags |= HOT_KEY_MODIFIERS(2), // MOD_CONTROL
            "SHIFT" => mod_flags |= HOT_KEY_MODIFIERS(4), // MOD_SHIFT
            "WIN" | "WINDOWS" => mod_flags |= HOT_KEY_MODIFIERS(8), // MOD_WIN
            _ => log::warn!("Unknown modifier: {}", mod_str),
        }
    }
    mod_flags
}

/// Convert key string to Windows virtual key code
#[cfg(windows)]
fn key_to_vk_code(key: &str) -> Result<u32, String> {
    let key_upper = key.to_uppercase();

    let code = match key_upper.as_str() {
        // Special keys
        "SPACE" => 0x20,
        "ENTER" | "RETURN" => 0x0D,
        "TAB" => 0x09,
        "ESCAPE" | "ESC" => 0x1B,
        "BACKSPACE" | "BACK" => 0x08,
        "DELETE" | "DEL" => 0x2E,
        "INSERT" | "INS" => 0x2D,
        "HOME" => 0x24,
        "END" => 0x23,
        "PAGEUP" | "PAGE_UP" => 0x21,
        "PAGEDOWN" | "PAGE_DOWN" => 0x22,

        // Arrow keys
        "UP" | "ARROWUP" => 0x26,
        "DOWN" | "ARROWDOWN" => 0x28,
        "LEFT" | "ARROWLEFT" => 0x25,
        "RIGHT" | "ARROWRIGHT" => 0x27,

        // Function keys
        "F1" => 0x70,
        "F2" => 0x71,
        "F3" => 0x72,
        "F4" => 0x73,
        "F5" => 0x74,
        "F6" => 0x75,
        "F7" => 0x76,
        "F8" => 0x77,
        "F9" => 0x78,
        "F10" => 0x79,
        "F11" => 0x7A,
        "F12" => 0x7B,

        // Letter keys (A-Z)
        "A" => 0x41,
        "B" => 0x42,
        "C" => 0x43,
        "D" => 0x44,
        "E" => 0x45,
        "F" => 0x46,
        "G" => 0x47,
        "H" => 0x48,
        "I" => 0x49,
        "J" => 0x4A,
        "K" => 0x4B,
        "L" => 0x4C,
        "M" => 0x4D,
        "N" => 0x4E,
        "O" => 0x4F,
        "P" => 0x50,
        "Q" => 0x51,
        "R" => 0x52,
        "S" => 0x53,
        "T" => 0x54,
        "U" => 0x55,
        "V" => 0x56,
        "W" => 0x57,
        "X" => 0x58,
        "Y" => 0x59,
        "Z" => 0x5A,

        // Number keys (0-9)
        "0" => 0x30,
        "1" => 0x31,
        "2" => 0x32,
        "3" => 0x33,
        "4" => 0x34,
        "5" => 0x35,
        "6" => 0x36,
        "7" => 0x37,
        "8" => 0x38,
        "9" => 0x39,

        // Numpad keys
        "NUMPAD0" | "NUM0" => 0x60,
        "NUMPAD1" | "NUM1" => 0x61,
        "NUMPAD2" | "NUM2" => 0x62,
        "NUMPAD3" | "NUM3" => 0x63,
        "NUMPAD4" | "NUM4" => 0x64,
        "NUMPAD5" | "NUM5" => 0x65,
        "NUMPAD6" | "NUM6" => 0x66,
        "NUMPAD7" | "NUM7" => 0x67,
        "NUMPAD8" | "NUM8" => 0x68,
        "NUMPAD9" | "NUM9" => 0x69,

        // Other common keys
        "MULTIPLY" | "*" => 0x6A,
        "ADD" | "+" => 0x6B,
        "SUBTRACT" | "-" => 0x6D,
        "DECIMAL" | "." => 0x6E,
        "DIVIDE" | "/" => 0x6F,

        _ => {
            return Err(format!("Unknown key: {}", key));
        }
    };

    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_config_from_string() {
        let config = HotkeyConfig::from_string("Alt+Space").unwrap();
        assert_eq!(config.modifiers, "Alt");
        assert_eq!(config.key, "Space");

        let config = HotkeyConfig::from_string("Ctrl+Shift+A").unwrap();
        assert_eq!(config.modifiers, "Ctrl+Shift");
        assert_eq!(config.key, "A");

        let config = HotkeyConfig::from_string("F1").unwrap();
        assert_eq!(config.modifiers, "");
        assert_eq!(config.key, "F1");
    }

    #[test]
    fn test_hotkey_config_to_string() {
        let config = HotkeyConfig {
            modifiers: "Alt".to_string(),
            key: "Space".to_string(),
            registered: false,
        };
        assert_eq!(config.to_string_repr(), "Alt+Space");

        let config = HotkeyConfig {
            modifiers: "".to_string(),
            key: "F1".to_string(),
            registered: false,
        };
        assert_eq!(config.to_string_repr(), "F1");
    }
}
