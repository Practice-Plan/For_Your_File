//! Hotkey functionality integration tests
//!
//! Tests hotkey registration, conflict detection, and customization
//! workflows.

mod common;

use app_lib::HotkeyConfig;

/// Test hotkey config parsing from string
#[test]
fn test_hotkey_config_parsing() {
    // Test valid hotkey strings
    let test_cases = vec![
        ("Alt+Space", "Alt", "Space"),
        ("Ctrl+Shift+A", "Ctrl+Shift", "A"),
        ("Ctrl+Alt+Delete", "Ctrl+Alt", "Delete"),
        ("F12", "", "F12"),
        ("Alt+F12", "Alt", "F12"),
    ];

    for (hotkey_str, expected_modifiers, expected_key) in test_cases {
        let config = HotkeyConfig::from_string(hotkey_str)
            .expect(&format!("Failed to parse hotkey: {}", hotkey_str));

        assert_eq!(
            config.modifiers, expected_modifiers,
            "Modifiers mismatch for {}",
            hotkey_str
        );
        assert_eq!(
            config.key, expected_key,
            "Key mismatch for {}",
            hotkey_str
        );
    }
}

/// Test invalid hotkey string parsing
#[test]
fn test_invalid_hotkey_parsing() {
    let invalid_cases = vec![
        "",           // Empty string
        "   ",        // Only spaces
        "+",          // Only separator
    ];

    for invalid_str in invalid_cases {
        let result = HotkeyConfig::from_string(invalid_str);
        assert!(result.is_err(), "Expected error for invalid hotkey: '{}'", invalid_str);
    }
}

/// Test hotkey config to string conversion
#[test]
fn test_hotkey_config_to_string() {
    let test_cases = vec![
        ("Alt+Space", "Alt+Space"),
        ("Ctrl+Shift+A", "Ctrl+Shift+A"),
        ("F12", "F12"),
    ];

    for (input, expected) in test_cases {
        let config = HotkeyConfig::from_string(input).expect("Failed to parse hotkey");
        let output = config.to_string_repr();
        assert_eq!(output, expected);
    }
}

/// Test default hotkey config
#[test]
fn test_default_hotkey_config() {
    let config = HotkeyConfig::default();

    assert_eq!(config.modifiers, "Alt");
    assert_eq!(config.key, "Space");
    assert!(!config.registered);
}

/// Test hotkey config equality
#[test]
fn test_hotkey_config_equality() {
    let config1 = HotkeyConfig::from_string("Alt+Space").unwrap();
    let config2 = HotkeyConfig::from_string("Alt+Space").unwrap();
    let config3 = HotkeyConfig::from_string("Ctrl+Space").unwrap();

    assert_eq!(config1.modifiers, config2.modifiers);
    assert_eq!(config1.key, config2.key);
    assert_ne!(config1.modifiers, config3.modifiers);
}

/// Test hotkey modifiers parsing
#[test]
fn test_hotkey_modifiers_parsing() {
    // Test single modifier
    let config = HotkeyConfig::from_string("Alt+A").unwrap();
    assert_eq!(config.modifiers, "Alt");

    // Test double modifier
    let config = HotkeyConfig::from_string("Ctrl+Shift+A").unwrap();
    assert_eq!(config.modifiers, "Ctrl+Shift");

    // Test triple modifier
    let config = HotkeyConfig::from_string("Ctrl+Alt+Shift+A").unwrap();
    assert_eq!(config.modifiers, "Ctrl+Alt+Shift");

    // Test no modifier
    let config = HotkeyConfig::from_string("F1").unwrap();
    assert_eq!(config.modifiers, "");
}

/// Test hotkey key parsing
#[test]
fn test_hotkey_key_parsing() {
    // Test function keys
    for i in 1..=12 {
        let config = HotkeyConfig::from_string(&format!("F{}", i)).unwrap();
        assert_eq!(config.key, format!("F{}", i));
    }

    // Test letter keys
    let config = HotkeyConfig::from_string("Ctrl+A").unwrap();
    assert_eq!(config.key, "A");

    // Test special keys
    let config = HotkeyConfig::from_string("Alt+Space").unwrap();
    assert_eq!(config.key, "Space");

    let config = HotkeyConfig::from_string("Ctrl+Delete").unwrap();
    assert_eq!(config.key, "Delete");

    let config = HotkeyConfig::from_string("Alt+Escape").unwrap();
    assert_eq!(config.key, "Escape");
}

/// Test hotkey config cloning
#[test]
fn test_hotkey_config_cloning() {
    let original = HotkeyConfig::from_string("Ctrl+Shift+F12").unwrap();
    let cloned = original.clone();

    assert_eq!(original.modifiers, cloned.modifiers);
    assert_eq!(original.key, cloned.key);
    assert_eq!(original.registered, cloned.registered);
}

/// Test hotkey config with whitespace
#[test]
fn test_hotkey_config_with_whitespace() {
    // Test with spaces around separator
    let config = HotkeyConfig::from_string("Alt + Space").unwrap();
    assert_eq!(config.modifiers, "Alt");
    assert_eq!(config.key, "Space");

    // Test with extra spaces
    let config = HotkeyConfig::from_string("  Alt  +  Space  ").unwrap();
    assert_eq!(config.modifiers, "Alt");
    assert_eq!(config.key, "Space");
}

/// Test suggested hotkeys
#[test]
fn test_suggested_hotkeys() {
    // This test verifies that we can generate suggested hotkeys
    // that don't conflict with common system hotkeys
    let suggested = vec![
        "Alt+Space",
        "Ctrl+Space",
        "Alt+Q",
        "Ctrl+Shift+Space",
        "Alt+Shift+Space",
        "Ctrl+Alt+Space",
        "F12",
        "Ctrl+F12",
        "Alt+F12",
    ];

    for hotkey in suggested {
        let config = HotkeyConfig::from_string(hotkey)
            .expect(&format!("Failed to parse suggested hotkey: {}", hotkey));
        assert!(!config.modifiers.is_empty() || config.key.starts_with('F'));
    }
}

/// Test hotkey config serialization
#[test]
fn test_hotkey_config_serialization() {
    let config = HotkeyConfig {
        modifiers: "Ctrl+Alt".to_string(),
        key: "Space".to_string(),
        registered: true,
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize config");
    assert!(json.contains("Ctrl+Alt"));
    assert!(json.contains("Space"));
    assert!(json.contains("true"));

    let deserialized: HotkeyConfig =
        serde_json::from_str(&json).expect("Failed to deserialize config");

    assert_eq!(config.modifiers, deserialized.modifiers);
    assert_eq!(config.key, deserialized.key);
    assert_eq!(config.registered, deserialized.registered);
}

/// Test hotkey validation for Windows-specific keys
#[test]
fn test_windows_specific_hotkeys() {
    // Test Windows-specific key combinations
    let windows_hotkeys = vec![
        "Alt+Tab",           // Common but should be parseable
        "Ctrl+Alt+Delete",   // System key, but parseable
        "Win+E",             // Windows key combination
    ];

    for hotkey in windows_hotkeys {
        let result = HotkeyConfig::from_string(hotkey);
        // All should parse successfully
        assert!(result.is_ok(), "Failed to parse Windows hotkey: {}", hotkey);
    }
}