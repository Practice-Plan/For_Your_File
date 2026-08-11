//! Search utility functions
//!
//! Provides helper functions for parsing and building search queries.

use std::collections::HashSet;

/// Represents a parsed search query
#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    /// Keywords to search for
    pub keywords: Vec<String>,
    /// Whether to use AND logic (true) or OR logic (false)
    pub use_and: bool,
    /// Quoted phrases that should be searched as-is
    pub phrases: Vec<String>,
    /// Raw query string
    pub raw: String,
}

/// Parse a search query into keywords and operators
///
/// Supports:
/// - Multiple keywords (space-separated)
/// - Quoted phrases
/// - AND/OR operators (case-insensitive)
///
/// # Arguments
/// * `query` - The raw search query string
///
/// # Returns
/// Parsed query structure
pub fn parse_search_query(query: &str) -> ParsedQuery {
    let mut parsed = ParsedQuery {
        raw: query.to_string(),
        ..Default::default()
    };

    let trimmed = query.trim();
    if trimmed.is_empty() {
        return parsed;
    }

    // Check for explicit OR operator
    let upper = trimmed.to_uppercase();
    parsed.use_and = !upper.contains(" OR ");

    // Split by spaces, respecting quotes
    let mut in_quote = false;
    let mut current = String::new();
    let mut tokens: Vec<String> = Vec::new();

    for ch in trimmed.chars() {
        match ch {
            '"' => {
                if in_quote {
                    // End of quoted phrase
                    if !current.is_empty() {
                        parsed.phrases.push(current.clone());
                        tokens.push(current.clone());
                        current.clear();
                    }
                    in_quote = false;
                } else {
                    // Start of quoted phrase
                    in_quote = true;
                }
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Don't forget the last token
    if !current.is_empty() {
        tokens.push(current);
    }

    // Filter out operators and collect keywords
    for token in tokens {
        let upper = token.to_uppercase();
        if upper != "AND" && upper != "OR" {
            parsed.keywords.push(token);
        }
    }

    parsed
}

/// Build an FTS5 MATCH query string
///
/// # Arguments
/// * `query` - The raw search query
///
/// # Returns
/// FTS5 compatible MATCH query string
pub fn build_fts_query(query: &str) -> String {
    let parsed = parse_search_query(query);

    if parsed.keywords.is_empty() {
        return "*".to_string(); // Match all
    }

    // Build query with proper FTS5 syntax
    let escaped_keywords: Vec<String> = parsed
        .keywords
        .iter()
        .map(|k| escape_fts_special_chars(k))
        .collect();

    // Add prefix matching for each keyword
    let prefixed_keywords: Vec<String> = escaped_keywords
        .iter()
        .map(|k| format!("{}*", k))
        .collect();

    if parsed.use_and {
        // AND logic: all keywords must match
        prefixed_keywords.join(" AND ")
    } else {
        // OR logic: any keyword can match
        prefixed_keywords.join(" OR ")
    }
}

/// Escape special characters for FTS5 queries
///
/// FTS5 has special characters: * ^ " ' ( ) { } : + - ~
///
/// # Arguments
/// * `query` - The query string to escape
///
/// # Returns
/// Escaped query string safe for FTS5
pub fn escape_fts_special_chars(query: &str) -> String {
    let special_chars: HashSet<char> = ['*', '^', '"', '\'', '(', ')', '{', '}', ':', '+', '-', '~'].iter().cloned().collect();

    query
        .chars()
        .map(|c| {
            if special_chars.contains(&c) {
                format!("\\{}", c)
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// Sanitize query for LIKE pattern matching
///
/// # Arguments
/// * `query` - The query string to sanitize
///
/// # Returns
/// Sanitized query string safe for LIKE
pub fn sanitize_like_pattern(query: &str) -> String {
    query
        .chars()
        .map(|c| match c {
            '%' => "\\%".to_string(),
            '_' => "\\_".to_string(),
            '\\' => "\\\\".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let parsed = parse_search_query("visual studio");
        assert_eq!(parsed.keywords, vec!["visual", "studio"]);
        assert!(parsed.use_and);
    }

    #[test]
    fn test_parse_or_query() {
        let parsed = parse_search_query("vs code OR vsc");
        assert_eq!(parsed.keywords, vec!["vs", "code", "vsc"]);
        assert!(!parsed.use_and);
    }

    #[test]
    fn test_parse_quoted_phrase() {
        let parsed = parse_search_query("\"visual studio\"");
        assert!(parsed.phrases.contains(&"visual studio".to_string()));
    }

    #[test]
    fn test_parse_empty_query() {
        let parsed = parse_search_query("");
        assert!(parsed.keywords.is_empty());
    }

    #[test]
    fn test_build_fts_query_and() {
        let fts_query = build_fts_query("visual studio");
        assert_eq!(fts_query, "visual* AND studio*");
    }

    #[test]
    fn test_build_fts_query_or() {
        let fts_query = build_fts_query("vs code OR vsc");
        assert_eq!(fts_query, "vs* OR code* OR vsc*");
    }

    #[test]
    fn test_build_fts_query_empty() {
        let fts_query = build_fts_query("");
        assert_eq!(fts_query, "*");
    }

    #[test]
    fn test_escape_fts_special_chars() {
        assert_eq!(escape_fts_special_chars("test*"), "test\\*");
        assert_eq!(escape_fts_special_chars("(test)"), "\\(test\\)");
        assert_eq!(escape_fts_special_chars("test"), "test");
    }

    #[test]
    fn test_sanitize_like_pattern() {
        assert_eq!(sanitize_like_pattern("test%"), "test\\%");
        assert_eq!(sanitize_like_pattern("test_"), "test\\_");
        assert_eq!(sanitize_like_pattern("test\\file"), "test\\\\file");
    }

    #[test]
    fn test_complex_query() {
        let parsed = parse_search_query("\"Visual Studio\" OR \"VS Code\"");
        assert_eq!(parsed.keywords, vec!["Visual", "Studio", "VS", "Code"]);
        assert!(!parsed.use_and);
    }
}