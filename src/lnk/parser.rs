//! LNK file parser for reading shortcut properties

use anyhow::{Context, Result};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during LNK file parsing
#[derive(Debug, Error)]
pub enum LnkParseError {
    #[error("LNK file does not exist: {0}")]
    FileNotFound(String),

    #[error("Failed to initialize COM: {0}")]
    ComInitError(String),

    #[error("Failed to create ShellLink object: {0}")]
    ShellLinkError(String),

    #[error("Failed to load LNK file: {0}")]
    LoadError(String),

    #[error("Failed to read property: {0}")]
    PropertyReadError(String),

    #[error("Corrupted LNK file: {0}")]
    CorruptedFile(String),

    #[error("Platform not supported")]
    PlatformNotSupported,
}

/// Parsed LNK file properties
#[derive(Debug, Clone)]
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
    /// Show command (1=normal, 3=maximized, 7=minimized)
    pub show_command: Option<i32>,
}

impl Default for LnkProperties {
    fn default() -> Self {
        Self {
            target_path: String::new(),
            arguments: None,
            working_directory: None,
            description: None,
            icon_location: None,
            icon_index: None,
            show_command: Some(1), // Normal window
        }
    }
}

/// Parse a .lnk file and extract its properties using Windows Shell API
pub fn parse_lnk_file<P: AsRef<Path>>(path: P) -> Result<LnkProperties> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(LnkParseError::FileNotFound(path.display().to_string()).into());
    }

    #[cfg(windows)]
    {
        parse_lnk_file_windows(path)
    }

    #[cfg(not(windows))]
    {
        Err(LnkParseError::PlatformNotSupported.into())
    }
}

#[cfg(windows)]
fn parse_lnk_file_windows(path: &Path) -> Result<LnkProperties> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{IPersistFile, CoCreateInstance, CoInitializeEx, CLSCTX_SERVER, COINIT_MULTITHREADED, STGM};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    // Initialize COM
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .context("Failed to initialize COM")?;

    let _com_guard = ComGuard;

    // Create ShellLink object
    let shell_link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_SERVER) }
        .context("Failed to create ShellLink instance")?;

    // Get IPersistFile interface
    let persist_file: IPersistFile = shell_link
        .cast()
        .context("Failed to get IPersistFile interface")?;

    // Load the LNK file
    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { persist_file.Load(PCWSTR(path_wide.as_ptr()), STGM(0)) }
        .context(format!("Failed to load LNK file: {}", path.display()))?;

    // Extract properties
    let mut props = LnkProperties::default();

    // Read target path
    let mut target_buffer = [0u16; 260];
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    let mut find_data = WIN32_FIND_DATAW::default();
    match unsafe { shell_link.GetPath(&mut target_buffer, &mut find_data, 0) } {
        Ok(_) => {
            let len = target_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&target_buffer[..len]);
                props.target_path = os_string.to_string_lossy().to_string();
            }
        }
        Err(e) => log::warn!("Failed to read target path: {:?}", e),
    }

    // Read arguments
    let mut args_buffer = [0u16; 260];
    match unsafe { shell_link.GetArguments(&mut args_buffer) } {
        Ok(_) => {
            let len = args_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&args_buffer[..len]);
                let args = os_string.to_string_lossy().to_string();
                if !args.is_empty() {
                    props.arguments = Some(args);
                }
            }
        }
        Err(e) => log::warn!("Failed to read arguments: {:?}", e),
    }

    // Read working directory
    let mut workdir_buffer = [0u16; 260];
    match unsafe { shell_link.GetWorkingDirectory(&mut workdir_buffer) } {
        Ok(_) => {
            let len = workdir_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&workdir_buffer[..len]);
                let workdir = os_string.to_string_lossy().to_string();
                if !workdir.is_empty() {
                    props.working_directory = Some(workdir);
                }
            }
        }
        Err(e) => log::warn!("Failed to read working directory: {:?}", e),
    }

    // Read description
    let mut desc_buffer = [0u16; 260];
    match unsafe { shell_link.GetDescription(&mut desc_buffer) } {
        Ok(_) => {
            let len = desc_buffer.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                let os_string = OsString::from_wide(&desc_buffer[..len]);
                let desc = os_string.to_string_lossy().to_string();
                if !desc.is_empty() {
                    props.description = Some(desc);
                }
            }
        }
        Err(e) => log::warn!("Failed to read description: {:?}", e),
    }

    // Read icon location
    let mut icon_buffer = [0u16; 260];
    let mut icon_index = 0i32;
    match unsafe { shell_link.GetIconLocation(&mut icon_buffer, &mut icon_index) } {
        Ok(_) => {
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
        Err(e) => log::warn!("Failed to read icon location: {:?}", e),
    }

    // Read show command
    match unsafe { shell_link.GetShowCmd() } {
        Ok(show_cmd) => {
            props.show_command = Some(show_cmd.0);
        }
        Err(e) => log::warn!("Failed to read show command: {:?}", e),
    }

    Ok(props)
}

/// RAII guard for COM initialization
#[cfg(windows)]
struct ComGuard;

#[cfg(windows)]
impl Drop for ComGuard {
    fn drop(&mut self) {
        use windows::Win32::System::Com::CoUninitialize;
        unsafe {
            CoUninitialize();
        }
    }
}