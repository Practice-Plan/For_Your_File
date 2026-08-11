//! LNK file manager for coordinating all LNK operations
//!
//! Provides a high-level API for managing Windows .lnk shortcut files.

use crate::lnk::{
    create_lnk_file, parse_lnk_file, update_lnk_target, validate_lnk_file, LnkBuilder,
    LnkProperties, ValidationResult, ValidationLevel, WindowState,
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur in LNK file management
#[derive(Debug, Error)]
pub enum LnkManagerError {
    #[error("Failed to create LNK file: {0}")]
    CreateFailed(String),

    #[error("Failed to read LNK file: {0}")]
    ReadFailed(String),

    #[error("Failed to update LNK file: {0}")]
    UpdateFailed(String),

    #[error("Failed to delete LNK file: {0}")]
    DeleteFailed(String),

    #[error("LNK file validation failed: {0}")]
    ValidationFailed(String),

    #[error("LNK file already exists: {0}")]
    AlreadyExists(String),

    #[error("LNK file not found: {0}")]
    NotFound(String),
}

/// Configuration for LNK file manager
#[derive(Debug, Clone)]
pub struct LnkManagerConfig {
    /// Default directory for creating new shortcuts
    pub default_directory: Option<PathBuf>,
    /// Default validation level for checking shortcuts
    pub default_validation_level: ValidationLevel,
    /// Whether to overwrite existing files by default
    pub overwrite_existing: bool,
}

impl Default for LnkManagerConfig {
    fn default() -> Self {
        Self {
            default_directory: None,
            default_validation_level: ValidationLevel::Standard,
            overwrite_existing: false,
        }
    }
}

/// Manager for LNK file operations
#[derive(Debug)]
pub struct LnkManager {
    config: LnkManagerConfig,
}

impl LnkManager {
    /// Create a new LNK manager with default configuration
    pub fn new() -> Self {
        Self {
            config: LnkManagerConfig::default(),
        }
    }

    /// Create a new LNK manager with custom configuration
    pub fn with_config(config: LnkManagerConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &LnkManagerConfig {
        &self.config
    }

    /// Create a new shortcut
    pub fn create_shortcut<P: AsRef<Path>>(
        &self,
        lnk_path: P,
        target_path: &str,
        arguments: Option<&str>,
        working_directory: Option<&str>,
        description: Option<&str>,
    ) -> Result<PathBuf> {
        let lnk_path = lnk_path.as_ref();

        // Check if file already exists
        if lnk_path.exists() && !self.config.overwrite_existing {
            return Err(LnkManagerError::AlreadyExists(lnk_path.display().to_string()).into());
        }

        // Create the shortcut
        create_lnk_file(
            lnk_path,
            target_path,
            arguments,
            working_directory,
            description,
            None, // icon_location
            None, // icon_index
            WindowState::Normal,
        )
        .context(format!("Failed to create shortcut: {}", lnk_path.display()))?;

        Ok(lnk_path.to_path_buf())
    }

    /// Create a shortcut using the builder pattern
    pub fn create_shortcut_builder(&self, target_path: impl Into<String>) -> LnkBuilder {
        LnkBuilder::new(target_path)
    }

    /// Read an existing shortcut
    pub fn read_shortcut<P: AsRef<Path>>(&self, lnk_path: P) -> Result<LnkProperties> {
        let lnk_path = lnk_path.as_ref();

        if !lnk_path.exists() {
            return Err(LnkManagerError::NotFound(lnk_path.display().to_string()).into());
        }

        parse_lnk_file(lnk_path)
            .context(format!("Failed to read shortcut: {}", lnk_path.display()))
    }

    /// Update an existing shortcut's target path
    pub fn update_shortcut<P: AsRef<Path>>(&self, lnk_path: P, new_target: &str) -> Result<()> {
        let lnk_path = lnk_path.as_ref();

        if !lnk_path.exists() {
            return Err(LnkManagerError::NotFound(lnk_path.display().to_string()).into());
        }

        update_lnk_target(lnk_path, new_target)
            .context(format!("Failed to update shortcut: {}", lnk_path.display()))
    }

    /// Delete a shortcut file
    pub fn delete_shortcut<P: AsRef<Path>>(&self, lnk_path: P) -> Result<()> {
        let lnk_path = lnk_path.as_ref();

        if !lnk_path.exists() {
            return Err(LnkManagerError::NotFound(lnk_path.display().to_string()).into());
        }

        std::fs::remove_file(lnk_path)
            .context(format!("Failed to delete shortcut: {}", lnk_path.display()))?;

        log::info!("Deleted shortcut: {}", lnk_path.display());
        Ok(())
    }

    /// Validate a shortcut file
    pub fn validate_shortcut<P: AsRef<Path>>(
        &self,
        lnk_path: P,
        level: Option<ValidationLevel>,
    ) -> ValidationResult {
        let level = level.unwrap_or(self.config.default_validation_level);
        validate_lnk_file(lnk_path, level)
    }

    /// Check if a shortcut is valid
    pub fn is_shortcut_valid<P: AsRef<Path>>(&self, lnk_path: P) -> bool {
        let result = self.validate_shortcut(lnk_path, None);
        result.is_valid
    }

    /// Copy a shortcut to a new location
    pub fn copy_shortcut<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        source: P,
        destination: Q,
    ) -> Result<PathBuf> {
        let source = source.as_ref();
        let destination = destination.as_ref();

        if !source.exists() {
            return Err(LnkManagerError::NotFound(source.display().to_string()).into());
        }

        // Read the source shortcut
        let props = self.read_shortcut(source)?;

        // Create the shortcut at the destination
        create_lnk_file(
            destination,
            &props.target_path,
            props.arguments.as_deref(),
            props.working_directory.as_deref(),
            props.description.as_deref(),
            props.icon_location.as_deref(),
            props.icon_index,
            WindowState::Normal,
        )
        .context(format!(
            "Failed to copy shortcut to: {}",
            destination.display()
        ))?;

        Ok(destination.to_path_buf())
    }

    /// Move a shortcut to a new location
    pub fn move_shortcut<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        source: P,
        destination: Q,
    ) -> Result<PathBuf> {
        let destination = self.copy_shortcut(&source, &destination)?;
        self.delete_shortcut(source)?;
        Ok(destination)
    }

    /// Get the default directory for shortcuts
    pub fn get_default_directory(&self) -> Option<&Path> {
        self.config.default_directory.as_deref()
    }

    /// Set the default directory for shortcuts
    pub fn set_default_directory(&mut self, directory: PathBuf) {
        self.config.default_directory = Some(directory);
    }

    /// Generate a unique LNK file path in the default directory
    pub fn generate_lnk_path(&self, name: &str) -> Result<PathBuf> {
        let base_dir = self
            .config
            .default_directory
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No default directory configured"))?;

        let mut lnk_path = base_dir.join(name).with_extension("lnk");

        // Ensure unique filename
        let mut counter = 1;
        while lnk_path.exists() {
            lnk_path = base_dir
                .join(format!("{} ({})", name, counter))
                .with_extension("lnk");
            counter += 1;
        }

        Ok(lnk_path)
    }
}

