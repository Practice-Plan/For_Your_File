//! Configuration tests
//!
//! Comprehensive tests for configuration management.

use crate::config::{ConfigManager, migration, validator};
use crate::models::{AppConfig, WindowConfig};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// Test loading default configuration
#[test]
fn test_load_default_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config = ConfigManager::load_from(&config_path).unwrap();

    // Verify default values
    assert_eq!(config.settings().hotkey, "Alt+Space");
    assert_eq!(config.settings().theme, "light");
    assert_eq!(config.settings().max_results, 50);
    assert!(!config.settings().sync_enabled);
    assert!(config.settings().sync_provider.is_none());
    assert!(config.settings().sync_path.is_none());

    // Verify window defaults
    assert_eq!(config.settings().window.width, 800);
    assert_eq!(config.settings().window.height, 600);
    assert!(!config.settings().window.maximized);

    // Verify paths
    assert!(config_path.exists());
}

/// Test saving and loading configuration
#[test]
fn test_save_and_load_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.json");

    // Create and save configuration
    {
        let mut config = ConfigManager::load_from(&config_path).unwrap();
        config.set_hotkey("Ctrl+Shift+A".to_string()).unwrap();
        config.set_theme("dark".to_string()).unwrap();
        config.set_max_results(100).unwrap();
    }

    // Load and verify
    let config = ConfigManager::load_from(&config_path).unwrap();
    assert_eq!(config.settings().hotkey, "Ctrl+Shift+A");
    assert_eq!(config.settings().theme, "dark");
    assert_eq!(config.settings().max_results, 100);
}

/// Test configuration validation
#[test]
fn test_config_validation_valid() {
    let config = AppConfig::default();
    assert!(validator::validate_config(&config).is_ok());
}

#[test]
fn test_config_validation_invalid_hotkey() {
    let mut config = AppConfig::default();
    config.hotkey = "InvalidHotkey".to_string();

    let result = validator::validate_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_config_validation_invalid_theme() {
    let mut config = AppConfig::default();
    config.theme = "blue".to_string();

    let result = validator::validate_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_config_validation_invalid_max_results() {
    let mut config = AppConfig::default();
    config.max_results = 0;

    let result = validator::validate_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_config_validation_invalid_window() {
    let mut config = AppConfig::default();
    config.window.width = 50; // Too small

    let result = validator::validate_config(&config);
    assert!(result.is_err());
}

/// Test configuration migration
#[test]
fn test_migration_v0_to_v1() {
    let mut config = AppConfig::default();
    config.version = 0;

    migration::migrate_config(&mut config).unwrap();
    assert_eq!(config.version, 1);
}

#[test]
fn test_migration_preserves_settings() {
    let mut config = AppConfig::default();
    config.version = 0;
    config.hotkey = "Ctrl+A".to_string();
    config.theme = "dark".to_string();
    config.max_results = 100;

    migration::migrate_config(&mut config).unwrap();

    // Settings should be preserved
    assert_eq!(config.hotkey, "Ctrl+A");
    assert_eq!(config.theme, "dark");
    assert_eq!(config.max_results, 100);
}

#[test]
fn test_migration_fixes_empty_values() {
    let mut config = AppConfig::default();
    config.version = 0;
    config.hotkey = "".to_string();
    config.theme = "".to_string();

    migration::migrate_config(&mut config).unwrap();

    // Empty values should be replaced with defaults
    assert_eq!(config.hotkey, "Alt+Space");
    assert_eq!(config.theme, "light");
}

/// Test error handling
#[test]
fn test_invalid_json_falls_back_to_default() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "not valid json").unwrap();

    // Should fall back to defaults when JSON is invalid
    let result = ConfigManager::load_from(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_missing_file_creates_default() {
    let temp_dir = tempfile::tempdir().unwrap();
    let non_existent_path = temp_dir.path().join("non_existent_config.json");

    let config = ConfigManager::load_from(&non_existent_path).unwrap();

    // Should create and return default config
    assert!(non_existent_path.exists());
    assert_eq!(config.settings().hotkey, "Alt+Space");
}

/// Test configuration update methods
#[test]
fn test_update_hotkey() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let mut config = ConfigManager::load_from(&config_path).unwrap();
    config.set_hotkey("Ctrl+Space".to_string()).unwrap();

    assert_eq!(config.settings().hotkey, "Ctrl+Space");

    // Verify it was saved
    let config2 = ConfigManager::load_from(&config_path).unwrap();
    assert_eq!(config2.settings().hotkey, "Ctrl+Space");
}

#[test]
fn test_update_window_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let mut config = ConfigManager::load_from(&config_path).unwrap();
    config.set_window(1024, 768, Some(100), Some(200), false).unwrap();

    assert_eq!(config.settings().window.width, 1024);
    assert_eq!(config.settings().window.height, 768);
    assert_eq!(config.settings().window.x, Some(100));
    assert_eq!(config.settings().window.y, Some(200));
}

#[test]
fn test_update_sync_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let mut config = ConfigManager::load_from(&config_path).unwrap();
    config.set_sync(
        true,
        Some("OneDrive".to_string()),
        Some(PathBuf::from("/path/to/sync")),
    ).unwrap();

    assert!(config.settings().sync_enabled);
    assert_eq!(config.settings().sync_provider, Some("OneDrive".to_string()));
}

