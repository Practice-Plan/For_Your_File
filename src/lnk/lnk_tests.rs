//! Tests for LNK file operations

#[cfg(test)]
mod tests {
    use crate::lnk::{
        create_lnk_file, parse_lnk_file, validate_lnk_file, LnkBuilder, LnkManager,
        LnkManagerConfig, LnkProperties, ValidationLevel, WindowState,
    };
    use std::fs;
    use std::path::PathBuf;

    /// Helper to create a temporary test directory
    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir().join("lnk_tests");
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
        fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
        temp_dir
    }

    /// Helper to clean up temporary test directory
    fn cleanup_temp_dir(temp_dir: &PathBuf) {
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(temp_dir);
        }
    }

    #[test]
    fn test_lnk_builder_pattern() {
        // Test that builder pattern compiles and can be used
        let _builder = LnkBuilder::new("C:\\Program Files\\App.exe")
            .arguments("--flag --verbose")
            .working_directory("C:\\Program Files")
            .description("Test Application")
            .icon("C:\\Program Files\\App.exe", 0)
            .window_state(WindowState::Maximized);

        // The test verifies the builder pattern compiles and can be used
    }

    #[test]
    fn test_window_state_values() {
        // Test WindowState enum values exist and can be created
        let normal = WindowState::Normal;
        let minimized = WindowState::Minimized;
        let maximized = WindowState::Maximized;

        // Verify they're different
        assert_ne!(normal, minimized);
        assert_ne!(normal, maximized);
        assert_ne!(minimized, maximized);
    }

    #[test]
    fn test_lnk_properties_default() {
        let props = LnkProperties::default();
        assert!(props.target_path.is_empty());
        assert!(props.arguments.is_none());
        assert!(props.working_directory.is_none());
        assert!(props.description.is_none());
        assert!(props.icon_location.is_none());
        assert!(props.icon_index.is_none());
        assert_eq!(props.show_command, Some(1));
    }

    #[test]
    fn test_lnk_manager_creation() {
        let manager = LnkManager::new();
        assert_eq!(
            manager.config().default_validation_level,
            ValidationLevel::Standard
        );
        assert!(manager.config().default_directory.is_none());
        assert!(!manager.config().overwrite_existing);
    }

    #[test]
    fn test_lnk_manager_with_config() {
        let config = LnkManagerConfig {
            default_directory: Some(PathBuf::from("C:\\Shortcuts")),
            default_validation_level: ValidationLevel::Full,
            overwrite_existing: true,
        };

        let manager = LnkManager::with_config(config);
        assert!(manager.config().default_directory.is_some());
        assert_eq!(
            manager.config().default_validation_level,
            ValidationLevel::Full
        );
        assert!(manager.config().overwrite_existing);
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_lnk_file("/nonexistent/path/file.lnk");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn test_validation_nonexistent_file() {
        let result = validate_lnk_file("/nonexistent/path/file.lnk", ValidationLevel::Basic);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("does not exist")));
    }

    #[test]
    fn test_manager_read_nonexistent() {
        let manager = LnkManager::new();
        let result = manager.read_shortcut("/nonexistent/path.lnk");
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_delete_nonexistent() {
        let manager = LnkManager::new();
        let result = manager.delete_shortcut("/nonexistent/path.lnk");
        assert!(result.is_err());
    }

    #[cfg(windows)]
    #[test]
    fn test_create_and_parse_lnk() {
        let temp_dir = create_temp_dir();
        let lnk_path = temp_dir.join("test_shortcut.lnk");

        // Use notepad.exe as a target since it's guaranteed to exist on Windows
        let target = "C:\\Windows\\System32\\notepad.exe";

        // Create the shortcut
        let result = create_lnk_file(
            &lnk_path,
            target,
            Some("--test-arg"),
            Some("C:\\"),
            Some("Test Notepad Shortcut"),
            None,
            None,
            WindowState::Normal,
        );

        assert!(result.is_ok(), "Failed to create LNK file");

        // Verify the file was created
        assert!(lnk_path.exists(), "LNK file was not created");

        // Parse the shortcut
        let parsed = parse_lnk_file(&lnk_path);
        assert!(parsed.is_ok(), "Failed to parse LNK file");

        let props = parsed.unwrap();
        assert_eq!(props.target_path, target);
        assert_eq!(props.arguments, Some("--test-arg".to_string()));
        assert_eq!(props.working_directory, Some("C:\\".to_string()));
        assert_eq!(props.description, Some("Test Notepad Shortcut".to_string()));

        // Clean up
        cleanup_temp_dir(&temp_dir);
    }

    #[cfg(windows)]
    #[test]
    fn test_lnk_builder_build() {
        let temp_dir = create_temp_dir();
        let lnk_path = temp_dir.join("builder_test.lnk");

        let target = "C:\\Windows\\System32\\calc.exe";

        let result = LnkBuilder::new(target)
            .arguments("--test")
            .description("Calculator Test")
            .window_state(WindowState::Maximized)
            .build(&lnk_path);

        assert!(result.is_ok(), "Builder failed to create LNK file");
        assert!(lnk_path.exists(), "LNK file was not created by builder");

        // Parse and verify
        let props = parse_lnk_file(&lnk_path).expect("Failed to parse");
        assert_eq!(props.target_path, target);
        assert_eq!(props.arguments, Some("--test".to_string()));

        cleanup_temp_dir(&temp_dir);
    }

    #[cfg(windows)]
    #[test]
    fn test_manager_operations() {
        let temp_dir = create_temp_dir();
        let manager = LnkManager::new();

        let lnk_path = temp_dir.join("manager_test.lnk");
        let target = "C:\\Windows\\System32\\cmd.exe";

        // Create
        let created = manager.create_shortcut(
            &lnk_path,
            target,
            Some("/c echo test"),
            Some("C:\\"),
            Some("CMD Test"),
        );

        assert!(created.is_ok(), "Manager failed to create shortcut");
        assert!(lnk_path.exists());

        // Read
        let props = manager.read_shortcut(&lnk_path).expect("Failed to read shortcut");
        assert_eq!(props.target_path, target);
        assert_eq!(props.arguments, Some("/c echo test".to_string()));

        // Validate
        let validation = manager.validate_shortcut(&lnk_path, Some(ValidationLevel::Standard));
        assert!(validation.is_valid);

        // Delete
        let deleted = manager.delete_shortcut(&lnk_path);
        assert!(deleted.is_ok(), "Failed to delete shortcut");
        assert!(!lnk_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[cfg(windows)]
    #[test]
    fn test_update_shortcut_target() {
        let temp_dir = create_temp_dir();
        let manager = LnkManager::new();

        let lnk_path = temp_dir.join("update_test.lnk");
        let original_target = "C:\\Windows\\System32\\notepad.exe";
        let new_target = "C:\\Windows\\System32\\calc.exe";

        // Create initial shortcut
        manager
            .create_shortcut(&lnk_path, original_target, None, None, None)
            .expect("Failed to create");

        let props = manager.read_shortcut(&lnk_path).expect("Failed to read");
        assert_eq!(props.target_path, original_target);

        // Update target
        use crate::lnk::update_lnk_target;
        update_lnk_target(&lnk_path, new_target).expect("Failed to update target");

        // Verify update
        let updated_props = manager.read_shortcut(&lnk_path).expect("Failed to read updated");
        assert_eq!(updated_props.target_path, new_target);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_validation_levels() {
        let basic_result = validate_lnk_file("/nonexistent.lnk", ValidationLevel::Basic);
        assert!(!basic_result.is_valid);

        let standard_result = validate_lnk_file("/nonexistent.lnk", ValidationLevel::Standard);
        assert!(!standard_result.is_valid);

        let full_result = validate_lnk_file("/nonexistent.lnk", ValidationLevel::Full);
        assert!(!full_result.is_valid);
    }

    #[test]
    fn test_global_manager() {
        let manager = crate::lnk::global_manager();
        assert_eq!(
            manager.config().default_validation_level,
            ValidationLevel::Standard
        );
    }

    #[test]
    fn test_generate_lnk_path() {
        let temp_dir = create_temp_dir();
        let mut manager = LnkManager::new();
        manager.set_default_directory(temp_dir.clone());

        let path1 = manager.generate_lnk_path("test").expect("Failed to generate path");
        assert!(path1.ends_with("test.lnk"));

        // Create the first file
        fs::write(&path1, "").expect("Failed to write test file");

        // Generate again - should get a unique path
        let path2 = manager.generate_lnk_path("test").expect("Failed to generate path");
        assert_ne!(path1, path2);

        cleanup_temp_dir(&temp_dir);
    }
}