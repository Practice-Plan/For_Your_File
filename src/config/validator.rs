//! Configuration validation
//!
//! Validates configuration settings to ensure they are correct and safe.

use crate::models::AppConfig;
use std::path::Path;
use thiserror::Error;

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid hotkey format: {0}")]
    InvalidHotkey(String),

    #[error("Invalid theme: {0}. Must be 'light' or 'dark'")]
    InvalidTheme(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid max_results: {0}. Must be between 1 and 1000")]
    InvalidMaxResults(usize),

    #[error("Invalid window dimensions: width={0}, height={1}. Must be positive")]
    InvalidWindowDimensions(u32, u32),

    #[error("Invalid sync provider: {0}. Must be 'OneDrive' or 'Jianguoyun'")]
    InvalidSyncProvider(String),

    #[error("Invalid sorting configuration: {0}")]
    InvalidSorting(String),
}

/// Supported hotkey modifiers
const VALID_MODIFIERS: &[&str] = &["Alt", "Ctrl", "Shift", "Win", "Meta", "Option", "Command"];

/// Supported hotkey keys
const VALID_KEYS: &[&str] = &[
    "Space", "Tab", "Enter", "Escape", "Backspace", "Delete", "Insert",
    "Home", "End", "PageUp", "PageDown",
    "Up", "Down", "Left", "Right",
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    "`", "-", "=", "[", "]", "\\", ";", "'", ",", ".", "/",
];

/// Supported themes
const VALID_THEMES: &[&str] = &["light", "dark"];

/// Supported sync providers
const VALID_SYNC_PROVIDERS: &[&str] = &["OneDrive", "Jianguoyun", "Nutstore"];

/// Validate hotkey format
///
/// Format: "Modifier+Key" or "Modifier1+Modifier2+Key"
/// Examples: "Alt+Space", "Ctrl+Shift+A", "Win+F1"
pub fn validate_hotkey(hotkey: &str) -> Result<(), ValidationError> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() || parts.len() < 2 {
        return Err(ValidationError::InvalidHotkey(
            format!("Hotkey must have at least one modifier and one key, got: {}", hotkey)
        ));
    }

    // All parts except the last must be modifiers
    for (i, part) in parts.iter().enumerate() {
        if i < parts.len() - 1 {
            // This is a modifier
            if !VALID_MODIFIERS.iter().any(|m| *m == *part) {
                return Err(ValidationError::InvalidHotkey(
                    format!("Invalid modifier '{}' in hotkey. Valid modifiers: {:?}", part, VALID_MODIFIERS)
                ));
            }
        } else {
            // This is the key (last element)
            if !VALID_KEYS.iter().any(|k| *k == *part) {
                return Err(ValidationError::InvalidHotkey(
                    format!("Invalid key '{}' in hotkey. Valid keys: {:?}", part, VALID_KEYS)
                ));
            }
        }
    }

    Ok(())
}

/// Validate path
///
/// Checks that the path is valid and parent directory exists or can be created.
pub fn validate_path<P: AsRef<Path>>(path: P) -> Result<(), ValidationError> {
    let path = path.as_ref();

    // Check if path is empty
    if path.as_os_str().is_empty() {
        return Err(ValidationError::InvalidPath("Path cannot be empty".to_string()));
    }

    // Check parent directory
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            // Try to create parent directory
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(ValidationError::InvalidPath(
                    format!("Cannot create parent directory {}: {}", parent.display(), e)
                ));
            }
        }
    }

    Ok(())
}

/// Validate theme
///
/// Must be one of: "light" or "dark"
pub fn validate_theme(theme: &str) -> Result<(), ValidationError> {
    if !VALID_THEMES.contains(&theme.to_lowercase().as_str()) {
        return Err(ValidationError::InvalidTheme(theme.to_string()));
    }
    Ok(())
}

/// Validate max_results
///
/// Must be between 1 and 1000
pub fn validate_max_results(max_results: usize) -> Result<(), ValidationError> {
    if max_results < 1 || max_results > 1000 {
        return Err(ValidationError::InvalidMaxResults(max_results));
    }
    Ok(())
}

