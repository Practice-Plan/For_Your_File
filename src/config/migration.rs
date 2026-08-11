//! Configuration migration
//!
//! Handles migrating configuration from older versions to newer versions.

use crate::models::AppConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Current configuration version
const CURRENT_VERSION: u32 = 1;

/// Migrate configuration to current version
///
/// Handles version changes and updates old settings to new format
/// while preserving user preferences.
pub fn migrate_config(config: &mut AppConfig) -> Result<()> {
    let original_version = config.version;

    // No migration needed if already at current version
    if original_version >= CURRENT_VERSION {
        return Ok(());
    }

    log::info!(
        "Migrating configuration from version {} to {}",
        original_version,
        CURRENT_VERSION
    );

    // Apply migrations sequentially
    if original_version < 1 {
        migrate_v0_to_v1(config)?;
    }

    // Update version to current
    config.version = CURRENT_VERSION;

    Ok(())
}

/// Migrate from version 0 (no version field) to version 1
///
/// This handles the initial structured configuration format.
fn migrate_v0_to_v1(config: &mut AppConfig) -> Result<()> {
    // Ensure all new fields have defaults
    if config.hotkey.is_empty() {
        config.hotkey = "Alt+Space".to_string();
    }

    if config.theme.is_empty() {
        config.theme = "light".to_string();
    }

    // Set default max_results if not set (unlikely for v0)
    if config.max_results == 0 {
        config.max_results = 50;
    }

    log::debug!("Migrated configuration from v0 to v1");
    Ok(())
}

/// Backup configuration file before migration
///
/// Creates a backup with .backup extension.
/// Returns the backup path.
pub fn backup_config<P: AsRef<Path>>(config_path: P) -> Result<std::path::PathBuf> {
    let config_path = config_path.as_ref();
    let backup_path = config_path.with_extension("json.backup");

    // Remove existing backup if present
    if backup_path.exists() {
        fs::remove_file(&backup_path)
            .with_context(|| format!("Failed to remove old backup: {}", backup_path.display()))?;
    }

    // Create backup
    fs::copy(config_path, &backup_path)
        .with_context(|| {
            format!(
                "Failed to backup config from {} to {}",
                config_path.display(),
                backup_path.display()
            )
        })?;

    log::info!("Configuration backed up to {}", backup_path.display());
    Ok(backup_path)
}

/// Restore configuration from backup
///
/// Restores configuration from a backup file.
pub fn restore_config<P1: AsRef<Path>, P2: AsRef<Path>>(backup_path: P1, config_path: P2) -> Result<()> {
    let backup_path = backup_path.as_ref();
    let config_path = config_path.as_ref();

    if !backup_path.exists() {
        anyhow::bail!("Backup file does not exist: {}", backup_path.display());
    }

    // Restore from backup
    fs::copy(backup_path, config_path).with_context(|| {
        format!(
            "Failed to restore config from {} to {}",
            backup_path.display(),
            config_path.display()
        )
    })?;

    log::info!("Configuration restored from {}", backup_path.display());
    Ok(())
}

/// Perform safe migration with backup
///
/// Creates a backup before migration and restores it if migration fails.
pub fn safe_migrate_config<P: AsRef<Path>>(
    config_path: P,
    config: &mut AppConfig,
) -> Result<()> {
    let config_path = config_path.as_ref();

    // Create backup if config exists
    let backup_path = if config_path.exists() {
        Some(backup_config(config_path)?)
    } else {
        None
    };

    // Attempt migration
    let result = migrate_config(config);

    // Restore backup if migration failed
    if result.is_err() {
        if let Some(ref backup) = backup_path {
            log::error!("Migration failed, restoring from backup");
            restore_config(backup, config_path)?;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_migrate_v0_to_v1() {
        let mut config = AppConfig::default();
        config.version = 0;

        migrate_config(&mut config).unwrap();
        assert_eq!(config.version, CURRENT_VERSION);
    }

    #[test]
    fn test_no_migration_needed() {
        let mut config = AppConfig::default();
        config.version = CURRENT_VERSION;

        // Should not change anything
        migrate_config(&mut config).unwrap();
        assert_eq!(config.version, CURRENT_VERSION);
    }

    #[test]
    fn test_migrate_empty_hotkey() {
        let mut config = AppConfig::default();
        config.version = 0;
        config.hotkey = "".to_string();

        migrate_config(&mut config).unwrap();
        assert_eq!(config.hotkey, "Alt+Space");
    }

    #[test]
    fn test_migrate_empty_theme() {
        let mut config = AppConfig::default();
        config.version = 0;
        config.theme = "".to_string();

        migrate_config(&mut config).unwrap();
        assert_eq!(config.theme, "light");
    }

    #[test]
    fn test_backup_and_restore() {
        let mut temp_config = NamedTempFile::new().unwrap();
        writeln!(temp_config, r#"{{"hotkey": "Alt+Space"}}"#).unwrap();
        temp_config.flush().unwrap();

        let config_path = temp_config.path().to_path_buf();
        let backup_path = backup_config(&config_path).unwrap();

        assert!(backup_path.exists());

        // Modify original
        temp_config = NamedTempFile::new().unwrap();
        writeln!(temp_config, r#"{{"hotkey": "Ctrl+A"}}"#).unwrap();
        temp_config.flush().unwrap();

        let config_path = temp_config.path().to_path_buf();
        restore_config(&backup_path, &config_path).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Alt+Space"));
    }

    #[test]
    fn test_safe_migrate() {
        let mut temp_config = NamedTempFile::new().unwrap();
        writeln!(temp_config, r#"{{"version": 0}}"#).unwrap();

        let config_path = temp_config.path();
        let mut config = AppConfig::default();
        config.version = 0;

        safe_migrate_config(config_path, &mut config).unwrap();
        assert_eq!(config.version, CURRENT_VERSION);
    }
}