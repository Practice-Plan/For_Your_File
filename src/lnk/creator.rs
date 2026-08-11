//! LNK file creator for generating Windows shortcuts

use anyhow::{Context, Result};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during LNK file creation
#[derive(Debug, Error)]
pub enum LnkCreateError {
    #[error("Failed to initialize COM: {0}")]
    ComInitError(String),

    #[error("Failed to create ShellLink object: {0}")]
    ShellLinkError(String),

    #[error("Failed to save LNK file: {0}")]
    SaveError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Platform not supported")]
    PlatformNotSupported,
}

/// Window state for the shortcut
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowState {
    /// Normal window
    Normal,
    /// Minimized window
    Minimized,
    /// Maximized window
    Maximized,
}

impl WindowState {
    /// Convert to Windows show command value
    fn to_show_cmd(&self) -> i32 {
        match self {
            WindowState::Normal => 1,     // SW_SHOWNORMAL
            WindowState::Minimized => 7,  // SW_SHOWMINNOACTIVE
            WindowState::Maximized => 3,  // SW_SHOWMAXIMIZED
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState::Normal
    }
}

/// Builder for creating LNK shortcuts with fluent API
#[derive(Debug, Clone)]
pub struct LnkBuilder {
    target_path: String,
    arguments: Option<String>,
    working_directory: Option<String>,
    description: Option<String>,
    icon_location: Option<String>,
    icon_index: Option<i32>,
    window_state: WindowState,
}

impl LnkBuilder {
    /// Create a new LNK builder with the specified target path
    pub fn new(target_path: impl Into<String>) -> Self {
        Self {
            target_path: target_path.into(),
            arguments: None,
            working_directory: None,
            description: None,
            icon_location: None,
            icon_index: None,
            window_state: WindowState::Normal,
        }
    }

    /// Set command line arguments
    pub fn arguments(mut self, args: impl Into<String>) -> Self {
        self.arguments = Some(args.into());
        self
    }

    /// Set working directory
    pub fn working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set icon location and index
    pub fn icon(mut self, location: impl Into<String>, index: i32) -> Self {
        self.icon_location = Some(location.into());
        self.icon_index = Some(index);
        self
    }

    /// Set window state
    pub fn window_state(mut self, state: WindowState) -> Self {
        self.window_state = state;
        self
    }

    /// Build the LNK file at the specified path
    pub fn build<P: AsRef<Path>>(&self, lnk_path: P) -> Result<()> {
        create_lnk_file(
            lnk_path,
            &self.target_path,
            self.arguments.as_deref(),
            self.working_directory.as_deref(),
            self.description.as_deref(),
            self.icon_location.as_deref(),
            self.icon_index,
            self.window_state,
        )
    }
}

/// Create a new .lnk shortcut file
pub fn create_lnk_file<P: AsRef<Path>>(
    lnk_path: P,
    target_path: &str,
    arguments: Option<&str>,
    working_directory: Option<&str>,
    description: Option<&str>,
    icon_location: Option<&str>,
    icon_index: Option<i32>,
    window_state: WindowState,
) -> Result<()> {
    let lnk_path = lnk_path.as_ref();

    #[cfg(windows)]
    {
        create_lnk_file_windows(
            lnk_path,
            target_path,
            arguments,
            working_directory,
            description,
            icon_location,
            icon_index,
            window_state,
        )
    }

    #[cfg(not(windows))]
    {
        Err(LnkCreateError::PlatformNotSupported.into())
    }
}

#[cfg(windows)]
fn create_lnk_file_windows(
    lnk_path: &Path,
    target_path: &str,
    arguments: Option<&str>,
    working_directory: Option<&str>,
    description: Option<&str>,
    icon_location: Option<&str>,
    icon_index: Option<i32>,
    window_state: WindowState,
) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{IPersistFile, CoCreateInstance, CoInitializeEx, CLSCTX_SERVER, COINIT_MULTITHREADED};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    // Ensure parent directory exists
    if let Some(parent) = lnk_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    // Initialize COM
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .context("Failed to initialize COM")?;

