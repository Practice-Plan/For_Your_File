//! Group management integration tests
//!
//! Tests group creation, entry associations, batch operations,
//! and group export/import functionality.

mod common;

use common::*;
use rusqlite::params;

/// Test complete group lifecycle: Create → Add entries → Batch operations → Delete
#[test]
fn test_group_lifecycle() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Step 1: Create a group
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Development', '#FF5733', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group");

    let group_id = conn.last_insert_rowid();

    // Verify group was created
    let (name, color): (String, String) = conn
        .query_row(
            "SELECT name, color FROM groups WHERE id = ?1",
            params![group_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("Failed to query group");

    assert_eq!(name, "Development");
    assert_eq!(color, "#FF5733");

    // Step 2: Create entries
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('vscode.lnk', 'C:\\VSCode.exe', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create entry 1");

    let entry1_id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('git.lnk', 'C:\\git.exe', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create entry 2");

    let entry2_id = conn.last_insert_rowid();

    // Step 3: Add entries to group
    conn.execute(
        "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        params![entry1_id, group_id],
    )
    .expect("Failed to add entry 1 to group");

    conn.execute(
        "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        params![entry2_id, group_id],
    )
    .expect("Failed to add entry 2 to group");

    // Verify associations
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )
        .expect("Failed to count group entries");

    assert_eq!(count, 2);

    // Step 4: Batch remove entries from group
    conn.execute(
        "DELETE FROM entry_groups WHERE group_id = ?1",
        params![group_id],
    )
    .expect("Failed to remove entries from group");

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )
        .expect("Failed to count group entries");

    assert_eq!(count, 0);

    // Step 5: Delete group
    conn.execute("DELETE FROM groups WHERE id = ?1", params![group_id])
        .expect("Failed to delete group");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM groups", [], |row| row.get(0))
        .expect("Failed to count groups");

    assert_eq!(count, 0);
}

/// Test entry-group associations
#[test]
fn test_entry_group_associations() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create groups
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Work', '#FF0000', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group 1");

    let work_group_id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Personal', '#00FF00', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group 2");

    let personal_group_id = conn.last_insert_rowid();

    // Create entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('app.lnk', 'C:\\app.exe', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create entry");

    let entry_id = conn.last_insert_rowid();

    // Associate entry with multiple groups
    conn.execute(
        "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        params![entry_id, work_group_id],
    )
    .expect("Failed to associate with work group");

    conn.execute(
        "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        params![entry_id, personal_group_id],
    )
    .expect("Failed to associate with personal group");

    // Verify entry belongs to both groups
    let group_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE entry_id = ?1",
            params![entry_id],
            |row| row.get(0),
        )
        .expect("Failed to count entry groups");

    assert_eq!(group_count, 2);

    // Verify we can query entries by group
    for group_id in &[work_group_id, personal_group_id] {
        let entry_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entry_groups WHERE group_id = ?1",
                params![group_id],
                |row| row.get(0),
            )
            .expect("Failed to count group entries");

        assert_eq!(entry_count, 1);
    }
}

/// Test batch add to group
#[test]
fn test_batch_add_to_group() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create group
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Test Group', '#0000FF', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group");

    let group_id = conn.last_insert_rowid();

    // Create multiple entries
    let mut entry_ids = Vec::new();
    for i in 1..=5 {
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                format!("app{}.lnk", i),
                format!("C:\\app{}.exe", i),
                now,
                now
            ],
        )
        .expect(&format!("Failed to create entry {}", i));

        entry_ids.push(conn.last_insert_rowid());
    }

    // Batch add all entries to group
    for entry_id in &entry_ids {
        conn.execute(
            "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
            params![entry_id, group_id],
        )
        .expect("Failed to add entry to group");
    }

    // Verify all entries are in the group
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )
        .expect("Failed to count group entries");

    assert_eq!(count, 5);
}

/// Test batch remove from group
#[test]
fn test_batch_remove_from_group() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create group
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Test Group', '#0000FF', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group");

    let group_id = conn.last_insert_rowid();

    // Create entries and add to group
    let mut entry_ids = Vec::new();
    for i in 1..=5 {
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                format!("app{}.lnk", i),
                format!("C:\\app{}.exe", i),
                now,
                now
            ],
        )
        .expect(&format!("Failed to create entry {}", i));

        let entry_id = conn.last_insert_rowid();
        entry_ids.push(entry_id);

        conn.execute(
            "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
            params![entry_id, group_id],
        )
        .expect("Failed to add entry to group");
    }

    // Batch remove first 2 entries from group
    for entry_id in &entry_ids[..2] {
        conn.execute(
            "DELETE FROM entry_groups WHERE entry_id = ?1 AND group_id = ?2",
            params![entry_id, group_id],
        )
        .expect("Failed to remove entry from group");
    }

    // Verify remaining entries
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )
        .expect("Failed to count group entries");

    assert_eq!(count, 3);
}

