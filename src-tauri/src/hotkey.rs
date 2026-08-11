//! Global hotkey management for Tauri
//!
//! Handles registration, unregistration, and monitoring of global hotkeys
//! that work across all applications on Windows.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

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
        let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return Err("Invalid hotkey format".to_string());
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

/// Global hotkey manager
pub struct HotkeyManager {
    /// Whether the hotkey listener is running
    running: Arc<AtomicBool>,
    /// Current hotkey configuration
    config: Arc<Mutex<HotkeyConfig>>,
    /// Handle to the Tauri app
    app_handle: Option<AppHandle<tauri::Wry>>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(HotkeyConfig::default())),
            app_handle: None,
        }
    }

    /// Set the app handle for emitting events
    #[allow(dead_code)]
    pub fn set_app_handle(&mut self, handle: AppHandle<tauri::Wry>) {
        self.app_handle = Some(handle);
    }

    /// Register a global hotkey
    ///
    /// # Arguments
    /// * `modifiers` - Modifier keys like "Alt", "Ctrl+Shift", etc.
    /// * `key` - The main key like "Space", "A", "F1", etc.
    ///
    /// # Returns
    /// * `Ok(())` if registration successful
    /// * `Err` if registration failed or hotkey already registered by another app
    #[cfg(windows)]
    pub fn register(&self, modifiers: &str, key: &str) -> Result<(), String> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            RegisterHotKey, HOT_KEY_MODIFIERS,
        };

        // Parse modifiers
        let mut mod_flags = HOT_KEY_MODIFIERS(0);
        for mod_str in modifiers.split('+').map(|s| s.trim().to_uppercase()) {
            match mod_str.as_str() {
                "ALT" => mod_flags |= HOT_KEY_MODIFIERS(1),      // MOD_ALT
                "CTRL" | "CONTROL" => mod_flags |= HOT_KEY_MODIFIERS(2), // MOD_CONTROL
                "SHIFT" => mod_flags |= HOT_KEY_MODIFIERS(4),    // MOD_SHIFT
                "WIN" | "WINDOWS" => mod_flags |= HOT_KEY_MODIFIERS(8), // MOD_WIN
                _ => log::warn!("Unknown modifier: {}", mod_str),
            }
        }

        // Convert key to virtual key code
        let vk_code = self.key_to_vk_code(key)?;

        // Attempt to register the hotkey
        let result = unsafe { RegisterHotKey(HWND::default(), HOTKEY_ID, mod_flags, vk_code) };

        match result {
            Ok(_) => {
                log::info!("Global hotkey registered: {} + {}", modifiers, key);

                // Update config
                let mut config = self.config.lock().unwrap();
                config.modifiers = modifiers.to_string();
                config.key = key.to_string();
                config.registered = true;

                Ok(())
            }
            Err(e) => {
                log::error!("Failed to register hotkey: {}", e);

                // Check if it's a conflict
                let error_msg = format!("{}", e);
                if error_msg.contains("already registered") || error_msg.contains("conflict") {
                    Err(format!(
                        "Hotkey {}+{} is already registered by another application. \
                         Please choose a different hotkey.",
                        modifiers, key
                    ))
                } else {
                    Err(format!("Failed to register hotkey: {}", e))
                }
            }
        }
    }

    /// Register hotkey (non-Windows placeholder)
    #[cfg(not(windows))]
    pub fn register(&self, _modifiers: &str, _key: &str) -> Result<(), String> {
        Err("Global hotkeys are only supported on Windows".to_string())
    }

    /// Unregister the global hotkey
    #[cfg(windows)]
    pub fn unregister(&self) -> Result<(), String> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey;

        let result = unsafe { UnregisterHotKey(HWND::default(), HOTKEY_ID) };

        match result {
            Ok(_) => {
                log::info!("Global hotkey unregistered");

                // Update config
                let mut config = self.config.lock().unwrap();
                config.registered = false;

                Ok(())
            }
            Err(e) => {
                log::error!("Failed to unregister hotkey: {}", e);
                Err(format!("Failed to unregister hotkey: {}", e))
            }
        }
    }

    /// Unregister hotkey (non-Windows placeholder)
    #[cfg(not(windows))]
    pub fn unregister(&self) -> Result<(), String> {
        Err("Global hotkeys are only supported on Windows".to_string())
    }

    /// Update the hotkey to a new combination
    pub fn update_hotkey(&self, modifiers: &str, key: &str) -> Result<(), String> {
        // Unregister old hotkey if registered
        {
            let config = self.config.lock().unwrap();
            if config.registered {
                self.unregister()?;
            }
        }

        // Register new hotkey
        self.register(modifiers, key)?;

        // Save to configuration
        self.save_config()?;

        Ok(())
    }

    /// Check if the current hotkey has a conflict
    #[cfg(windows)]
    pub fn check_conflict(&self, modifiers: &str, key: &str) -> Result<bool, String> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
        };

        // Parse modifiers
        let mut mod_flags = HOT_KEY_MODIFIERS(0);
        for mod_str in modifiers.split('+').map(|s| s.trim().to_uppercase()) {
            match mod_str.as_str() {
                "ALT" => mod_flags |= HOT_KEY_MODIFIERS(1),
                "CTRL" | "CONTROL" => mod_flags |= HOT_KEY_MODIFIERS(2),
                "SHIFT" => mod_flags |= HOT_KEY_MODIFIERS(4),
                "WIN" | "WINDOWS" => mod_flags |= HOT_KEY_MODIFIERS(8),
                _ => {}
            }
        }

        let vk_code = self.key_to_vk_code(key)?;

        // Try to register temporarily
        let result = unsafe { RegisterHotKey(HWND::default(), HOTKEY_ID + 1, mod_flags, vk_code) };

        match result {
            Ok(_) => {
                // No conflict, unregister immediately
                unsafe {
                    let _ = UnregisterHotKey(HWND::default(), HOTKEY_ID + 1);
                }
                Ok(false)
            }
            Err(_) => {
                // Hotkey is already registered by another app
                Ok(true)
            }
        }
    }

    /// Check conflict (non-Windows placeholder)
    #[cfg(not(windows))]
    pub fn check_conflict(&self, _modifiers: &str, _key: &str) -> Result<bool, String> {
        Ok(false)
    }

    /// Start listening for hotkey events
    pub fn start_listener(&mut self, app_handle: AppHandle<tauri::Wry>) -> Result<(), String> {
        self.app_handle = Some(app_handle);
        self.running.store(true, Ordering::SeqCst);

        #[cfg(windows)]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

            let running = self.running.clone();
            let app_handle_clone = self.app_handle.clone();

            thread::spawn(move || {
                let mut msg = MSG::default();

                while running.load(Ordering::SeqCst) {
                    let result = unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0) };

                    if result.0 == 0 {
                        break;
                    }

                    if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
                        log::info!("Global hotkey pressed!");

                        // Emit event to frontend
                        if let Some(handle) = &app_handle_clone {
                            if let Err(e) = handle.emit("hotkey-pressed", ()) {
                                log::error!("Failed to emit hotkey event: {}", e);
                            }
                        }
                    }
                }
            });
        }

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

    /// Convert key string to Windows virtual key code
    #[cfg(windows)]
    fn key_to_vk_code(&self, key: &str) -> Result<u32, String> {
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

    /// Key to VK code (non-Windows placeholder)
    #[cfg(not(windows))]
    fn key_to_vk_code(&self, _key: &str) -> Result<u32, String> {
        Err("Virtual key codes are only available on Windows".to_string())
    }

    /// Save configuration to file
    fn save_config(&self) -> Result<(), String> {
        // TODO: Implement configuration persistence
        // This would typically save to a config file
        Ok(())
    }

    /// Load configuration from file
    pub fn load_config(&self) -> Result<HotkeyConfig, String> {
        // TODO: Implement configuration loading
        // For now, return default
        Ok(HotkeyConfig::default())
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
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