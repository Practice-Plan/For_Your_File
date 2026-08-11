//! Helper utility functions

use std::path::Path;

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get the application data directory
pub fn get_data_dir() -> std::path::PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    exe_dir.join("data")
}

/// Get the database path
pub fn get_database_path() -> std::path::PathBuf {
    get_data_dir().join("entries.db")
}

/// Get the shortcuts directory path
pub fn get_shortcuts_dir() -> std::path::PathBuf {
    get_data_dir().join("shortcuts")
}

/// Get the configuration file path
pub fn get_config_path() -> std::path::PathBuf {
    get_data_dir().join("config.json")
}

/// Extract filename from a path
pub fn extract_filename<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

/// Check if a file has a specific extension (case-insensitive)
pub fn has_extension<P: AsRef<Path>>(path: P, ext: &str) -> bool {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase() == ext.to_lowercase())
        .unwrap_or(false)
}

/// Format a timestamp as a human-readable string
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Invalid timestamp".to_string())
}

/// Get current Unix timestamp
pub fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_extension() {
        assert!(has_extension("test.lnk", "lnk"));
        assert!(has_extension("test.LNK", "lnk"));
        assert!(!has_extension("test.txt", "lnk"));
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(
            extract_filename("C:\\path\\to\\file.txt"),
            Some("file.txt".to_string())
        );
    }
}