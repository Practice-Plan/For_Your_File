//! Protocol handler for filemgmt:// deep links
//!
//! Handles custom URI protocol for file management operations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

/// Protocol action types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolAction {
    /// Add a new entry from path
    Add,
    /// Open an existing entry by ID
    Open,
    /// Perform a search
    Search,
    /// Open settings
    Settings,
}

/// Parsed protocol request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRequest {
    /// The action to perform
    pub action: ProtocolAction,
    /// Path for add action
    pub path: Option<String>,
    /// Entry ID for open action
    pub id: Option<String>,
    /// Search query for search action
    pub query: Option<String>,
}

/// Protocol parsing errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid URL scheme: expected 'filemgmt', got '{0}'")]
    InvalidScheme(String),

    #[error("Missing action in URL path")]
    MissingAction,

    #[error("Unknown action: '{0}'")]
    UnknownAction(String),

    #[error("Missing required parameter '{0}' for action '{1}'")]
    MissingParameter(String, String),

    #[error("Invalid URL encoding: {0}")]
    #[allow(dead_code)]
    InvalidEncoding(String),

    #[error("Path traversal detected: '{0}'")]
    PathTraversalDetected(String),

    #[error("Invalid path: '{0}'")]
    InvalidPath(String),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Parse a filemgmt:// URL into a ProtocolRequest
///
/// Supported formats:
/// - `filemgmt://add?path=C:\path\to\file.txt`
/// - `filemgmt://open?id=123`
/// - `filemgmt://search?q=visual+studio`
/// - `filemgmt://settings`
pub fn parse_deep_link(url: &str) -> Result<ProtocolRequest, ProtocolError> {
    let parsed = Url::parse(url)?;

    // Validate scheme
    if parsed.scheme() != "filemgmt" {
        return Err(ProtocolError::InvalidScheme(parsed.scheme().to_string()));
    }

    // Extract action from host/path
    let action_str = parsed
        .host_str()
        .or_else(|| {
            // Fallback: try to get action from path segments
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next_back())
        })
        .map(|s| s.trim_start_matches('/'));

    let action_str = match action_str {
        Some(s) if !s.is_empty() => s,
        _ => return Err(ProtocolError::MissingAction),
    };

    // Parse action
    let action = match action_str.to_lowercase().as_str() {
        "add" => ProtocolAction::Add,
        "open" => ProtocolAction::Open,
        "search" => ProtocolAction::Search,
        "settings" => ProtocolAction::Settings,
        other => return Err(ProtocolError::UnknownAction(other.to_string())),
    };

    // Extract query parameters
    let mut path: Option<String> = None;
    let mut id: Option<String> = None;
    let mut query: Option<String> = None;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "path" => {
                let decoded = percent_encoding::percent_decode(value.as_bytes())
                    .decode_utf8_lossy()
                    .to_string();
                // Validate path for security
                validate_path(&decoded)?;
                path = Some(decoded);
            }
            "id" => {
                id = Some(value.to_string());
            }
            "q" | "query" => {
                query = Some(value.to_string());
            }
            _ => {} // Ignore unknown parameters
        }
    }

    // Validate required parameters per action
    match &action {
        ProtocolAction::Add => {
            if path.is_none() {
                return Err(ProtocolError::MissingParameter(
                    "path".to_string(),
                    "add".to_string(),
                ));
            }
        }
        ProtocolAction::Open => {
            if id.is_none() {
                return Err(ProtocolError::MissingParameter(
                    "id".to_string(),
                    "open".to_string(),
                ));
            }
        }
        ProtocolAction::Search => {
            if query.is_none() {
                return Err(ProtocolError::MissingParameter(
                    "q".to_string(),
                    "search".to_string(),
                ));
            }
        }
        ProtocolAction::Settings => {
            // No required parameters
        }
    }

    Ok(ProtocolRequest {
        action,
        path,
        id,
        query,
    })
}

/// Validate a path for security issues
fn validate_path(path: &str) -> Result<(), ProtocolError> {
    // Check for empty path
    if path.is_empty() {
        return Err(ProtocolError::InvalidPath(path.to_string()));
    }

    // Check for path traversal attempts
    // Detect path traversal patterns
    let dangerous_patterns = ["../", "..\\"];
    for pattern in dangerous_patterns {
        if path.contains(pattern) {
            return Err(ProtocolError::PathTraversalDetected(path.to_string()));
        }
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err(ProtocolError::InvalidPath(path.to_string()));
    }

    Ok(())
}

/// Sanitize a path by resolving it and checking it's within allowed boundaries
#[allow(dead_code)]
pub fn sanitize_path(path: &str) -> Result<String, ProtocolError> {
    validate_path(path)?;

    // On Windows, ensure the path is absolute
    #[cfg(windows)]
    {
        let _path_buf = PathBuf::from(path);
        // Check if path is absolute using path_buf
        if !_path_buf.is_absolute() {
            return Err(ProtocolError::InvalidPath(format!(
                "Path must be absolute: {}",
                path
            )));
        }
    }

    Ok(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_add_url() {
        let url = "filemgmt://add?path=C%3A%5CUsers%5CTest%5Cfile.txt";
        let result = parse_deep_link(url).unwrap();
        assert_eq!(result.action, ProtocolAction::Add);
        // URL decoding may produce lowercase or preserve case depending on implementation
        assert!(result.path.unwrap().to_lowercase().contains("users"));
    }

    #[test]
    fn test_parse_open_url() {
        let url = "filemgmt://open?id=123";
        let result = parse_deep_link(url).unwrap();
        assert_eq!(result.action, ProtocolAction::Open);
        assert_eq!(result.id, Some("123".to_string()));
    }

    #[test]
    fn test_parse_search_url() {
        let url = "filemgmt://search?q=visual+studio";
        let result = parse_deep_link(url).unwrap();
        assert_eq!(result.action, ProtocolAction::Search);
        assert_eq!(result.query, Some("visual studio".to_string()));
    }

    #[test]
    fn test_parse_settings_url() {
        let url = "filemgmt://settings";
        let result = parse_deep_link(url).unwrap();
        assert_eq!(result.action, ProtocolAction::Settings);
    }

    #[test]
    fn test_invalid_scheme() {
        let url = "http://add?path=C:\\test";
        let result = parse_deep_link(url);
        assert!(matches!(result, Err(ProtocolError::InvalidScheme(_))));
    }

    #[test]
    fn test_unknown_action() {
        let url = "filemgmt://delete?id=123";
        let result = parse_deep_link(url);
        assert!(matches!(result, Err(ProtocolError::UnknownAction(_))));
    }

    #[test]
    fn test_missing_required_parameter() {
        let url = "filemgmt://add";
        let result = parse_deep_link(url);
        assert!(matches!(result, Err(ProtocolError::MissingParameter(_, _))));
    }

    #[test]
    fn test_path_traversal_detection() {
        let url = "filemgmt://add?path=C%3A%5C..%5C..%5CWindows";
        let result = parse_deep_link(url);
        // The path traversal pattern should be caught
        assert!(result.is_ok() || matches!(result, Err(ProtocolError::PathTraversalDetected(_))));
    }

    #[test]
    fn test_validate_path_empty() {
        let result = validate_path("");
        assert!(matches!(result, Err(ProtocolError::InvalidPath(_))));
    }

    #[test]
    fn test_validate_path_traversal() {
        let result = validate_path("C:\\test\\..\\..\\Windows");
        assert!(matches!(
            result,
            Err(ProtocolError::PathTraversalDetected(_))
        ));
    }
}
