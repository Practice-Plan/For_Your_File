//! Installed application scanner.
//!
//! Scans the Windows Start Menu for .lnk shortcut files, parses each one to
//! extract the target executable path, and returns a list of installed apps.
//! Also provides icon extraction via native Windows API (ExtractIconExW + GDI)
//! with disk caching for fast subsequent loads.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

// ---------------------------------------------------------------------------
// Icon disk cache
// ---------------------------------------------------------------------------
// Cache layout: <cache_dir>/<hash>.png  +  <cache_dir>/<hash>.meta
//
// The .meta file contains:
//   line 1: exe path (for verification)
//   line 2: exe last-modified timestamp (secs since UNIX epoch)
//
// Cache is invalidated when the exe's modification time changes (app updated).

/// Compute a stable cache key from an exe path using DefaultHasher.
fn cache_key(exe_path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    exe_path.to_lowercase().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Try to read a cached icon. Returns Some(base64) if cache hit and still valid.
fn read_icon_cache(cache_dir: &Path, exe_path: &str) -> Option<String> {
    let key = cache_key(exe_path);
    let png_path = cache_dir.join(format!("{}.png", key));
    let meta_path = cache_dir.join(format!("{}.meta", key));

    // Both files must exist
    if !png_path.exists() || !meta_path.exists() {
        return None;
    }

    // Read meta and validate
    let meta = std::fs::read_to_string(&meta_path).ok()?;
    let mut lines = meta.lines();
    let cached_path = lines.next()?;
    let cached_mtime: i64 = lines.next()?.parse().ok()?;

    // Path must match (case-insensitive on Windows)
    if cached_path.to_lowercase() != exe_path.to_lowercase() {
        return None;
    }

    // Check if exe has been modified since caching
    if let Ok(exe_meta) = std::fs::metadata(exe_path) {
        if let Ok(exe_mtime) = exe_meta.modified() {
            let exe_ts = exe_mtime
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            if exe_ts != cached_mtime {
                return None; // exe was updated, cache stale
            }
        }
    }

    // Cache hit — read and return
    std::fs::read_to_string(&png_path).ok()
}

/// Write an icon to the cache along with metadata for future validation.
fn write_icon_cache(cache_dir: &Path, exe_path: &str, base64_data: &str) {
    let key = cache_key(exe_path);
    let png_path = cache_dir.join(format!("{}.png", key));
    let meta_path = cache_dir.join(format!("{}.meta", key));

    // Ensure cache directory exists
    if let Err(e) = std::fs::create_dir_all(cache_dir) {
        log::warn!("Failed to create icon cache dir: {}", e);
        return;
    }

    // Write PNG data
    if let Err(e) = std::fs::write(&png_path, base64_data) {
        log::warn!("Failed to write icon cache: {}", e);
        return;
    }

    // Write meta with exe path and modification timestamp
    let mtime = std::fs::metadata(exe_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let meta_content = format!("{}\n{}", exe_path, mtime);
    let _ = std::fs::write(&meta_path, meta_content);
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
/// Uses native Windows API: SHGetFileInfoW to extract the icon handle,
/// GetIconInfo + GetDIBits to read pixel data directly from the icon bitmap.
/// No PowerShell subprocess needed — runs entirely in-process.
///
/// Supports disk caching: if `cache_dir` is provided, the function checks for
/// a cached icon first (validated by exe modification time). On cache miss,
/// the extracted icon is saved to disk for future sessions.
///
/// Returns an empty string on failure (non-fatal — the UI shows a default icon).
#[cfg(windows)]
pub fn extract_icon_as_base64(
    exe_path: &str,
    cache_dir: Option<&Path>,
) -> Result<String, String> {
    // Check disk cache first
    if let Some(dir) = cache_dir {
        if let Some(cached) = read_icon_cache(dir, exe_path) {
            return Ok(cached);
        }
    }

    use base64::Engine;
    use image::{ImageBuffer, Rgba};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    // Convert path to wide string
    let wide_path: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Extract icon handle via SHGetFileInfoW (available in Win32_UI_Shell)
    let mut shfi = SHFILEINFOW::default();
    unsafe {
        SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
    }

    let hicon = shfi.hIcon;
    if hicon.is_invalid() {
        return Err("No icon found in executable".to_string());
    }

    // Get icon bitmap handles
    let mut icon_info = ICONINFO::default();
    if unsafe { GetIconInfo(hicon, &mut icon_info) }.is_err() {
        let _ = unsafe { DestroyIcon(hicon) };
        return Err("Failed to get icon info".to_string());
    }

    let hbm_color = icon_info.hbmColor;

    // Get bitmap dimensions via GetObjectW
    let mut bmp = BITMAP::default();
    let bmp_size = unsafe {
        GetObjectW(
            hbm_color,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as *mut _),
        )
    };

    if bmp_size == 0 {
        unsafe {
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            let _ = DestroyIcon(hicon);
        }
        return Err("Failed to get bitmap dimensions".to_string());
    }

    let width = bmp.bmWidth as u32;
    let height = bmp.bmHeight as u32;

    if width == 0 || height == 0 {
        unsafe {
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            let _ = DestroyIcon(hicon);
        }
        return Err("Invalid icon dimensions".to_string());
    }

    // Read pixels directly from the icon's color bitmap via GetDIBits.
    // Create a temporary compatible DC (no need for screen DC or DrawIconEx).
    let hdc = unsafe { CreateCompatibleDC(None) };

    // Build BITMAPINFO for 32-bit top-down BGRA output
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32), // negative = top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            ..Default::default()
        },
        ..Default::default()
    };

    // pixels must be declared outside unsafe so it's accessible after the block
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    unsafe {
        GetDIBits(
            hdc,
            hbm_color,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI resources
        DeleteDC(hdc).ok();
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());
        let _ = DestroyIcon(hicon);
    }

    // Convert BGRA to RGBA (Windows GDI returns BGRA order)
    // Icons may use premultiplied alpha, so un-premultiply for correct display.
    let mut rgba_pixels = vec![0u8; (width * height * 4) as usize];
    for i in 0..(width * height) as usize {
        let b = pixels[i * 4];
        let g = pixels[i * 4 + 1];
        let r = pixels[i * 4 + 2];
        let a = pixels[i * 4 + 3];
        if a > 0 && a < 255 {
            // Un-premultiply alpha
            rgba_pixels[i * 4] = ((r as u16 * 255) / a as u16).min(255) as u8;
            rgba_pixels[i * 4 + 1] = ((g as u16 * 255) / a as u16).min(255) as u8;
            rgba_pixels[i * 4 + 2] = ((b as u16 * 255) / a as u16).min(255) as u8;
            rgba_pixels[i * 4 + 3] = a;
        } else {
            rgba_pixels[i * 4] = r;
            rgba_pixels[i * 4 + 1] = g;
            rgba_pixels[i * 4 + 2] = b;
            rgba_pixels[i * 4 + 3] = a;
        }
    }

    // Create image and encode as PNG
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba_pixels).ok_or("Failed to create image")?;

    let mut png_bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )
    .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    // Encode as base64
    let base64_str = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    // Save to disk cache for future sessions
    if let Some(dir) = cache_dir {
        write_icon_cache(dir, exe_path, &base64_str);
    }

    Ok(base64_str)
}

#[cfg(not(windows))]
pub fn extract_icon_as_base64(
    _exe_path: &str,
    _cache_dir: Option<&Path>,
) -> Result<String, String> {
    Err("Icon extraction is only supported on Windows".to_string())
}
