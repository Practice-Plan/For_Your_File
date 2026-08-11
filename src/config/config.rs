//! Configuration loading, saving, and management
//!
//! Handles loading configuration from platform-specific locations,
//! saving changes, and managing configuration lifecycle.

use crate::config::validator::{validate_config, ValidationError};
use crate::config::migration::migrate_config;
use crate::models::AppConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration management errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Configuration validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),

    #[error("Failed to write configuration: {0}")]
    WriteError(String),
}

/// Configuration manager
///
/// Handles loading, saving, and updating application configuration.
/// Configuration is stored in JSON format for human readability.
pub struct ConfigManager {
    /// Current configuration settings
    settings: AppConfig,
    /// Path to configuration file
    path: PathBuf,
}

impl ConfigManager {
    /// Get the platform-specific configuration directory
    ///
    /// Windows: %APPDATA%/FileManagementCenter
    /// Creates the directory if it doesn't exist.
    pub fn get_config_dir() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "FileManagementCenter", "FileManagementCenter")
            .context("Failed to determine configuration directory")?;

        let config_dir = dirs.config_dir().to_path_buf();

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory: {}", config_dir.display()))?;
        }

        Ok(config_dir)
    }

    /// Get the platform-specific configuration file path
    ///
    /// Returns: %APPDATA%/FileManagementCenter/config.json on Windows
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = Self::get_config_dir()?;
        Ok(config_dir.join("config.json"))
    }

    /// Get the data directory path
    ///
    /// Returns the directory where database and shortcuts are stored.
    pub fn get_data_dir() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "FileManagementCenter", "FileManagementCenter")
            .context("Failed to determine data directory")?;

        let data_dir = dirs.data_dir().to_path_buf();

        // Create directory if it doesn't exist
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)
                .with_context(|| format!("Failed to create data directory: {}", data_dir.display()))?;
        }

        Ok(data_dir)
    }

    /// Load configuration from the default platform-specific path
    ///
    /// If configuration doesn't exist, creates a default one.
    /// If configuration is invalid, uses defaults and logs a warning.
    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;
        Self::load_from(&path)
    }

    /// Load configuration from a specific path
    ///
    /// Creates default configuration if file doesn't exist.
    /// Validates and migrates configuration from older versions.
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if path.exists() {
            // Read existing configuration
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from {}", path.display()))?;

            // Parse JSON
            let mut settings: AppConfig = serde_json::from_str(&content)
                .with_context(|| {
                    format!("Failed to parse config JSON from {}", path.display())
                })?;

            // Run migration if needed
            migrate_config(&mut settings)?;

            // Validate configuration, use defaults for invalid fields
            if let Err(e) = validate_config(&settings) {
                log::warn!("Configuration validation failed, using defaults for invalid fields: {}", e);
                // Apply defaults for invalid fields
                let default = AppConfig::default();
                if validate_config(&settings).is_err() {
                    // If validation still fails, use defaults for specific fields
                    if !validate_config(&default).is_err() {
                        settings = default;
                    }
                }
            }

            Ok(Self { settings, path })
        } else {
            // Create default configuration
            log::info!("Configuration file not found, creating default at {}", path.display());
            let config = Self {
                settings: AppConfig::default(),
                path,
            };
            config.save()?;
            Ok(config)
        }
    }

    /// Save the current configuration to disk
    ///
    /// Serializes configuration to pretty-printed JSON.
    /// Creates parent directories if they don't exist.
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        // Serialize to pretty JSON
        let content = serde_json::to_string_pretty(&self.settings)
            .context("Failed to serialize configuration")?;

        // Write to file
        std::fs::write(&self.path, content)
            .with_context(|| format!("Failed to write config to {}", self.path.display()))?;

        log::info!("Configuration saved to {}", self.path.display());
        Ok(())
    }

    /// Get a reference to the current settings
    pub fn settings(&self) -> &AppConfig {
        &self.settings
    }

    /// Get a mutable reference to the settings
    pub fn settings_mut(&mut self) -> &mut AppConfig {
        &mut self.settings
    }

    /// Update configuration with new settings
    ///
    /// Validates the changes before saving.
    pub fn update<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        f(&mut self.settings);

        // Validate updated configuration
        validate_config(&self.settings)?;

        // Save to disk
        self.save()
    }

    /// Update hotkey configuration
    pub fn set_hotkey(&mut self, hotkey: String) -> Result<()> {
        self.update(|s| s.hotkey = hotkey)
    }

    /// Update theme configuration
    pub fn set_theme(&mut self, theme: String) -> Result<()> {
        self.update(|s| s.theme = theme)
    }

    /// Update shortcuts directory
    pub fn set_shortcuts_dir(&mut self, path: PathBuf) -> Result<()> {
        self.update(|s| s.shortcuts_dir = path)
    }

    /// Update database path
    pub fn set_database_path(&mut self, path: PathBuf) -> Result<()> {
        self.update(|s| s.database_path = path)
    }

    /// Update max results setting
    pub fn set_max_results(&mut self, max_results: usize) -> Result<()> {
        self.update(|s| s.max_results = max_results)
    }

    /// Update window configuration
    pub fn set_window(&mut self, width: u32, height: u32, x: Option<i32>, y: Option<i32>, maximized: bool) -> Result<()> {
        self.update(|s| {
            s.window.width = width;
            s.window.height = height;
            s.window.x = x;
            s.window.y = y;
            s.window.maximized = maximized;
        })
    }

    /// Reset configuration to defaults
    ///
    /// Preserves the configuration file path.
    pub fn reset(&mut self) -> Result<()> {
        self.settings = AppConfig::default();
        self.save()
    }

    /// Get the configuration file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Display for ConfigManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConfigManager(path={})", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_default_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_file_name("new_config.json");

        let config = ConfigManager::load_from(&path).unwrap();
        assert_eq!(config.settings().hotkey, "Alt+Space");
        assert_eq!(config.settings().theme, "light");
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_file_name("test_config.json");

        // Save a config
        {
            let mut config = ConfigManager::load_from(&path).unwrap();
            config.set_hotkey("Ctrl+Shift+A".to_string()).unwrap();
            config.set_theme("dark".to_string()).unwrap();
        }

        // Load and verify
        let config = ConfigManager::load_from(&path).unwrap();
        assert_eq!(config.settings().hotkey, "Ctrl+Shift+A");
        assert_eq!(config.settings().theme, "dark");
    }

    #[test]
    fn test_reset_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_file_name("reset_config.json");

        let mut config = ConfigManager::load_from(&path).unwrap();
        config.set_hotkey("Ctrl+A".to_string()).unwrap();
        config.reset().unwrap();

        assert_eq!(config.settings().hotkey, "Alt+Space");
    }

    #[test]
    fn test_update_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_file_name("update_config.json");

        let mut config = ConfigManager::load_from(&path).unwrap();
        config.update(|s| {
            s.max_results = 100;
            s.window.width = 1024;
            s.window.height = 768;
        }).unwrap();

        assert_eq!(config.settings().max_results, 100);
        assert_eq!(config.settings().window.width, 1024);
        assert_eq!(config.settings().window.height, 768);
    }
}