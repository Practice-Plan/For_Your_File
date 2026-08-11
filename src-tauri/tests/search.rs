//! Search integration tests
//!
//! Tests real-time search, FTS5 queries, result ranking,
//! and search performance.

mod common;

use common::*;
use rusqlite::params;

/// Test basic search functionality
#[test]
fn test_basic_search() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create test entries
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
         VALUES ('vscode.lnk', 'C:\\VSCode\\Code.exe', 'editor,ide', 'Code editor', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 1");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
         VALUES ('chrome.lnk', 'C:\\Chrome\\chrome.exe', 'browser', 'Web browser', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 2");

    // Test search by path
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE target_path LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%VSCode%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "vscode.lnk");

    // Test search by tags
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE tags LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%browser%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "chrome.lnk");
}

/// Test search result ranking by frequency
#[test]
fn test_search_ranking_by_frequency() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entries with different frequencies
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, frequency, created_at, updated_at)
         VALUES ('low_freq.lnk', 'C:\\low.exe', 1, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 1");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, frequency, created_at, updated_at)
         VALUES ('high_freq.lnk', 'C:\\high.exe', 100, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 2");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, frequency, created_at, updated_at)
         VALUES ('medium_freq.lnk', 'C:\\medium.exe', 50, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 3");

    // Query ordered by frequency
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries ORDER BY frequency DESC")
        .expect("Failed to prepare query")
        .query_map([], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(
        results,
        vec!["high_freq.lnk", "medium_freq.lnk", "low_freq.lnk"]
    );
}

/// Test search with multiple criteria
#[test]
fn test_multi_criteria_search() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entries
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, frequency, created_at, updated_at)
         VALUES ('vscode.lnk', 'C:\\VSCode\\Code.exe', 'editor,ide', 'Main editor', 10, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 1");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, frequency, created_at, updated_at)
         VALUES ('chrome.lnk', 'C:\\Chrome\\chrome.exe', 'browser', 'Web browser', 5, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 2");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, frequency, created_at, updated_at)
         VALUES ('notepad.lnk', 'C:\\notepad.exe', 'editor,text', 'Text editor', 20, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 3");

    // Search for editors, ordered by frequency
    let results: Vec<String> = conn
        .prepare(
            "SELECT lnk_path FROM entries
             WHERE tags LIKE ?1 OR notes LIKE ?1
             ORDER BY frequency DESC",
        )
        .expect("Failed to prepare query")
        .query_map(params!["%editor%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    // Should return both editors, with notepad first due to higher frequency
    assert_eq!(results, vec!["notepad.lnk", "vscode.lnk"]);
}

/// Test case-insensitive search
#[test]
fn test_case_insensitive_search() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entry with mixed case
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('VSCode.lnk', 'C:\\VSCode\\Code.exe', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    // Test different case queries
    let test_cases = vec!["%vscode%", "%VSCODE%", "%VsCoDe%"];

    for pattern in test_cases {
        let results: Vec<String> = conn
            .prepare("SELECT lnk_path FROM entries WHERE target_path LIKE ?1")
            .expect("Failed to prepare query")
            .query_map(params![pattern], |row| row.get(0))
            .expect("Failed to query entries")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect results");

        assert_eq!(results.len(), 1, "Expected result for pattern: {}", pattern);
    }
}

/// Test search with empty results
#[test]
fn test_search_empty_results() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('test.lnk', 'C:\\test.exe', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    // Search for non-existent entry
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE target_path LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%nonexistent%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(results.len(), 0);
}

/// Test search performance with large dataset
#[test]
fn test_search_performance() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create a large number of entries
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                format!("app{}.lnk", i),
                format!("C:\\app{}.exe", i),
                now,
                now
            ],
        ).expect(&format!("Failed to insert entry {}", i));
    }

    // Measure search time
    let start = std::time::Instant::now();

    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE target_path LIKE ?1 LIMIT 10")
        .expect("Failed to prepare query")
        .query_map(params!["%app5%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    let duration = start.elapsed();

    // Search should complete quickly (less than 100ms)
    assert!(duration.as_millis() < 100, "Search took too long: {:?}", duration);

    // Should find results (app5, app50-59, app500-599)
    assert!(results.len() > 0);
}

/// Test search with special characters
#[test]
fn test_search_special_characters() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entries with special characters in paths
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('test.lnk', 'C:\\Program Files (x86)\\App\\app.exe', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 1");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('test2.lnk', 'C:\\Users\\User Name\\App.exe', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 2");

    // Search for path with parentheses
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE target_path LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%(x86)%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "test.lnk");

    // Search for path with spaces
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE target_path LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%User Name%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "test2.lnk");
}

/// Test search ordering by last opened
#[test]
fn test_search_ordering_by_last_opened() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let base_time = chrono::Utc::now().timestamp();

    // Create entries with different last_opened times
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, last_opened, created_at, updated_at)
         VALUES ('old.lnk', 'C:\\old.exe', ?1, ?2, ?2)",
        params![base_time - 3600, base_time], // 1 hour ago
    ).expect("Failed to insert entry 1");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, last_opened, created_at, updated_at)
         VALUES ('recent.lnk', 'C:\\recent.exe', ?1, ?2, ?2)",
        params![base_time - 60, base_time], // 1 minute ago
    ).expect("Failed to insert entry 2");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, last_opened, created_at, updated_at)
         VALUES ('never.lnk', 'C:\\never.exe', NULL, ?1, ?1)",
        params![base_time],
    ).expect("Failed to insert entry 3");

    // Query ordered by last_opened (most recent first, NULL last)
    let results: Vec<String> = conn
        .prepare(
            "SELECT lnk_path FROM entries
             ORDER BY COALESCE(last_opened, 0) DESC",
        )
        .expect("Failed to prepare query")
        .query_map([], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    // Most recent should come first, never opened should be last
    assert_eq!(results, vec!["recent.lnk", "old.lnk", "never.lnk"]);
}

/// Test partial match search
#[test]
fn test_partial_match_search() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entries
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('vscode.lnk', 'C:\\VSCode\\Code.exe', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('visual_studio.lnk', 'C:\\Visual Studio\\devenv.exe', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    // Search for partial match
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE target_path LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%Visual%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    // Should find only Visual Studio, not VSCode
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "visual_studio.lnk");
}