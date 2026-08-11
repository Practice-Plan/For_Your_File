//! Comprehensive tests for FTS5 search functionality
//!
//! Tests cover basic search, multi-keyword search, Pinyin search,
//! ranking, and performance benchmarks.

use std::sync::Arc;
use std::time::Instant;

use rusqlite::Connection;

use crate::db::{Database, DbConnection};
use crate::models::Entry;
use crate::search::*;

/// Helper to create a test database with sample entries
fn setup_test_db() -> (Database, DbConnection) {
    let db = Database::new_in_memory().expect("Failed to create in-memory database");
    let conn = db.connection().expect("Failed to get connection");

    // Insert test entries
    let entries = vec![
        ("Visual Studio", "C:\\Program Files\\Microsoft Visual Studio\\2022\\Common7\\IDE\\devenv.exe", "ide,development", "Microsoft IDE"),
        ("VS Code", "C:\\Users\\test\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe", "editor,development", "Lightweight code editor"),
        ("文件管理器", "C:\\Windows\\explorer.exe", "system,file", "Windows file manager"),
        ("Chrome", "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe", "browser,web", "Web browser"),
        ("Notepad", "C:\\Windows\\System32\\notepad.exe", "editor,text", "Text editor"),
        ("Word", "C:\\Program Files\\Microsoft Office\\root\\Office16\\WINWORD.EXE", "office,document", "Word processor"),
        ("Excel", "C:\\Program Files\\Microsoft Office\\root\\Office16\\EXCEL.EXE", "office,spreadsheet", "Spreadsheet application"),
        ("PowerPoint", "C:\\Program Files\\Microsoft Office\\root\\Office16\\POWERPNT.EXE", "office,presentation", "Presentation software"),
        ("Terminal", "C:\\Windows\\System32\\cmd.exe", "system,terminal", "Command prompt"),
        ("PowerShell", "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", "system,terminal", "PowerShell terminal"),
    ];

    let now = chrono::Utc::now().timestamp();
    for (name, target, tags, notes) in entries {
        let entry = Entry::new(
            format!("C:\\Shortcuts\\{}.lnk", name.replace(' ', "_")),
            target.to_string(),
        )
        .with_tags(tags.to_string())
        .with_notes(notes.to_string());

        crate::db::insert_entry(&conn, &entry).expect("Failed to insert entry");
    }

    (db, conn)
}

/// Helper to create a large test database for performance testing
fn setup_large_test_db(count: usize) -> (Database, DbConnection) {
    let db = Database::new_in_memory().expect("Failed to create in-memory database");
    let conn = db.connection().expect("Failed to get connection");

    let now = chrono::Utc::now().timestamp();
    for i in 0..count {
        let entry = Entry::new(
            format!("C:\\Shortcuts\\App_{}.lnk", i),
            format!("C:\\Program Files\\App{}\\app.exe", i),
        )
        .with_tags(format!("tag{}", i % 10))
        .with_notes(format!("Application number {}", i));

        let mut entry = entry;
        entry.frequency = (i % 100) as i32;
        entry.last_opened = if i % 5 == 0 { Some(now - (i as i64) * 100) } else { None };

        crate::db::insert_entry(&conn, &entry).expect("Failed to insert entry");
    }

    (db, conn)
}

#[test]
fn test_basic_keyword_search() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Search for "visual"
    let results = engine.search("visual").expect("Search failed");
    assert!(!results.is_empty(), "Should find results for 'visual'");
    assert!(results.iter().any(|r| r.entry.target_path.contains("Visual Studio")),
        "Should find Visual Studio entry");

    // Search for "chrome"
    let results = engine.search("chrome").expect("Search failed");
    assert!(!results.is_empty(), "Should find results for 'chrome'");
    assert!(results.iter().any(|r| r.entry.target_path.contains("Chrome")),
        "Should find Chrome entry");
}

#[test]
fn test_multi_keyword_search() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Search for "visual studio" (AND logic by default)
    let results = engine.search("visual studio").expect("Search failed");
    assert!(!results.is_empty(), "Should find results for 'visual studio'");

    // Search for "office" should return multiple entries
    let results = engine.search("office").expect("Search failed");
    assert!(results.len() >= 3, "Should find at least 3 Office entries");
}

