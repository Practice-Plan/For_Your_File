//! Comprehensive database tests
//!
//! Tests for database initialization, schema creation, index creation, and FTS5 functionality.

use std::time::Instant;

use crate::db::{Database, PoolStatus};
use crate::models::{Entry, Group};

/// Test database initialization performance
#[test]
fn test_database_initialization_performance() {
    let start = Instant::now();

    let db = Database::new_in_memory().expect("Failed to create database");

    let elapsed = start.elapsed();

    // Database initialization must complete within 200ms as per requirements
    assert!(
        elapsed.as_millis() < 200,
        "Database initialization took {}ms, expected < 200ms",
        elapsed.as_millis()
    );
}

/// Test that all required tables are created
#[test]
fn test_all_tables_created() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    let required_tables = ["entries", "groups", "entry_groups", "entries_fts", "schema_version"];

    for table in required_tables {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                rusqlite::params![table],
                |row| row.get(0),
            )
            .expect("Failed to query table existence");

        assert_eq!(count, 1, "Table '{}' was not created", table);
    }
}

/// Test that all required indexes are created
#[test]
fn test_all_indexes_created() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    let required_indexes = [
        "idx_entries_frequency",
        "idx_entries_last_opened",
        "idx_entries_lnk_path",
        "idx_entries_target_path",
        "idx_entries_expires_at",
        "idx_groups_name",
        "idx_entry_groups_entry_id",
        "idx_entry_groups_group_id",
    ];

    for index in required_indexes {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                rusqlite::params![index],
                |row| row.get(0),
            )
            .expect("Failed to query index existence");

        assert_eq!(count, 1, "Index '{}' was not created", index);
    }
}

/// Test that FTS5 virtual table is properly configured
#[test]
fn test_fts5_table_configuration() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Verify FTS5 table exists and is a virtual table
    let table_type: String = conn
        .query_row(
            "SELECT type FROM sqlite_master WHERE name='entries_fts'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5 table type");

    assert_eq!(table_type, "table", "entries_fts should be a table");

    // Verify we can query the FTS5 table structure
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE name='entries_fts'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5 table SQL");

    assert!(sql.contains("fts5"), "entries_fts should use FTS5");
}

/// Test that FTS5 triggers are created
#[test]
fn test_fts5_triggers_created() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    let required_triggers = ["entries_ai", "entries_ad", "entries_au"];

    for trigger in required_triggers {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name=?1",
                rusqlite::params![trigger],
                |row| row.get(0),
            )
            .expect("Failed to query trigger existence");

        assert_eq!(count, 1, "Trigger '{}' was not created", trigger);
    }
}

/// Test FTS5 synchronization after insert
#[test]
fn test_fts5_sync_on_insert() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Insert entry
    let entry = Entry::new("C:/test.lnk".to_string(), "C:/target.exe".to_string());
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![entry.lnk_path, entry.target_path, entry.created_at, entry.updated_at],
    )
    .expect("Failed to insert entry");

    // Verify FTS5 index was updated
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'test'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5");

    assert!(count > 0, "FTS5 index should contain the inserted entry");
}

/// Test FTS5 synchronization after update
#[test]
fn test_fts5_sync_on_update() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Insert entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at) VALUES ('C:/old.lnk', 'C:/old.exe', 0, 0)",
        [],
    )
    .expect("Failed to insert entry");

    // Update entry
    conn.execute(
        "UPDATE entries SET lnk_path = 'C:/updated.lnk', target_path = 'C:/updated.exe' WHERE lnk_path = 'C:/old.lnk'",
        [],
    )
    .expect("Failed to update entry");

    // Verify FTS5 index was updated
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'updated'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5");

    assert!(count > 0, "FTS5 index should contain the updated entry");

    // Verify old content is removed
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'old'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5");

    assert_eq!(count, 0, "FTS5 index should not contain old content");
}

/// Test FTS5 synchronization after delete
#[test]
fn test_fts5_sync_on_delete() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Insert entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at) VALUES ('C:/delete.lnk', 'C:/delete.exe', 0, 0)",
        [],
    )
    .expect("Failed to insert entry");

    // Delete entry
    conn.execute("DELETE FROM entries WHERE lnk_path = 'C:/delete.lnk'", [])
        .expect("Failed to delete entry");

    // Verify FTS5 index was updated
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'delete'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5");

    assert_eq!(count, 0, "FTS5 index should not contain deleted entry");
}