/// Validate window dimensions
///
/// Width and height must be positive and reasonable
pub fn validate_window_dimensions(width: u32, height: u32) -> Result<(), ValidationError> {
    if width < 100 || height < 100 || width > 10000 || height > 10000 {
        return Err(ValidationError::InvalidWindowDimensions(width, height));
    }
    Ok(())
}

/// Validate sync provider
///
/// Must be one of: "OneDrive", "Jianguoyun", "Nutstore"
pub fn validate_sync_provider(provider: &str) -> Result<(), ValidationError> {
    if !VALID_SYNC_PROVIDERS.contains(&provider) {
        return Err(ValidationError::InvalidSyncProvider(provider.to_string()));
    }
    Ok(())
}

/// Validate entire configuration
///
/// Validates all configuration settings.
pub fn validate_config(config: &AppConfig) -> Result<(), ValidationError> {
    // Validate hotkey
    validate_hotkey(&config.hotkey)?;

    // Validate theme
    validate_theme(&config.theme)?;

    // Validate paths
    validate_path(&config.shortcuts_dir)?;
    validate_path(&config.database_path)?;

    // Validate max_results
    validate_max_results(config.max_results)?;

    // Validate window dimensions
    validate_window_dimensions(config.window.width, config.window.height)?;

    // Validate sorting configuration
    config.sorting.validate().map_err(|e| ValidationError::InvalidSorting(e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hotkey_valid() {
        assert!(validate_hotkey("Alt+Space").is_ok());
        assert!(validate_hotkey("Ctrl+Shift+A").is_ok());
        assert!(validate_hotkey("Win+F1").is_ok());
        assert!(validate_hotkey("Ctrl+Alt+Delete").is_ok());
    }

    #[test]
    fn test_validate_hotkey_invalid() {
        assert!(validate_hotkey("").is_err());
        assert!(validate_hotkey("Space").is_err()); // No modifier
        assert!(validate_hotkey("Invalid+Space").is_err()); // Invalid modifier
        assert!(validate_hotkey("Alt+InvalidKey").is_err()); // Invalid key
    }

    #[test]
    fn test_validate_theme_valid() {
        assert!(validate_theme("light").is_ok());
        assert!(validate_theme("dark").is_ok());
        assert!(validate_theme("Light").is_ok()); // Case insensitive
        assert!(validate_theme("DARK").is_ok());
    }

    #[test]
    fn test_validate_theme_invalid() {
        assert!(validate_theme("blue").is_err());
        assert!(validate_theme("").is_err());
    }

    #[test]
    fn test_validate_max_results_valid() {
        assert!(validate_max_results(1).is_ok());
        assert!(validate_max_results(50).is_ok());
        assert!(validate_max_results(1000).is_ok());
    }

    #[test]
    fn test_validate_max_results_invalid() {
        assert!(validate_max_results(0).is_err());
        assert!(validate_max_results(1001).is_err());
    }

    #[test]
    fn test_validate_window_dimensions_valid() {
        assert!(validate_window_dimensions(800, 600).is_ok());
        assert!(validate_window_dimensions(100, 100).is_ok());
        assert!(validate_window_dimensions(10000, 10000).is_ok());
    }

    #[test]
    fn test_validate_window_dimensions_invalid() {
        assert!(validate_window_dimensions(99, 600).is_err()); // Too narrow
        assert!(validate_window_dimensions(800, 99).is_err()); // Too short
        assert!(validate_window_dimensions(10001, 600).is_err()); // Too wide
    }

    #[test]
    fn test_validate_sync_provider_valid() {
        assert!(validate_sync_provider("OneDrive").is_ok());
        assert!(validate_sync_provider("Jianguoyun").is_ok());
        assert!(validate_sync_provider("Nutstore").is_ok());
    }

    #[test]
    fn test_validate_sync_provider_invalid() {
        assert!(validate_sync_provider("Dropbox").is_err());
        assert!(validate_sync_provider("").is_err());
    }

    #[test]
    fn test_validate_config_valid() {
        let config = AppConfig::default();
        assert!(validate_config(&config).is_ok());
    }
}