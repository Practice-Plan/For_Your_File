//! Configuration models

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::sorting::{SortMethod, SortingWeights};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Hotkey string (e.g., "Alt+Space")
    pub hotkey: String,
    /// UI theme ("light" or "dark")
    pub theme: String,
    /// Directory for shortcuts storage
    #[serde(default = "default_shortcuts_dir")]
    pub shortcuts_dir: PathBuf,
    /// Database file path
    #[serde(default = "default_database_path")]
    pub database_path: PathBuf,
    /// Maximum search results to display
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    /// Window configuration
    #[serde(default)]
    pub window: WindowConfig,
    /// Sorting configuration
    #[serde(default)]
    pub sorting: SortingConfig,
    /// Configuration version for migration
    #[serde(default = "default_config_version")]
    pub version: u32,
}

/// Window position and size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width in pixels
    #[serde(default = "default_window_width")]
    pub width: u32,
    /// Window height in pixels
    #[serde(default = "default_window_height")]
    pub height: u32,
    /// Window X position (None for centered)
    pub x: Option<i32>,
    /// Window Y position (None for centered)
    pub y: Option<i32>,
    /// Whether window is maximized
    #[serde(default)]
    pub maximized: bool,
}

fn default_shortcuts_dir() -> PathBuf {
    PathBuf::from("data/shortcuts")
}

fn default_database_path() -> PathBuf {
    PathBuf::from("data/filemgmt.db")
}

fn default_max_results() -> usize {
    50
}

fn default_config_version() -> u32 {
    1
}

fn default_window_width() -> u32 {
    800
}

fn default_window_height() -> u32 {
    600
}

fn default_frequency_half_life() -> f32 {
    7.0 // 7 days
}

/// Sorting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortingConfig {
    /// Current sort method
    #[serde(default)]
    pub method: SortMethod,
    /// Custom weights for sorting
    #[serde(default)]
    pub weights: SortingWeights,
    /// Frequency decay half-life in days
    #[serde(default = "default_frequency_half_life")]
    pub frequency_half_life: f32,
    /// Enable debug mode to show score breakdown
    #[serde(default)]
    pub debug_mode: bool,
}

impl Default for SortingConfig {
    fn default() -> Self {
        Self {
            method: SortMethod::Relevance,
            weights: SortingWeights::default(),
            frequency_half_life: default_frequency_half_life(),
            debug_mode: false,
        }
    }
}

impl SortingConfig {
    /// Create a new sorting configuration with custom weights
    pub fn with_weights(weights: SortingWeights) -> Self {
        Self {
            method: SortMethod::Custom,
            weights,
            frequency_half_life: default_frequency_half_life(),
            debug_mode: false,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        self.weights.validate()?;

        if self.frequency_half_life <= 0.0 {
            return Err("Frequency half-life must be positive".to_string());
        }

        Ok(())
    }

    /// Set the sort method
    pub fn set_method(&mut self, method: SortMethod) {
        self.method = method;
    }

    /// Set the frequency weight
    pub fn set_frequency_weight(&mut self, weight: f32) {
        self.weights.set_frequency(weight);
    }

    /// Set the recency weight
    pub fn set_recency_weight(&mut self, weight: f32) {
        self.weights.set_recency(weight);
    }

    /// Set the relevance weight
    pub fn set_relevance_weight(&mut self, weight: f32) {
        self.weights.set_relevance(weight);
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Space".to_string(),
            theme: "light".to_string(),
            shortcuts_dir: default_shortcuts_dir(),
            database_path: default_database_path(),
            max_results: default_max_results(),
            window: WindowConfig::default(),
            sorting: SortingConfig::default(),
            version: default_config_version(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_window_width(),
            height: default_window_height(),
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// Hotkey configuration (legacy compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Main hotkey modifiers (e.g., "ALT")
    pub modifiers: String,
    /// Main hotkey key (e.g., "SPACE")
    pub key: String,
    /// Whether hotkey is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// UI configuration (legacy compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Window width
    pub window_width: i32,
    /// Window height
    pub window_height: i32,
    /// Theme ("light" or "dark")
    pub theme: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.hotkey, "Alt+Space");
        assert_eq!(config.theme, "light");
        assert_eq!(config.max_results, 50);
    }

    #[test]
    fn test_default_window_config() {
        let window = WindowConfig::default();
        assert_eq!(window.width, 800);
        assert_eq!(window.height, 600);
        assert!(window.x.is_none());
        assert!(window.y.is_none());
        assert!(!window.maximized);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.hotkey, deserialized.hotkey);
        assert_eq!(config.theme, deserialized.theme);
    }
}