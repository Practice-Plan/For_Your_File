//! Global hotkey management
//!
//! Handles registration and processing of global hotkeys.

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Hotkey identifiers
#[cfg(windows)]
pub const HOTKEY_ID: i32 = 1;

/// Global hotkey manager
pub struct HotkeyManager {
    running: Arc<AtomicBool>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Register the global hotkey
    #[cfg(windows)]
    pub fn register(&self, modifiers: &str, key: &str) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            RegisterHotKey, VIRTUAL_KEY, HOT_KEY_MODIFIERS,
        };
        use windows::Win32::Foundation::HWND;

        let mut mod_flags = HOT_KEY_MODIFIERS(0);
        for mod_str in modifiers.split('+').map(|s| s.trim().to_uppercase()) {
            match mod_str.as_str() {
                "ALT" => mod_flags |= HOT_KEY_MODIFIERS(1),     // MOD_ALT
                "CTRL" | "CONTROL" => mod_flags |= HOT_KEY_MODIFIERS(2), // MOD_CONTROL
                "SHIFT" => mod_flags |= HOT_KEY_MODIFIERS(4),    // MOD_SHIFT
                "WIN" => mod_flags |= HOT_KEY_MODIFIERS(8),      // MOD_WIN
                _ => log::warn!("Unknown modifier: {}", mod_str),
            }
        }

        // Convert key string to virtual key code
        let vk_code: u32 = match key.to_uppercase().as_str() {
            "SPACE" => 0x20,
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
            _ => key.chars().next().unwrap_or(' ') as u32,
        };

        unsafe {
            RegisterHotKey(HWND::default(), HOTKEY_ID, mod_flags, vk_code)
                .map_err(|e| anyhow::anyhow!("Failed to register hotkey: {}", e))?;
        }

        log::info!("Global hotkey registered: {} + {}", modifiers, key);
        Ok(())
    }

    /// Unregister the global hotkey
    #[cfg(windows)]
    pub fn unregister(&self) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey;
        use windows::Win32::Foundation::HWND;

        unsafe {
            let _ = UnregisterHotKey(HWND::default(), HOTKEY_ID);
        }

        log::info!("Global hotkey unregistered");
        Ok(())
    }

    /// Start listening for hotkey events
    pub fn start<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut() + Send + 'static,
    {
        self.running.store(true, Ordering::SeqCst);

        #[cfg(windows)]
        {
            use std::thread;
            use windows::Win32::UI::WindowsAndMessaging::{
                GetMessageW, WM_HOTKEY, MSG,
            };
            use windows::Win32::Foundation::HWND;

            let running = self.running.clone();
            thread::spawn(move || {
                let mut msg = MSG::default();
                while running.load(Ordering::SeqCst) {
                    unsafe {
                        if GetMessageW(&mut msg, HWND::default(), 0, 0).0 == 0 {
                            break;
                        }

                        if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
                            callback();
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// Stop listening for hotkey events
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(windows))]
impl HotkeyManager {
    pub fn register(&self, _modifiers: &str, _key: &str) -> Result<()> {
        anyhow::bail!("Hotkey registration is only supported on Windows");
    }

    pub fn unregister(&self) -> Result<()> {
        anyhow::bail!("Hotkey unregistration is only supported on Windows");
    }

    pub fn start<F>(&self, _callback: F) -> Result<()>
    where
        F: FnMut() + Send + 'static,
    {
        anyhow::bail!("Hotkey listening is only supported on Windows");
    }
}