    let _com_guard = ComGuard;

    // Create ShellLink object
    let shell_link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_SERVER) }
        .context("Failed to create ShellLink instance")?;

    // Set target path
    let target_wide: Vec<u16> = target_path
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe { shell_link.SetPath(PCWSTR(target_wide.as_ptr())) }
        .context("Failed to set target path")?;

    // Set arguments if provided
    if let Some(args) = arguments {
        let args_wide: Vec<u16> = args.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { shell_link.SetArguments(PCWSTR(args_wide.as_ptr())) }
            .context("Failed to set arguments")?;
    }

    // Set working directory if provided
    if let Some(workdir) = working_directory {
        let workdir_wide: Vec<u16> = workdir.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { shell_link.SetWorkingDirectory(PCWSTR(workdir_wide.as_ptr())) }
            .context("Failed to set working directory")?;
    }

    // Set description if provided
    if let Some(desc) = description {
        let desc_wide: Vec<u16> = desc.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { shell_link.SetDescription(PCWSTR(desc_wide.as_ptr())) }
            .context("Failed to set description")?;
    }

    // Set icon if provided
    if let Some(icon_loc) = icon_location {
        let icon_wide: Vec<u16> = icon_loc.encode_utf16().chain(std::iter::once(0)).collect();
        let index = icon_index.unwrap_or(0);
        unsafe { shell_link.SetIconLocation(PCWSTR(icon_wide.as_ptr()), index) }
            .context("Failed to set icon location")?;
    }

    // Set window state
    use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;
    unsafe { shell_link.SetShowCmd(SHOW_WINDOW_CMD(window_state.to_show_cmd())) }
        .context("Failed to set window state")?;

    // Get IPersistFile interface and save
    let persist_file: IPersistFile = shell_link
        .cast()
        .context("Failed to get IPersistFile interface")?;

    let lnk_wide: Vec<u16> = lnk_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { persist_file.Save(PCWSTR(lnk_wide.as_ptr()), true) }
        .context(format!("Failed to save LNK file: {}", lnk_path.display()))?;

    log::info!("Successfully created LNK file: {}", lnk_path.display());
    Ok(())
}

/// Update an existing .lnk file's target
pub fn update_lnk_target<P: AsRef<Path>>(lnk_path: P, new_target: &str) -> Result<()> {
    let lnk_path = lnk_path.as_ref();

    #[cfg(windows)]
    {
        update_lnk_target_windows(lnk_path, new_target)
    }

    #[cfg(not(windows))]
    {
        Err(LnkCreateError::PlatformNotSupported.into())
    }
}

#[cfg(windows)]
fn update_lnk_target_windows(lnk_path: &Path, new_target: &str) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{IPersistFile, CoCreateInstance, CoInitializeEx, CLSCTX_SERVER, COINIT_MULTITHREADED, STGM};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    if !lnk_path.exists() {
        anyhow::bail!("LNK file does not exist: {}", lnk_path.display());
    }

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

    // Load the existing LNK file
    let lnk_wide: Vec<u16> = lnk_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { persist_file.Load(PCWSTR(lnk_wide.as_ptr()), STGM(0)) }
        .context(format!("Failed to load LNK file: {}", lnk_path.display()))?;

    // Update the target path
    let target_wide: Vec<u16> = new_target.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { shell_link.SetPath(PCWSTR(target_wide.as_ptr())) }
        .context("Failed to set new target path")?;

    // Save the changes
    unsafe { persist_file.Save(PCWSTR(lnk_wide.as_ptr()), true) }
        .context("Failed to save updated LNK file")?;

    log::info!(
        "Successfully updated LNK file target: {} -> {}",
        lnk_path.display(),
        new_target
    );
    Ok(())
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