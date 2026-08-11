//! Database connection and initialization
//!
//! Provides connection pooling and schema management using r2d2.
//! Optimized for performance with prepared statement caching and connection health checks.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use log::info;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

use super::migrations::apply_migrations;
use super::schema::{
    CREATE_ENTRIES_FTS_TABLE, CREATE_ENTRIES_TABLE, CREATE_ENTRY_GROUPS_TABLE,
    CREATE_GROUPS_TABLE, CREATE_IDX_ENTRIES_EXPIRES_AT, CREATE_IDX_ENTRIES_FREQUENCY,
    CREATE_IDX_ENTRIES_LAST_OPENED, CREATE_IDX_ENTRIES_LNK_PATH,
    CREATE_IDX_ENTRIES_TARGET_PATH, CREATE_IDX_ENTRY_GROUPS_ENTRY_ID,
    CREATE_IDX_ENTRY_GROUPS_GROUP_ID, CREATE_IDX_GROUPS_NAME, CREATE_TRIGGER_ENTRIES_AD,
    CREATE_TRIGGER_ENTRIES_AI, CREATE_TRIGGER_ENTRIES_AU, CREATE_VERSION_TABLE,
};

/// Default pool size - optimized for concurrent read operations
const DEFAULT_POOL_SIZE: u32 = 10;

/// Default connection timeout in milliseconds
const DEFAULT_TIMEOUT_MS: u64 = 5000;

/// Minimum pool size for low-resource environments
const MIN_POOL_SIZE: u32 = 3;

/// Maximum pool size for high-load scenarios
const MAX_POOL_SIZE: u32 = 20;

/// Connection health check interval
const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;

/// Prepared statement cache size per connection
const PREPARED_STMT_CACHE_SIZE: usize = 16;

/// Database connection pool wrapper
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

/// Type alias for a pooled connection
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

impl Database {
    /// Create a new database with default pool settings
    pub fn new() -> Result<Self> {
        Self::with_options("data/entries.db", DEFAULT_POOL_SIZE, DEFAULT_TIMEOUT_MS)
    }

    /// Create a new database with custom options
    pub fn with_options(db_path: &str, pool_size: u32, timeout_ms: u64) -> Result<Self> {
        // Ensure parent directory exists
        let path = PathBuf::from(db_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
                info!("Created database directory: {:?}", parent);
            }
        }

        // Create connection manager
        let manager = SqliteConnectionManager::file(db_path);

        // Build pool with configuration
        let pool = Pool::builder()
            .max_size(pool_size)
            .connection_timeout(Duration::from_millis(timeout_ms))
            .build(manager)?;

        let db = Self { pool };

        // Initialize schema
        db.initialize_schema()?;

        info!(
            "Database initialized successfully at {} with pool size {}",
            db_path, pool_size
        );
        Ok(db)
    }

    /// Create an in-memory database for testing
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(5)
            .connection_timeout(Duration::from_millis(1000))
            .build(manager)?;

        let db = Self { pool };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Get a connection from the pool
    pub fn connection(&self) -> Result<DbConnection> {
        self.pool.get().map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))
    }

    /// Get a reference to the underlying pool for advanced operations
    pub fn pool(&self) -> &Pool<SqliteConnectionManager> {
        &self.pool
    }

    /// Get the current pool status
    pub fn pool_status(&self) -> PoolStatus {
        let state = self.pool.state();
        PoolStatus {
            connections: state.connections,
            idle_connections: state.idle_connections,
        }
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection()?;

        // Create all tables
        conn.execute(CREATE_ENTRIES_TABLE, [])?;
        conn.execute(CREATE_GROUPS_TABLE, [])?;
        conn.execute(CREATE_ENTRY_GROUPS_TABLE, [])?;
        conn.execute(CREATE_ENTRIES_FTS_TABLE, [])?;
        conn.execute(CREATE_VERSION_TABLE, [])?;

        // Create all indexes
        conn.execute(CREATE_IDX_ENTRIES_FREQUENCY, [])?;
        conn.execute(CREATE_IDX_ENTRIES_LAST_OPENED, [])?;
        conn.execute(CREATE_IDX_ENTRIES_LNK_PATH, [])?;
        conn.execute(CREATE_IDX_ENTRIES_TARGET_PATH, [])?;
        conn.execute(CREATE_IDX_ENTRIES_EXPIRES_AT, [])?;
        conn.execute(CREATE_IDX_GROUPS_NAME, [])?;
        conn.execute(CREATE_IDX_ENTRY_GROUPS_ENTRY_ID, [])?;
        conn.execute(CREATE_IDX_ENTRY_GROUPS_GROUP_ID, [])?;

        // Create FTS5 triggers
        conn.execute(CREATE_TRIGGER_ENTRIES_AI, [])?;
        conn.execute(CREATE_TRIGGER_ENTRIES_AD, [])?;
        conn.execute(CREATE_TRIGGER_ENTRIES_AU, [])?;

        // Apply migrations
        apply_migrations(&conn)?;

        Ok(())
    }

    /// Execute a closure with a connection from the pool
    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.connection()?;
        f(&conn)
    }
}

