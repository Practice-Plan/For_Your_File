//! Common test utilities and fixtures for integration tests
//!
//! Provides shared functionality for creating test databases,
//! sample LNK files, and other test fixtures.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture manager for integration tests
pub struct TestFixture {
    /// Temporary directory for test files
    pub temp_dir: TempDir,
    /// Database path
    pub db_path: PathBuf,
    /// Sample LNK files directory
    pub lnk_dir: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with isolated environment
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let lnk_dir = temp_dir.path().join("lnk_files");

        fs::create_dir_all(&lnk_dir).expect("Failed to create LNK directory");

        Self {
            temp_dir,
            db_path,
            lnk_dir,
        }
    }

    /// Create a sample LNK file pointing to an executable
    pub fn create_exe_lnk(&self, name: &str, target: &str) -> PathBuf {
        self.create_lnk_file(name, target, "exe")
    }

    /// Create a sample LNK file pointing to a document
    pub fn create_doc_lnk(&self, name: &str, target: &str) -> PathBuf {
        self.create_lnk_file(name, target, "doc")
    }

    /// Create a sample LNK file pointing to a folder
    pub fn create_folder_lnk(&self, name: &str, target: &str) -> PathBuf {
        self.create_lnk_file(name, target, "folder")
    }

    /// Create a mock LNK file for testing
    fn create_lnk_file(&self, name: &str, target: &str, target_type: &str) -> PathBuf {
        let lnk_path = self.lnk_dir.join(format!("{}.lnk", name));

        // Create target file/folder
        let target_path = match target_type {
            "folder" => {
                let path = self.temp_dir.path().join(target);
                fs::create_dir_all(&path).expect("Failed to create target folder");
                path.to_string_lossy().to_string()
            }
            _ => {
                let path = self.temp_dir.path().join(target);
                let mut file = File::create(&path).expect("Failed to create target file");
                writeln!(file, "Test content for {}", name).unwrap();
                path.to_string_lossy().to_string()
            }
        };

        // Write LNK metadata (simplified for testing)
        let mut file = File::create(&lnk_path).expect("Failed to create LNK file");
        writeln!(file, "[Shortcut]").unwrap();
        writeln!(file, "Target={}", target_path).unwrap();
        writeln!(file, "Type={}", target_type).unwrap();

        lnk_path
    }

    /// Create a test database with schema
    pub fn create_test_database(&self) -> rusqlite::Result<rusqlite::Connection> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        self.initialize_database_schema(&conn)?;
        Ok(conn)
    }

    /// Initialize database schema for testing
    fn initialize_database_schema(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lnk_path TEXT NOT NULL UNIQUE,
                target_path TEXT NOT NULL,
                parameters TEXT,
                working_dir TEXT,
                tags TEXT,
                notes TEXT,
                frequency INTEGER DEFAULT 0,
                last_opened INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entry_groups (
                entry_id INTEGER NOT NULL,
                group_id INTEGER NOT NULL,
                PRIMARY KEY (entry_id, group_id),
                FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
                FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
                lnk_path,
                target_path,
                tags,
                notes,
                content=entries,
                content_rowid=id
            );

            CREATE INDEX IF NOT EXISTS idx_entries_frequency ON entries(frequency DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_last_opened ON entries(last_opened DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_expires_at ON entries(expires_at);
            "#,
        )?;
        Ok(())
    }

    /// Get the database path
    pub fn database_path(&self) -> &Path {
        &self.db_path
    }

    /// Clean up test resources
    pub fn cleanup(&self) {
        // TempDir automatically cleans up when dropped
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a sample entry for testing
pub fn create_sample_entry(lnk_path: String, target_path: String) -> app_lib::Entry {
    app_lib::Entry::new(lnk_path, target_path)
}

/// Helper to wait for async operations
pub async fn wait_for_ms(ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}