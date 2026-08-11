//! LNK file parsing for the Tauri backend.
//!
//! Uses the Windows Shell API (IShellLinkW + IPersistFile) to extract properties
//! from .lnk shortcut files. This is used for auto-completing entry fields when
//! a user uploads a .lnk file.

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
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_SERVER,
        COINIT_MULTITHREADED, STGM,
    };
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    // Initialize COM
    let co_init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if co_init.is_err() {
        return Err("Failed to initialize COM".to_string());
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
    let shell_link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_SERVER) }
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
    if let Ok(()) = unsafe { shell_link.GetPath(&mut target_buffer, &mut find_data, 0) } {
        let len = target_buffer.iter().position(|&c| c == 0).unwrap_or(0);
        if len > 0 {
            let os_string = OsString::from_wide(&target_buffer[..len]);
            props.target_path = os_string.to_string_lossy().to_string();
        }
    }

    // Read arguments
    let mut args_buffer = [0u16; 260];
    if let Ok(()) = unsafe { shell_link.GetArguments(&mut args_buffer) } {
        let len = args_buffer.iter().position(|&c| c == 0).unwrap_or(0);
        if len > 0 {
            let os_string = OsString::from_wide(&args_buffer[..len]);
            let args = os_string.to_string_lossy().to_string();
            if !args.is_empty() {
                props.arguments = Some(args);
            }
        }
    }

    // Read working directory
    let mut workdir_buffer = [0u16; 260];
    if let Ok(()) = unsafe { shell_link.GetWorkingDirectory(&mut workdir_buffer) } {
        let len = workdir_buffer.iter().position(|&c| c == 0).unwrap_or(0);
        if len > 0 {
            let os_string = OsString::from_wide(&workdir_buffer[..len]);
            let workdir = os_string.to_string_lossy().to_string();
            if !workdir.is_empty() {
                props.working_directory = Some(workdir);
            }
        }
    }

    // Read description
    let mut desc_buffer = [0u16; 260];
    if let Ok(()) = unsafe { shell_link.GetDescription(&mut desc_buffer) } {
        let len = desc_buffer.iter().position(|&c| c == 0).unwrap_or(0);
        if len > 0 {
            let os_string = OsString::from_wide(&desc_buffer[..len]);
            let desc = os_string.to_string_lossy().to_string();
            if !desc.is_empty() {
                props.description = Some(desc);
            }
        }
    }

    // Read icon location
    let mut icon_buffer = [0u16; 260];
    let mut icon_index = 0i32;
    if let Ok(()) = unsafe { shell_link.GetIconLocation(&mut icon_buffer, &mut icon_index) } {
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

    Ok(props)
}
