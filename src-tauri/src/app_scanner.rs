//! Installed application scanner.
//!
//! Scans the Windows Start Menu for .lnk shortcut files, parses each one to
//! extract the target executable path, and returns a list of installed apps.
//! Also provides icon extraction via PowerShell's System.Drawing API.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Information about an installed application discovered in the Start Menu.
#[derive(Debug, Clone, Serialize)]
pub struct InstalledApp {
    /// Display name (derived from the .lnk filename or the shortcut description)
    pub name: String,
    /// Target executable path that the shortcut points to
    pub target_path: String,
    /// Path to the .lnk shortcut file
    pub lnk_path: String,
    /// Optional description from the shortcut
    pub description: Option<String>,
}

/// Scan the Windows Start Menu for installed applications.
///
/// Searches both the user and system Start Menu directories for .lnk files,
/// parses each to extract the target executable, and returns a deduplicated list.
///
/// On non-Windows platforms, returns an empty list.
pub fn list_installed_apps() -> Vec<InstalledApp> {
    #[cfg(windows)]
    {
        let start_menu_paths = get_start_menu_paths();
        let lnk_files = collect_lnk_files(&start_menu_paths);
        parse_lnk_files(lnk_files)
    }

    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// Get the Start Menu program paths (user and system).
#[cfg(windows)]
fn get_start_menu_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // User Start Menu: %APPDATA%\Microsoft\Windows\Start Menu\Programs
    if let Ok(appdata) = std::env::var("APPDATA") {
        let user_start_menu = PathBuf::from(&appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        if user_start_menu.exists() {
            paths.push(user_start_menu);
        }
    }

    // System Start Menu: C:\ProgramData\Microsoft\Windows\Start Menu\Programs
    let system_start_menu = PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs");
    if system_start_menu.exists() {
        paths.push(system_start_menu);
    }

    paths
}

/// Recursively collect all .lnk files from the given directories.
#[cfg(windows)]
fn collect_lnk_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut lnk_files = Vec::new();

    for dir in dirs {
        collect_lnk_files_recursive(dir, &mut lnk_files);
    }

    lnk_files
}

/// Recursively walk a directory and collect .lnk file paths.
#[cfg(windows)]
fn collect_lnk_files_recursive(dir: &Path, lnk_files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_lnk_files_recursive(&path, lnk_files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("lnk") {
            lnk_files.push(path);
        }
    }
}

/// Parse a list of .lnk files and convert them to InstalledApp entries.
///
/// Deduplicates by target_path to avoid showing the same app twice
/// (once from user Start Menu, once from system Start Menu).
#[cfg(windows)]
fn parse_lnk_files(lnk_files: Vec<PathBuf>) -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut seen_targets = std::collections::HashSet::new();

    for lnk_path in &lnk_files {
        // Derive display name from filename (without .lnk extension)
        let name = lnk_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Parse the .lnk file to get the target path
        match crate::lnk::parse_lnk_file(&lnk_path.to_string_lossy()) {
            Ok(props) => {
                let target_path = props.target_path.clone();

                // Skip if target is empty or already seen
                if target_path.is_empty() || seen_targets.contains(&target_path) {
                    continue;
                }

                // Skip uninstallers and help files
                let lower = target_path.to_lowercase();
                if lower.contains("unins") || lower.contains("uninstall") {
                    continue;
                }

                seen_targets.insert(target_path.clone());
                apps.push(InstalledApp {
                    name: props.description.clone().filter(|d| !d.is_empty()).unwrap_or(name),
                    target_path,
                    lnk_path: lnk_path.to_string_lossy().to_string(),
                    description: props.description,
                });
            }
            Err(_) => {
                // If parsing fails, still include the app with the .lnk path as target
                // (the .lnk file itself can be launched)
                let lnk_str = lnk_path.to_string_lossy().to_string();
                if !seen_targets.contains(&lnk_str) {
                    seen_targets.insert(lnk_str.clone());
                    apps.push(InstalledApp {
                        name,
                        target_path: lnk_str,
                        lnk_path: lnk_path.to_string_lossy().to_string(),
                        description: None,
                    });
                }
            }
        }
    }

    // Sort alphabetically by name
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    apps
}

/// Extract an application icon as a base64-encoded PNG string.
///
/// Uses PowerShell's System.Drawing.Icon.ExtractAssociatedIcon to extract the
/// icon from the target executable, converts it to PNG, and returns as base64.
///
/// Returns an empty string on failure (non-fatal — the UI shows a default icon).
#[cfg(windows)]
pub fn extract_icon_as_base64(exe_path: &str) -> Result<String, String> {
    // PowerShell script that extracts the icon and outputs base64
    let ps_script = format!(
        r#"
Add-Type -AssemblyName System.Drawing
try {{
    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon('{}')
    if ($icon -ne $null) {{
        $ms = New-Object System.IO.MemoryStream
        $bmp = $icon.ToBitmap()
        $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
        $icon.Dispose()
        [Convert]::ToBase64String($ms.ToArray())
    }}
}} catch {{}}
"#,
        exe_path.replace('\'', "''")
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if stdout.is_empty() {
        return Err("Icon extraction returned empty output".to_string());
    }

    Ok(stdout)
}

#[cfg(not(windows))]
pub fn extract_icon_as_base64(_exe_path: &str) -> Result<String, String> {
    Err("Icon extraction is only supported on Windows".to_string())
}
