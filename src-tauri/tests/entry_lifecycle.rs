//! Entry lifecycle integration tests
//!
//! Tests end-to-end workflows for entry management including
//! creation, search, launch, update, and deletion.

mod common;

use common::*;
use rusqlite::params;
use std::fs;
use tempfile::TempDir;

/// Test complete entry lifecycle: Create → Search → Launch → Update → Delete
#[test]
fn test_entry_lifecycle_workflow() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    // Step 1: Create entries for different file types
    let exe_lnk = fixture.create_exe_lnk("vscode", "Code.exe");
    let doc_lnk = fixture.create_doc_lnk("readme", "README.md");
    let folder_lnk = fixture.create_folder_lnk("projects", "Projects");

    // Insert entry for executable
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, target_type, created_at, updated_at)
         VALUES (?1, ?2, 'exe', ?3, ?4)",
        params![
            exe_lnk.to_string_lossy().to_string(),
            fixture.temp_dir.path().join("Code.exe").to_string_lossy().to_string(),
            now,
            now
        ],
    ).expect("Failed to insert exe entry");
    let exe_id = conn.last_insert_rowid();

    // Insert entry for document
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, target_type, created_at, updated_at)
         VALUES (?1, ?2, 'doc', ?3, ?4)",
        params![
            doc_lnk.to_string_lossy().to_string(),
            fixture.temp_dir.path().join("README.md").to_string_lossy().to_string(),
            now,
            now
        ],
    ).expect("Failed to insert doc entry");
    let doc_id = conn.last_insert_rowid();

    // Insert entry for folder
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, target_type, created_at, updated_at)
         VALUES (?1, ?2, 'folder', ?3, ?4)",
        params![
            folder_lnk.to_string_lossy().to_string(),
            fixture.temp_dir.path().join("Projects").to_string_lossy().to_string(),
            now,
            now
        ],
    ).expect("Failed to insert folder entry");
    let folder_id = conn.last_insert_rowid();

    // Step 2: Search for entries
    let entries: Vec<i64> = conn
        .prepare("SELECT id FROM entries ORDER BY created_at")
        .expect("Failed to prepare query")
        .query_map([], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect entries");

    assert_eq!(entries.len(), 3);
    assert!(entries.contains(&exe_id));
    assert!(entries.contains(&doc_id));
    assert!(entries.contains(&folder_id));

    // Step 3: Simulate launch (increment frequency)
    conn.execute(
        "UPDATE entries SET frequency = frequency + 1, last_opened = ?1 WHERE id = ?2",
        params![chrono::Utc::now().timestamp(), exe_id],
    ).expect("Failed to update frequency");

    let frequency: i32 = conn
        .query_row(
            "SELECT frequency FROM entries WHERE id = ?1",
            params![exe_id],
            |row| row.get(0),
        )
        .expect("Failed to query frequency");

    assert_eq!(frequency, 1);

    // Step 4: Update entry
    conn.execute(
        "UPDATE entries SET tags = ?1, notes = ?2, updated_at = ?3 WHERE id = ?4",
        params!["editor,development", "Main code editor", chrono::Utc::now().timestamp(), exe_id],
    ).expect("Failed to update entry");

    let (tags, notes): (String, String) = conn
        .query_row(
            "SELECT tags, notes FROM entries WHERE id = ?1",
            params![exe_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("Failed to query updated entry");

    assert_eq!(tags, "editor,development");
    assert_eq!(notes, "Main code editor");

    // Step 5: Delete entry
    conn.execute("DELETE FROM entries WHERE id = ?1", params![doc_id])
        .expect("Failed to delete entry");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
        .expect("Failed to count entries");

    assert_eq!(count, 2);

    // Verify data integrity
    let remaining_ids: Vec<i64> = conn
        .prepare("SELECT id FROM entries ORDER BY id")
        .expect("Failed to prepare query")
        .query_map([], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect entries");

    assert!(remaining_ids.contains(&exe_id));
    assert!(remaining_ids.contains(&folder_id));
    assert!(!remaining_ids.contains(&doc_id));
}

/// Test entry creation with various file types
#[test]
fn test_entry_creation_different_types() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    // Create entries for different file types
    let test_cases = vec![
        ("app.exe", "exe"),
        ("document.pdf", "doc"),
        ("folder", "folder"),
        ("website.url", "url"),
    ];

    for (name, file_type) in test_cases {
        let now = chrono::Utc::now().timestamp();
        let lnk_path = fixture.lnk_dir.join(format!("{}.lnk", name));
        let target_path = fixture.temp_dir.path().join(name);

        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, target_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                lnk_path.to_string_lossy().to_string(),
                target_path.to_string_lossy().to_string(),
                file_type,
                now,
                now
            ],
        ).expect(&format!("Failed to insert {} entry", file_type));
    }

    // Verify all entries were created
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
        .expect("Failed to count entries");

    assert_eq!(count, 4);

    // Verify each type exists
    for file_type in &["exe", "doc", "folder", "url"] {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entries WHERE target_type = ?1",
                params![file_type],
                |row| row.get(0),
            )
            .expect(&format!("Failed to count {} entries", file_type));

        assert_eq!(count, 1, "Expected 1 {} entry", file_type);
    }
}

