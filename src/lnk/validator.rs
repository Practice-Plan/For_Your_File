//! LNK file validation module
//!
//! Provides validation for LNK shortcut files including file existence,
//! format validation, and target path validation.

use crate::lnk::{parse_lnk_file, LnkProperties};
use anyhow::Result;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during LNK file validation
#[derive(Debug, Error)]
pub enum LnkValidationError {
    #[error("LNK file does not exist: {0}")]
    FileNotFound(String),

    #[error("LNK file is corrupted: {0}")]
    CorruptedFile(String),

    #[error("Target path does not exist: {0}")]
    TargetNotFound(String),

    #[error("Invalid LNK file format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Working directory does not exist: {0}")]
    WorkingDirNotFound(String),

    #[error("Icon location does not exist: {0}")]
    IconNotFound(String),

    #[error("Validation error: {0}")]
    Other(String),
}

/// Validation level for LNK files
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ValidationLevel {
    /// Basic validation: file exists and is readable
    Basic,
    /// Standard validation: basic + target path exists
    Standard,
    /// Full validation: all fields validated
    Full,
}

impl Default for ValidationLevel {
    fn default() -> Self {
        ValidationLevel::Standard
    }
}

/// Result of LNK file validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the LNK file is valid
    pub is_valid: bool,
    /// Parsed properties (if successfully parsed)
    pub properties: Option<LnkProperties>,
    /// List of validation errors
    pub errors: Vec<String>,
    /// List of validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            properties: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
        self.is_valid = false;
    }

    /// Add a warning to the validation result
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a LNK file at the specified path
pub fn validate_lnk_file<P: AsRef<Path>>(
    lnk_path: P,
    level: ValidationLevel,
) -> ValidationResult {
    let lnk_path = lnk_path.as_ref();
    let mut result = ValidationResult::new();

    // Check if file exists
    if !lnk_path.exists() {
        result.add_error(format!("LNK file does not exist: {}", lnk_path.display()));
        return result;
    }

    // Check file extension
    if lnk_path.extension().map_or(true, |ext| ext != "lnk") {
        result.add_error(format!(
            "Invalid file extension: expected .lnk, got {:?}",
            lnk_path.extension()
        ));
        return result;
    }

    // Parse the LNK file
    let props = match parse_lnk_file(lnk_path) {
        Ok(props) => props,
        Err(e) => {
            result.add_error(format!("Failed to parse LNK file: {}", e));
            return result;
        }
    };

    result.properties = Some(props.clone());

    // Validate required fields
    if props.target_path.is_empty() {
        result.add_error("Target path is empty");
    }

    // Standard validation: check target path exists
    if level >= ValidationLevel::Standard {
        if !props.target_path.is_empty() {
            let target_path = Path::new(&props.target_path);
            if !target_path.exists() {
                result.add_warning(format!(
                    "Target path does not exist: {}",
                    props.target_path
                ));
            }
        }
    }

    // Full validation: check all referenced paths
    if level >= ValidationLevel::Full {
        // Validate working directory
        if let Some(ref workdir) = props.working_directory {
            if !workdir.is_empty() {
                let workdir_path = Path::new(workdir);
                if !workdir_path.exists() {
                    result.add_warning(format!("Working directory does not exist: {}", workdir));
                }
            }
        }

        // Validate icon location
        if let Some(ref icon_loc) = props.icon_location {
            if !icon_loc.is_empty() {
                let icon_path = Path::new(icon_loc);
                if !icon_path.exists() {
                    result.add_warning(format!("Icon location does not exist: {}", icon_loc));
                }
            }
        }

        // Validate show command
        if let Some(show_cmd) = props.show_command {
            match show_cmd {
                1 | 3 | 7 => {}, // Valid values
                _ => result.add_warning(format!("Invalid show command: {}", show_cmd)),
            }
        }
    }

    result
}

/// Quick validation check - returns true if valid, false otherwise
pub fn is_valid_lnk<P: AsRef<Path>>(lnk_path: P) -> bool {
    let result = validate_lnk_file(lnk_path, ValidationLevel::Standard);
    result.is_valid
}

/// Check if a target path exists
pub fn target_path_exists(target_path: &str) -> bool {
    if target_path.is_empty() {
        return false;
    }
    Path::new(target_path).exists()
}

/// Validate that a path can be used as a target for a shortcut
pub fn validate_target_path<P: AsRef<Path>>(target_path: P) -> Result<()> {
    let target_path = target_path.as_ref();

    if !target_path.exists() {
        anyhow::bail!("Target path does not exist: {}", target_path.display());
    }

    // On Windows, we can create shortcuts to files, folders, or URLs
    // No additional validation needed beyond existence check
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        result.add_error("Test error");
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        result.add_warning("Test warning");
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_nonexistent_file() {
        let result = validate_lnk_file("/nonexistent/path/file.lnk", ValidationLevel::Basic);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("does not exist")));
    }

    #[test]
    fn test_target_path_exists() {
        assert!(!target_path_exists(""));
        assert!(!target_path_exists("/nonexistent/path"));
    }
}