impl Default for LnkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global LNK manager instance (lazy initialized)
static LNK_MANAGER: std::sync::OnceLock<LnkManager> = std::sync::OnceLock::new();

/// Get the global LNK manager instance
pub fn global_manager() -> &'static LnkManager {
    LNK_MANAGER.get_or_init(LnkManager::new)
}

/// Initialize the global LNK manager with custom configuration
pub fn init_global_manager(config: LnkManagerConfig) -> Result<()> {
    LNK_MANAGER
        .set(LnkManager::with_config(config))
        .map_err(|_| anyhow::anyhow!("Global LNK manager already initialized"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = LnkManager::new();
        assert_eq!(
            manager.config().default_validation_level,
            ValidationLevel::Standard
        );
    }

    #[test]
    fn test_manager_with_config() {
        let config = LnkManagerConfig {
            default_directory: Some(PathBuf::from("/tmp")),
            default_validation_level: ValidationLevel::Full,
            overwrite_existing: true,
        };
        let manager = LnkManager::with_config(config);
        assert_eq!(
            manager.config().default_validation_level,
            ValidationLevel::Full
        );
    }

    #[test]
    fn test_nonexistent_shortcut() {
        let manager = LnkManager::new();
        let result = manager.read_shortcut("/nonexistent/path.lnk");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent() {
        let manager = LnkManager::new();
        let result = manager.delete_shortcut("/nonexistent/path.lnk");
        assert!(result.is_err());
    }
}