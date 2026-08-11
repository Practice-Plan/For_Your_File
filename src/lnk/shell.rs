//! Windows Shell integration
//!
//! Handles context menu registration and shell operations.

use anyhow::Result;
use std::path::Path;

/// Register the application in Windows context menu
#[cfg(windows)]
pub fn register_context_menu() -> Result<()> {
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyW, RegSetValueExW, HKEY_CURRENT_USER, HKEY, REG_SZ,
    };
    use windows::core::PCWSTR;

    unsafe {
        // Create registry key for context menu
        let key_path = "Software\\Classes\\*\\shell\\AddToLnkManager\\command";
        let key_path_wide: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();

        let mut h_key = HKEY::default();

        // Create the key
        RegCreateKeyW(HKEY_CURRENT_USER, PCWSTR(key_path_wide.as_ptr()), &mut h_key).ok()?;

        // Set command
        let exe_path = std::env::current_exe()?;
        let command = format!("\"{}\" add \"%1\"", exe_path.display());
        // Convert to bytes (UTF-16LE for Windows registry)
        let cmd_bytes: Vec<u8> = command
            .encode_utf16()
            .flat_map(|c| [c as u8, (c >> 8) as u8])
            .chain(std::iter::once(0))
            .chain(std::iter::once(0))
            .collect();

        RegSetValueExW(
            h_key,
            PCWSTR::null(),
            0,
            REG_SZ,
            Some(cmd_bytes.as_slice()),
        ).ok()?;

        let _ = RegCloseKey(h_key);
    }

    log::info!("Context menu registered successfully");
    Ok(())
}

/// Unregister the application from Windows context menu
#[cfg(windows)]
pub fn unregister_context_menu() -> Result<()> {
    use windows::Win32::System::Registry::{RegDeleteKeyW, HKEY_CURRENT_USER};
    use windows::core::PCWSTR;

    unsafe {
        // Delete the command subkey first
        let command_path = "Software\\Classes\\*\\shell\\AddToLnkManager\\command";
        let command_wide: Vec<u16> = command_path.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(command_wide.as_ptr()));

        // Then delete the parent key
        let path = "Software\\Classes\\*\\shell\\AddToLnkManager";
        let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(path_wide.as_ptr()));
    }

    log::info!("Context menu unregistered successfully");
    Ok(())
}

/// Launch a shortcut by executing its target
pub fn launch_shortcut<P: AsRef<Path>>(lnk_path: P) -> Result<()> {
    use std::process::Command;

    let path = lnk_path.as_ref();

    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()?;
    }

    #[cfg(not(windows))]
    {
        anyhow::bail!("Shortcut launching is only supported on Windows");
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn register_context_menu() -> Result<()> {
    anyhow::bail!("Context menu registration is only supported on Windows");
}

#[cfg(not(windows))]
pub fn unregister_context_menu() -> Result<()> {
    anyhow::bail!("Context menu unregistration is only supported on Windows");
}