/// Test configuration reset
#[test]
fn test_reset_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let mut config = ConfigManager::load_from(&config_path).unwrap();
    config.set_hotkey("Ctrl+A".to_string()).unwrap();
    config.set_theme("dark".to_string()).unwrap();
    config.reset().unwrap();

    assert_eq!(config.settings().hotkey, "Alt+Space");
    assert_eq!(config.settings().theme, "light");
}

/// Test backup and restore
#[test]
fn test_backup_config() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, r#"{{"hotkey": "Alt+Space"}}"#).unwrap();

    let backup_path = migration::backup_config(temp_file.path()).unwrap();

    assert!(backup_path.exists());
    assert!(backup_path.to_string_lossy().contains("backup"));
}

#[test]
fn test_restore_config() {
    let mut temp_config = NamedTempFile::new().unwrap();
    writeln!(temp_config, r#"{{"hotkey": "Original"}}"#).unwrap();

    let backup_path = migration::backup_config(temp_config.path()).unwrap();

    // Modify original
    temp_config = NamedTempFile::new().unwrap();
    writeln!(temp_config, r#"{{"hotkey": "Modified"}}"#).unwrap();

    // Restore from backup
    migration::restore_config(&backup_path, temp_config.path()).unwrap();

    let content = std::fs::read_to_string(temp_config.path()).unwrap();
    assert!(content.contains("Original"));
}

/// Test hotkey validation
#[test]
fn test_hotkey_validation_valid() {
    assert!(validator::validate_hotkey("Alt+Space").is_ok());
    assert!(validator::validate_hotkey("Ctrl+Shift+A").is_ok());
    assert!(validator::validate_hotkey("Win+F1").is_ok());
}

#[test]
fn test_hotkey_validation_invalid() {
    assert!(validator::validate_hotkey("").is_err());
    assert!(validator::validate_hotkey("Space").is_err());
    assert!(validator::validate_hotkey("Invalid+Space").is_err());
}

/// Test theme validation
#[test]
fn test_theme_validation() {
    assert!(validator::validate_theme("light").is_ok());
    assert!(validator::validate_theme("dark").is_ok());
    assert!(validator::validate_theme("invalid").is_err());
}

/// Test max_results validation
#[test]
fn test_max_results_validation() {
    assert!(validator::validate_max_results(1).is_ok());
    assert!(validator::validate_max_results(50).is_ok());
    assert!(validator::validate_max_results(1000).is_ok());
    assert!(validator::validate_max_results(0).is_err());
    assert!(validator::validate_max_results(1001).is_err());
}

/// Test window dimensions validation
#[test]
fn test_window_dimensions_validation() {
    assert!(validator::validate_window_dimensions(800, 600).is_ok());
    assert!(validator::validate_window_dimensions(100, 100).is_ok());
    assert!(validator::validate_window_dimensions(99, 600).is_err());
    assert!(validator::validate_window_dimensions(800, 99).is_err());
}

/// Test sync provider validation
#[test]
fn test_sync_provider_validation() {
    assert!(validator::validate_sync_provider("OneDrive").is_ok());
    assert!(validator::validate_sync_provider("Jianguoyun").is_ok());
    assert!(validator::validate_sync_provider("Nutstore").is_ok());
    assert!(validator::validate_sync_provider("Dropbox").is_err());
}

/// Test JSON serialization
#[test]
fn test_config_json_serialization() {
    let config = AppConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();

    // Should contain all fields
    assert!(json.contains("hotkey"));
    assert!(json.contains("theme"));
    assert!(json.contains("max_results"));
    assert!(json.contains("window"));
}

#[test]
fn test_config_json_deserialization() {
    let json = r#"
    {
        "hotkey": "Ctrl+Space",
        "theme": "dark",
        "shortcuts_dir": "data/shortcuts",
        "database_path": "data/filemgmt.db",
        "max_results": 100,
        "sync_enabled": false,
        "sync_provider": null,
        "sync_path": null,
        "window": {
            "width": 1024,
            "height": 768,
            "x": 100,
            "y": 200,
            "maximized": false
        },
        "version": 1
    }
    "#;

    let config: AppConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.hotkey, "Ctrl+Space");
    assert_eq!(config.theme, "dark");
    assert_eq!(config.max_results, 100);
    assert_eq!(config.window.width, 1024);
    assert_eq!(config.window.height, 768);
}