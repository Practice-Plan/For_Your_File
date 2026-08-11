//! Command-line argument handling
//!
//! Provides parsing and handling of CLI arguments for the application.

use serde::{Deserialize, Serialize};
use std::ffi::OsString;

/// Parsed CLI arguments
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliArgs {
    /// Add a new entry from path
    pub add: Option<String>,
    /// Open an entry by ID
    pub open: Option<String>,
    /// Start with search query
    pub search: Option<String>,
    /// Start minimized to tray
    pub minimized: bool,
    /// Show version
    pub version: bool,
    /// Help requested
    pub help: bool,
    /// Raw deep link URL (from protocol activation)
    pub deep_link: Option<String>,
    /// Positional arguments (typically file paths)
    pub files: Vec<String>,
}

impl CliArgs {
    /// Parse CLI arguments from the environment
    pub fn parse() -> Self {
        let args: Vec<OsString> = std::env::args_os().collect();
        Self::parse_from(args)
    }

    /// Parse CLI arguments from a vector of OsString
    pub fn parse_from<I>(args: I) -> Self
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut result = CliArgs::default();
        let mut iter = args.into_iter().skip(1); // Skip program name

        while let Some(arg) = iter.next() {
            let arg_str = arg.to_string_lossy();

            match arg_str.as_ref() {
                "--add" | "-a" => {
                    if let Some(value) = iter.next() {
                        result.add = Some(value.to_string_lossy().to_string());
                    }
                }
                "--open" | "-o" => {
                    if let Some(value) = iter.next() {
                        result.open = Some(value.to_string_lossy().to_string());
                    }
                }
                "--search" | "-s" => {
                    if let Some(value) = iter.next() {
                        result.search = Some(value.to_string_lossy().to_string());
                    }
                }
                "--minimized" | "-m" => {
                    result.minimized = true;
                }
                "--version" | "-v" | "-V" => {
                    result.version = true;
                }
                "--help" | "-h" => {
                    result.help = true;
                }
                // Handle deep link URL (typically passed by OS)
                arg if arg.starts_with("filemgmt://") => {
                    result.deep_link = Some(arg_str.to_string());
                }
                // Handle positional arguments (files)
                arg if !arg.starts_with('-') => {
                    result.files.push(arg_str.to_string());
                }
                _ => {
                    // Ignore unknown flags
                    log::warn!("Unknown CLI argument: {}", arg_str);
                }
            }
        }

        result
    }

    /// Check if any action is requested
    #[allow(dead_code)]
    pub fn has_action(&self) -> bool {
        self.add.is_some()
            || self.open.is_some()
            || self.search.is_some()
            || self.deep_link.is_some()
            || !self.files.is_empty()
    }

    /// Convert to a help string
    pub fn help_text() -> String {
        format!(
            r#"LNK File Management Center v{}

Usage: lnk-management [OPTIONS] [FILES]

Options:
  -a, --add <PATH>      Add a new entry from the specified path
  -o, --open <ID>       Open an entry by its ID
  -s, --search <QUERY>  Start application with an active search
  -m, --minimized       Start minimized to system tray
  -v, --version         Show version information
  -h, --help            Show this help message

Protocol URLs:
  filemgmt://add?path=<PATH>    Add a new entry
  filemgmt://open?id=<ID>       Open an entry by ID
  filemgmt://search?q=<QUERY>   Perform a search
  filemgmt://settings           Open settings

Examples:
  lnk-management --add "C:\Users\Test\file.lnk"
  lnk-management --open 123
  lnk-management --search "visual studio"
  lnk-management --minimized
"#,
            env!("CARGO_PKG_VERSION")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_parse_add() {
        let args = vec![
            OsString::from("program"),
            OsString::from("--add"),
            OsString::from("C:\\test\\file.lnk"),
        ];
        let parsed = CliArgs::parse_from(args);
        assert_eq!(parsed.add, Some("C:\\test\\file.lnk".to_string()));
    }

    #[test]
    fn test_parse_open() {
        let args = vec![
            OsString::from("program"),
            OsString::from("--open"),
            OsString::from("123"),
        ];
        let parsed = CliArgs::parse_from(args);
        assert_eq!(parsed.open, Some("123".to_string()));
    }

    #[test]
    fn test_parse_search() {
        let args = vec![
            OsString::from("program"),
            OsString::from("--search"),
            OsString::from("visual studio"),
        ];
        let parsed = CliArgs::parse_from(args);
        assert_eq!(parsed.search, Some("visual studio".to_string()));
    }

    #[test]
    fn test_parse_minimized() {
        let args = vec![OsString::from("program"), OsString::from("--minimized")];
        let parsed = CliArgs::parse_from(args);
        assert!(parsed.minimized);
    }

    #[test]
    fn test_parse_version() {
        let args = vec![OsString::from("program"), OsString::from("--version")];
        let parsed = CliArgs::parse_from(args);
        assert!(parsed.version);
    }

    #[test]
    fn test_parse_deep_link() {
        let args = vec![
            OsString::from("program"),
            OsString::from("filemgmt://open?id=123"),
        ];
        let parsed = CliArgs::parse_from(args);
        assert_eq!(
            parsed.deep_link,
            Some("filemgmt://open?id=123".to_string())
        );
    }

    #[test]
    fn test_parse_files() {
        let args = vec![
            OsString::from("program"),
            OsString::from("C:\\file1.lnk"),
            OsString::from("C:\\file2.lnk"),
        ];
        let parsed = CliArgs::parse_from(args);
        assert_eq!(parsed.files.len(), 2);
        assert_eq!(parsed.files[0], "C:\\file1.lnk");
        assert_eq!(parsed.files[1], "C:\\file2.lnk");
    }

    #[test]
    fn test_parse_short_flags() {
        let args = vec![
            OsString::from("program"),
            OsString::from("-a"),
            OsString::from("test.lnk"),
            OsString::from("-m"),
        ];
        let parsed = CliArgs::parse_from(args);
        assert_eq!(parsed.add, Some("test.lnk".to_string()));
        assert!(parsed.minimized);
    }

    #[test]
    fn test_has_action() {
        let args = vec![
            OsString::from("program"),
            OsString::from("--add"),
            OsString::from("test.lnk"),
        ];
        let parsed = CliArgs::parse_from(args);
        assert!(parsed.has_action());

        let args = vec![OsString::from("program")];
        let parsed = CliArgs::parse_from(args);
        assert!(!parsed.has_action());
    }

    #[test]
    fn test_parse_empty() {
        let args = vec![OsString::from("program")];
        let parsed = CliArgs::parse_from(args);
        assert!(parsed.add.is_none());
        assert!(parsed.open.is_none());
        assert!(parsed.search.is_none());
        assert!(!parsed.minimized);
        assert!(!parsed.version);
        assert!(!parsed.help);
    }
}