/// Test FTS5 MATCH syntax queries
#[test]
fn test_fts5_match_syntax() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Insert test entries
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
         VALUES ('C:/work.lnk', 'C:/work.exe', 'project, urgent', 'Work project', 0, 0)",
        [],
    )
    .expect("Failed to insert entry");

    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
         VALUES ('C:/personal.lnk', 'C:/game.exe', 'fun, gaming', 'Personal game', 0, 0)",
        [],
    )
    .expect("Failed to insert entry");

    // Test simple MATCH
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'work'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5");

    assert!(count > 0, "FTS5 should find 'work'");

    // Test MATCH with OR
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'work OR game'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5");

    assert_eq!(count, 2, "FTS5 should find both 'work' and 'game'");

    // Test MATCH with column specification
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'tags: project'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query FTS5");

    assert!(count > 0, "FTS5 should find 'project' in tags");
}

/// Test connection pool functionality
#[test]
fn test_connection_pool() {
    let db = Database::new_in_memory().expect("Failed to create database");

    // Get pool status
    let status = db.pool_status();

    // Pool should be configured
    assert!(
        status.connections <= 5,
        "Pool should have at most 5 connections"
    );

    // Get multiple connections
    let conn1 = db.connection().expect("Failed to get connection 1");
    let conn2 = db.connection().expect("Failed to get connection 2");

    // Both should be valid
    conn1.execute("SELECT 1", []).expect("Connection 1 failed");
    conn2.execute("SELECT 1", []).expect("Connection 2 failed");
}

/// Test query performance with indexes
#[test]
fn test_index_query_performance() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Insert test data
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, frequency, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, 0)",
            rusqlite::params![format!("C:/test{}.lnk", i), format!("C:/target{}.exe", i), i % 100],
        )
        .expect("Failed to insert entry");
    }

    // Test indexed query performance
    let start = Instant::now();

    let _count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entries WHERE frequency > 50",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query");

    let elapsed = start.elapsed();

    // Should complete in under 1ms for 1000 rows
    assert!(
        elapsed.as_micros() < 1000,
        "Indexed query took {} microseconds",
        elapsed.as_micros()
    );
}

/// Test entry_groups junction table relationships
#[test]
fn test_entry_groups_relationship() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Create entry
    conn.execute(
        "INSERT INTO entries (lnk_path, target_path, created_at, updated_at)
         VALUES ('C:/test.lnk', 'C:/target.exe', 0, 0)",
        [],
    )
    .expect("Failed to insert entry");
    let entry_id = conn.last_insert_rowid();

    // Create groups
    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at) VALUES ('Work', '#FF0000', 0, 0)",
        [],
    )
    .expect("Failed to insert group");
    let group1_id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO groups (name, color, created_at, updated_at) VALUES ('Personal', '#00FF00', 0, 0)",
        [],
    )
    .expect("Failed to insert group");
    let group2_id = conn.last_insert_rowid();

    // Create relationships
    conn.execute(
        "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        rusqlite::params![entry_id, group1_id],
    )
    .expect("Failed to create relationship 1");

    conn.execute(
        "INSERT INTO entry_groups (entry_id, group_id) VALUES (?1, ?2)",
        rusqlite::params![entry_id, group2_id],
    )
    .expect("Failed to create relationship 2");

    // Verify relationship
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_groups WHERE entry_id = ?1",
            rusqlite::params![entry_id],
            |row| row.get(0),
        )
        .expect("Failed to count relationships");

    assert_eq!(count, 2, "Entry should belong to 2 groups");
}

/// Test schema version tracking
#[test]
fn test_schema_version_tracking() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let conn = db.connection().expect("Failed to get connection");

    // Verify version table exists and has version
    let version: i64 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("Failed to get schema version");

    assert_eq!(version, 1, "Schema version should be 1");
}

/// Test database clone shares pool
#[test]
fn test_database_clone_shares_pool() {
    let db1 = Database::new_in_memory().expect("Failed to create database");
    let db2 = db1.clone();

    // Both should work
    let conn1 = db1.connection().expect("Failed to get connection from db1");
    let conn2 = db2.connection().expect("Failed to get connection from db2");

    conn1.execute("SELECT 1", []).expect("Connection 1 failed");
    conn2.execute("SELECT 1", []).expect("Connection 2 failed");
}