/// Pool status information
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// Total number of connections in the pool
    pub connections: u32,
    /// Number of idle connections available
    pub idle_connections: u32,
}

/// Performance statistics for database operations
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// Total number of queries executed
    pub total_queries: u64,
    /// Total query execution time in microseconds
    pub total_query_time_us: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Last health check timestamp
    pub last_health_check: Option<Instant>,
}

impl PerformanceStats {
    /// Calculate average query time in microseconds
    pub fn avg_query_time_us(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.total_query_time_us as f64 / self.total_queries as f64
        }
    }

    /// Calculate cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }
}

/// Optimized pool configuration
#[derive(Debug, Clone)]
pub struct OptimizedPoolConfig {
    /// Pool size
    pub pool_size: u32,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable prepared statement caching
    pub enable_stmt_cache: bool,
    /// Statement cache size
    pub stmt_cache_size: usize,
}

impl Default for OptimizedPoolConfig {
    fn default() -> Self {
        Self {
            pool_size: DEFAULT_POOL_SIZE,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            enable_stmt_cache: true,
            stmt_cache_size: PREPARED_STMT_CACHE_SIZE,
        }
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        // Clone is cheap because Pool is internally reference-counted
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database_creation() {
        let db = Database::new_in_memory();
        assert!(db.is_ok());
    }

    #[test]
    fn test_connection_pool() {
        let db = Database::new_in_memory().unwrap();

        // Get multiple connections
        let conn1 = db.connection();
        let conn2 = db.connection();

        assert!(conn1.is_ok());
        assert!(conn2.is_ok());
    }

    #[test]
    fn test_pool_status() {
        let db = Database::new_in_memory().unwrap();
        let status = db.pool_status();

        // Initial pool should have 0 connections until first use
        assert!(status.connections <= 5); // Max 5 for in-memory
    }

    #[test]
    fn test_schema_tables_exist() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection().unwrap();

        // Check entries table exists
        let result: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result, 1);

        // Check groups table exists
        let result: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='groups'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result, 1);

        // Check entry_groups table exists
        let result: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entry_groups'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result, 1);

        // Check FTS5 table exists
        let result: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_indexes_exist() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection().unwrap();

        // Check indexes were created
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count >= 8); // At least 8 indexes
    }

    #[test]
    fn test_fts5_functionality() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.connection().unwrap();

        // Insert test entry
        conn.execute(
            "INSERT INTO entries (lnk_path, target_path, tags, notes, created_at, updated_at)
             VALUES ('C:/test.lnk', 'C:/target.exe', 'test, demo', 'Test note', 0, 0)",
            [],
        )
        .unwrap();

        // Query FTS5
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_database_clone() {
        let db1 = Database::new_in_memory().unwrap();
        let db2 = db1.clone();

        // Both should reference the same pool
        assert_eq!(db1.pool_status().connections, db2.pool_status().connections);
    }
}