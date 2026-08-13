//! LNK file parsing for the Tauri backend.
//!
//! Uses the Windows Shell API (IShellLinkW + IPersistFile) to extract properties
//! from .lnk shortcut files. This is used for auto-completing entry fields when
//! a user uploads a .lnk file.
//!
//! IMPORTANT: COM operations must run on a dedicated thread to avoid apartment
//! model conflicts. Tauri command handlers run on async worker threads that may
//! already have COM initialized with a different apartment model (MTA vs STA).
//! Shell COM objects require STA (Single-Threaded Apartment). If we call
//! CoInitializeEx(STA) on a thread that already has MTA, it fails with
//! RPC_E_CHANGED_MODE. The fix is to spawn a fresh thread for each parse.

use serde::Serialize;

/// Parsed LNK file properties, serialized to the frontend for auto-completion.
#[derive(Debug, Clone, Serialize)]
pub struct LnkProperties {
    /// Target path the shortcut points to
    pub target_path: String,
    /// Command line arguments
    pub arguments: Option<String>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Icon location
    pub icon_location: Option<String>,
    /// Icon index
    pub icon_index: Option<i32>,
}

/// Parse a .lnk file and extract its properties using Windows Shell API.
///
/// On non-Windows platforms, returns an error.
pub fn parse_lnk_file(path: &str) -> Result<LnkProperties, String> {
    let path = std::path::Path::new(path);

    if !path.exists() {
        return Err(format!("LNK file does not exist: {}", path.display()));
    }

    #[cfg(windows)]
    {
        parse_lnk_file_windows(path)
    }

    #[cfg(not(windows))]
    {
        Err("LNK file parsing is only supported on Windows".to_string())
    }
}

#[cfg(windows)]
fn parse_lnk_file_windows(path: &std::path::Path) -> Result<LnkProperties, String> {
    // Spawn a dedicated thread for COM operations.
    // This avoids apartment model conflicts with Tauri's async worker threads,
    // which may already have COM initialized with MTA (Multi-Threaded Apartment).
    // Shell COM objects (IShellLinkW) require STA (Single-Threaded Apartment).
    let path = path.to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = parse_lnk_with_com(&path);
        let _ = tx.send(result);
    });

    rx.recv()
        .map_err(|e| format!("Failed to receive LNK parse result from worker thread: {}", e))?
}

#[cfg(windows)]
fn parse_lnk_with_com(path: &std::path::Path) -> Result<LnkProperties, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, STGM,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    // Initialize COM with STA (Single-Threaded Apartment).
    // STA is the recommended apartment model for Shell COM objects.
    // This thread is fresh (spawned by parse_lnk_file_windows), so there
    // should be no prior COM initialization to conflict with.
    let co_init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if co_init.is_err() {
        return Err(format!(
            "Failed to initialize COM (STA): {co_init:?}. This may indicate a COM apartment conflict."
        ));
    }

    // Ensure CoUninitialize is called when we're done
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    let _com_guard = ComGuard;

    // Create ShellLink object
    let shell_link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
        .map_err(|e| format!("Failed to create ShellLink instance: {}", e))?;

    // Get IPersistFile interface
    let persist_file: IPersistFile = shell_link
        .cast()
        .map_err(|e| format!("Failed to get IPersistFile interface: {}", e))?;

    // Load the LNK file
    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { persist_file.Load(PCWSTR(path_wide.as_ptr()), STGM(0)) }
        .map_err(|e| format!("Failed to load LNK file '{}': {}", path.display(), e))?;

    let mut props = LnkProperties {
        target_path: String::new(),
        arguments: None,
        working_directory: None,
        description: None,
        icon_location: None,
        icon_index: None,
    };

    // Read target path
    let mut target_buffer = [0u16; 260];
    let mut find_data = WIN32_FIND_DATAW::default();
    match unsafe { shell_link.GetPath(&mut target_buffer, &mut find_data, 0) } {
        Ok(()) => {
            let len = target_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&target_buffer[..len]);
                props.target_path = os_string.to_string_lossy().to_string();
            }
        }
        Err(e) => {
            log::warn!("Failed to read target path from LNK: {}", e);
        }
    }

    // Read arguments
    let mut args_buffer = [0u16; 260];
    match unsafe { shell_link.GetArguments(&mut args_buffer) } {
        Ok(()) => {
            let len = args_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&args_buffer[..len]);
                let args = os_string.to_string_lossy().to_string();
                if !args.is_empty() {
                    props.arguments = Some(args);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to read arguments from LNK: {}", e);
        }
    }

    // Read working directory
    let mut workdir_buffer = [0u16; 260];
    match unsafe { shell_link.GetWorkingDirectory(&mut workdir_buffer) } {
        Ok(()) => {
            let len = workdir_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&workdir_buffer[..len]);
                let workdir = os_string.to_string_lossy().to_string();
                if !workdir.is_empty() {
                    props.working_directory = Some(workdir);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to read working directory from LNK: {}", e);
        }
    }

    // Read description
    let mut desc_buffer = [0u16; 260];
    match unsafe { shell_link.GetDescription(&mut desc_buffer) } {
        Ok(()) => {
            let len = desc_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&desc_buffer[..len]);
                let desc = os_string.to_string_lossy().to_string();
                if !desc.is_empty() {
                    props.description = Some(desc);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to read description from LNK: {}", e);
        }
    }

    // Read icon location
    let mut icon_buffer = [0u16; 260];
    let mut icon_index = 0i32;
    match unsafe { shell_link.GetIconLocation(&mut icon_buffer, &mut icon_index) } {
        Ok(()) => {
            let len = icon_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&icon_buffer[..len]);
                let icon = os_string.to_string_lossy().to_string();
                if !icon.is_empty() {
                    props.icon_location = Some(icon);
                    props.icon_index = Some(icon_index);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to read icon location from LNK: {}", e);
        }
    }

    // If target_path is still empty, the LNK file may be invalid or use a
    // non-standard format. Return an error so the frontend can inform the user.
    if props.target_path.is_empty() {
        return Err(format!(
            "LNK file '{}' was loaded but no target path was found. \
             The file may be corrupted or use a non-standard format.",
            path.display()
        ));
    }

    Ok(props)
}