/// Test entry search functionality
#[test]
fn test_entry_search() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create test entries
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
         VALUES ('vscode.lnk', 'C:\\VSCode\\Code.exe', 'editor,ide', 'Main editor', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 1");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
         VALUES ('chrome.lnk', 'C:\\Chrome\\chrome.exe', 'browser,web', 'Web browser', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 2");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
         VALUES ('spotify.lnk', 'C:\\Spotify\\spotify.exe', 'music,media', 'Music player', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry 3");

    // Test search by target path
    let results: Vec<String> = conn
        .prepare("SELECT target_path FROM entries WHERE target_path LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%VSCode%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert!(results[0].contains("VSCode"));

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

    // Test search by notes
    let results: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries WHERE notes LIKE ?1")
        .expect("Failed to prepare query")
        .query_map(params!["%player%"], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "spotify.lnk");
}

/// Test entry update tracking
#[test]
fn test_entry_update_tracking() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let created_at = chrono::Utc::now().timestamp();

    // Create entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('test.lnk', 'C:\\test.exe', ?1, ?2)",
        params![created_at, created_at],
    ).expect("Failed to insert entry");

    let id = conn.last_insert_rowid();

    // Wait a moment before update
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Update entry
    let updated_at = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE entries SET notes = 'updated', updated_at = ?1 WHERE id = ?2",
        params![updated_at, id],
    ).expect("Failed to update entry");

    // Verify timestamps differ
    let (created, updated): (i64, i64) = conn
        .query_row(
            "SELECT created_at, updated_at FROM entries WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("Failed to query timestamps");

    assert!(updated > created, "Updated timestamp should be greater than created");
    assert_eq!(created, created_at);
    assert_eq!(updated, updated_at);
}

/// Test frequency tracking across multiple launches
#[test]
fn test_frequency_tracking() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('test.lnk', 'C:\\test.exe', ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    let id = conn.last_insert_rowid();

    // Simulate multiple launches
    for _ in 0..5 {
        conn.execute(
            "UPDATE entries SET frequency = frequency + 1, last_opened = ?1 WHERE id = ?2",
            params![chrono::Utc::now().timestamp(), id],
        ).expect("Failed to update frequency");
    }

    // Verify frequency
    let frequency: i32 = conn
        .query_row(
            "SELECT frequency FROM entries WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .expect("Failed to query frequency");

    assert_eq!(frequency, 5);
}

/// Test entry ordering by frequency
#[test]
fn test_entry_ordering_by_frequency() {
    let fixture = TestFixture::new();
    let conn = fixture.create_test_database().expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entries with different frequencies
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, frequency, created_at, updated_at)
         VALUES ('low.lnk', 'C:\\low.exe', 1, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, frequency, created_at, updated_at)
         VALUES ('high.lnk', 'C:\\high.exe', 10, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, frequency, created_at, updated_at)
         VALUES ('medium.lnk', 'C:\\medium.exe', 5, ?1, ?2)",
        params![now, now],
    ).expect("Failed to insert entry");

    // Query ordered by frequency
    let lnk_paths: Vec<String> = conn
        .prepare("SELECT lnk_path FROM entries ORDER BY frequency DESC")
        .expect("Failed to prepare query")
        .query_map([], |row| row.get(0))
        .expect("Failed to query entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect results");

    assert_eq!(lnk_paths, vec!["high.lnk", "medium.lnk", "low.lnk"]);
}