/// Test group export functionality
#[test]
fn test_group_export() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create group
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Export Test', '#123456', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group");

    let group_id = conn.last_insert_rowid();

    // Create entries and add to group
    let mut entry_ids = Vec::new();
    for i in 1..=3 {
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                format!("app{}.lnk", i),
                format!("C:\\app{}.exe", i),
                now,
                now
            ],
        )
        .expect(&format!("Failed to create entry {}", i));

        let entry_id = conn.last_insert_rowid();
        entry_ids.push(entry_id);

        conn.execute(
            "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
            params![entry_id, group_id],
        )
        .expect("Failed to add entry to group");
    }

    // Simulate export by querying group data
    let (name, color): (String, String) = conn
        .query_row(
            "SELECT name, color FROM groups WHERE id = ?1",
            params![group_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("Failed to query group");

    let exported_entry_ids: Vec<i64> = conn
        .prepare("SELECT entry_id FROM entry_groups WHERE group_id = ?1")
        .expect("Failed to prepare query")
        .query_map(params![group_id], |row| row.get(0))
        .expect("Failed to query entry IDs")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect entry IDs");

    // Verify export data
    assert_eq!(name, "Export Test");
    assert_eq!(color, "#123456");
    assert_eq!(exported_entry_ids.len(), 3);
    assert!(exported_entry_ids.contains(&entry_ids[0]));
    assert!(exported_entry_ids.contains(&entry_ids[1]));
    assert!(exported_entry_ids.contains(&entry_ids[2]));
}

/// Test group import functionality
#[test]
fn test_group_import() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create entries (simulating existing data)
    let mut entry_ids = Vec::new();
    for i in 1..=3 {
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                format!("existing{}.lnk", i),
                format!("C:\\existing{}.exe", i),
                now,
                now
            ],
        )
        .expect(&format!("Failed to create entry {}", i));

        entry_ids.push(conn.last_insert_rowid());
    }

    // Import a new group with entry associations
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Imported Group', '#ABCDEF', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create imported group");

    let imported_group_id = conn.last_insert_rowid();

    // Associate entries (skip non-existent)
    for entry_id in &entry_ids {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM entries WHERE id = ?1",
                params![entry_id],
                |_| Ok(true),
            )
            .optional()
            .expect("Failed to check entry")
            .is_some();

        if exists {
            conn.execute(
                "INSERT OR IGNORE INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
                params![entry_id, imported_group_id],
            )
            .expect("Failed to add entry to imported group");
        }
    }

    // Verify import
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE group_id = ?1",
            params![imported_group_id],
            |row| row.get(0),
        )
        .expect("Failed to count imported group entries");

    assert_eq!(count, 3);
}

/// Test group uniqueness constraint
#[test]
fn test_group_uniqueness() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create first group
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Unique Group', '#000000', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group");

    // Try to create duplicate group (should fail due to UNIQUE constraint on name)
    let result = conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Unique Group', '#FFFFFF', ?1, ?2)",
        params![now, now],
    );

    assert!(result.is_err(), "Expected unique constraint violation");
}

/// Test cascade delete when deleting entries
#[test]
fn test_entry_group_cascade_delete() {
    let fixture = TestFixture::new();
    let conn = fixture
        .create_test_database()
        .expect("Failed to create database");

    let now = chrono::Utc::now().timestamp();

    // Create group
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at)
         VALUES ('Test Group', '#0000FF', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create group");

    let group_id = conn.last_insert_rowid();

    // Create entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('app.lnk', 'C:\\app.exe', ?1, ?2)",
        params![now, now],
    )
    .expect("Failed to create entry");

    let entry_id = conn.last_insert_rowid();

    // Associate entry with group
    conn.execute(
        "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        params![entry_id, group_id],
    )
    .expect("Failed to add entry to group");

    // Delete entry (should cascade delete from entry_groups)
    conn.execute("DELETE FROM entries WHERE id = ?1", params![entry_id])
        .expect("Failed to delete entry");

    // Verify association was deleted
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )
        .expect("Failed to count group entries");

    assert_eq!(count, 0, "Entry-group association should be deleted");
}