#[test]
fn test_or_search() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Search for "word OR excel"
    let results = engine.search("word OR excel").expect("Search failed");
    assert!(results.len() >= 2, "Should find both Word and Excel entries");
}

#[test]
fn test_search_by_tags() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Search for "terminal" tag
    let results = engine.search("terminal").expect("Search failed");
    assert!(results.len() >= 2, "Should find terminal entries (cmd and powershell)");
}

#[test]
fn test_search_by_notes() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Search for "browser"
    let results = engine.search("browser").expect("Search failed");
    assert!(!results.is_empty(), "Should find browser entries");
}

#[test]
fn test_search_with_paging() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Get first page
    let page1 = engine.search_with_paging("office", 0, 2).expect("Search failed");
    assert_eq!(page1.results.len(), 2, "Should return 2 results");
    assert!(page1.total_count >= 3, "Should have at least 3 total matches");
    assert_eq!(page1.offset, 0);
    assert_eq!(page1.limit, 2);

    // Get second page
    let page2 = engine.search_with_paging("office", 2, 2).expect("Search failed");
    assert!(page2.results.len() <= 2, "Should return at most 2 results");
    assert_eq!(page2.offset, 2);
}

#[test]
fn test_search_ranking_by_frequency() {
    let (_db, conn) = setup_test_db();
    let mut engine = SearchEngine::new(conn.clone());

    // Set frequency sort criteria
    engine.set_sort_criteria(SortCriteria::Frequency);

    let results = engine.search("*").expect("Search failed");
    // Results should be sorted by frequency
    for i in 1..results.len().min(10) {
        assert!(results[i - 1].entry.frequency >= results[i].entry.frequency,
            "Results should be sorted by frequency descending");
    }
}

#[test]
fn test_search_ranking_relevance() {
    let (_db, conn) = setup_test_db();
    let mut engine = SearchEngine::new(conn);
    engine.set_sort_criteria(SortCriteria::Relevance);

    let results = engine.search("visual studio").expect("Search failed");
    // Results should be sorted by relevance score
    for i in 1..results.len() {
        assert!(results[i - 1].score >= results[i].score,
            "Results should be sorted by score descending");
    }
}

#[test]
fn test_custom_ranking_weights() {
    let (_db, conn) = setup_test_db();
    let mut engine = SearchEngine::new(conn);
    engine.set_ranking_weights(0.5, 0.3, 0.2);

    let results = engine.search("office").expect("Search failed");
    assert!(!results.is_empty(), "Should have results");
}

#[test]
fn test_highlight() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Test highlight function
    let highlighted = engine.highlight("Visual Studio", "visual", "<mark>", "</mark>");
    // Note: This might fail if there's no match in FTS, but should not panic
    assert!(highlighted.is_ok());
}

#[test]
fn test_flexible_search_fallback() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    // Search for something that might not be in FTS but in LIKE
    let results = engine.search_flexible("explorer").expect("Search failed");
    assert!(!results.is_empty(), "Should find explorer through LIKE fallback");
}

#[test]
fn test_no_results_search() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    let results = engine.search("nonexistentrandomstring12345").expect("Search failed");
    assert!(results.is_empty(), "Should return empty for non-existent query");
}

#[test]
fn test_search_performance() {
    // Create database with 1000 entries for performance test
    let (_db, conn) = setup_large_test_db(1000);
    let engine = SearchEngine::new(conn);

    // Measure search time
    let start = Instant::now();
    let results = engine.search("App_500").expect("Search failed");
    let duration = start.elapsed();

    println!("Search time for 1K entries: {:?}", duration);

    // Should complete within 50ms
    assert!(duration.as_millis() < 50, 
        "Search should complete within 50ms, took {:?}", duration);
}

#[test]
fn test_search_performance_paging() {
    let (_db, conn) = setup_large_test_db(1000);
    let engine = SearchEngine::new(conn);

    let start = Instant::now();
    let results = engine.search_with_paging("App", 0, 100).expect("Search failed");
    let duration = start.elapsed();

    println!("Paged search time for 1K entries: {:?}", duration);

    // Should complete within 50ms
    assert!(duration.as_millis() < 50,
        "Paged search should complete within 50ms, took {:?}", duration);

    assert_eq!(results.results.len(), 100);
}

#[test]
fn test_pinyin_tokenizer() {
    let tokenizer = PinyinTokenizer::new();

    // Test full pinyin conversion
    let (full, initials) = tokenizer.to_pinyin("文件管理");
    assert_eq!(initials, "wjgl");

    // Test ASCII handling
    let (full, _) = tokenizer.to_pinyin("VS Code");
    assert!(full.contains("vs"));
    assert!(full.contains("code"));
}

#[test]
fn test_pinyin_search_expansion() {
    let tokenizer = PinyinTokenizer::new();

    // Test Chinese query expansion
    let expanded = tokenizer.search_with_pinyin("文件");
    assert!(expanded.contains("wenjian") || expanded.contains("wj"),
        "Expansion should contain pinyin or initials");

    // Test ASCII query (should not expand)
    let expanded = tokenizer.search_with_pinyin("vscode");
    assert!(expanded.contains("vscode"));
}

#[test]
fn test_pinyin_caching() {
    let tokenizer = PinyinTokenizer::new();

    // First call - should cache
    tokenizer.to_pinyin("文件");
    let cache_size = tokenizer.cache_size();
    assert!(cache_size > 0, "Cache should contain entries");

    // Second call - should use cache
    tokenizer.to_pinyin("文件");
    assert_eq!(tokenizer.cache_size(), cache_size, "Cache size should not change");

    // Clear cache
    tokenizer.clear_cache();
    assert_eq!(tokenizer.cache_size(), 0, "Cache should be empty after clear");
}

#[test]
fn test_parse_search_query() {
    // Test simple query
    let parsed = parse_search_query("visual studio");
    assert_eq!(parsed.keywords, vec!["visual", "studio"]);
    assert!(parsed.use_and);

    // Test OR query
    let parsed = parse_search_query("word OR excel");
    assert_eq!(parsed.keywords, vec!["word", "excel"]);
    assert!(!parsed.use_and);

    // Test quoted query
    let parsed = parse_search_query("\"visual studio\"");
    assert!(parsed.phrases.contains(&"visual studio".to_string()));
}

#[test]
fn test_build_fts_query() {
    // Test AND query
    let query = build_fts_query("visual studio");
    assert_eq!(query, "visual* AND studio*");

    // Test OR query
    let query = build_fts_query("word OR excel");
    assert_eq!(query, "word* OR excel*");

    // Test empty query
    let query = build_fts_query("");
    assert_eq!(query, "*");
}

#[test]
fn test_escape_fts_special_chars() {
    assert_eq!(escape_fts_special_chars("test*"), "test\\*");
    assert_eq!(escape_fts_special_chars("(test)"), "\\(test\\)");
    assert_eq!(escape_fts_special_chars("test"), "test");
}

#[test]
fn test_ranking_engine() {
    let engine = RankingEngine::new();

    // Create test results
    let results = vec![
        SearchResult {
            entry: Entry::new("a.lnk".to_string(), "a.exe".to_string()),
            score: 1.0,
            snippet: None,
        },
        SearchResult {
            entry: Entry::new("b.lnk".to_string(), "b.exe".to_string()),
            score: 0.5,
            snippet: None,
        },
    ];

    let ranked = engine.rank_results(results);
    assert_eq!(ranked.len(), 2);
}

#[test]
fn test_sort_criteria() {
    let (_db, conn) = setup_test_db();
    let mut engine = SearchEngine::new(conn);

    // Test frequency sort
    engine.set_sort_criteria(SortCriteria::Frequency);
    assert_eq!(engine.ranking_engine.get_sort_criteria(), SortCriteria::Frequency);

    // Test recency sort
    engine.set_sort_criteria(SortCriteria::Recency);
    assert_eq!(engine.ranking_engine.get_sort_criteria(), SortCriteria::Recency);
}

#[test]
fn test_get_entries_by_frequency() {
    let (_db, conn) = setup_test_db();
    let engine = SearchEngine::new(conn);

    let entries = engine.get_entries_by_frequency(10).expect("Failed to get entries");
    assert!(entries.len() <= 10, "Should return at most 10 entries");
}