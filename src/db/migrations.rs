//! Database migration system
//!
//! Handles version tracking and schema migrations.

use anyhow::Result;
use log::{info, warn};
use rusqlite::{Connection, OptionalExtension};

use super::schema::{
    GET_SCHEMA_VERSION, INSERT_SCHEMA_VERSION, INDEX_STATEMENTS, SCHEMA_VERSION,
    TABLE_STATEMENTS, TRIGGER_STATEMENTS,
};

/// Migration definition
pub struct Migration {
    /// Version number for this migration
    pub version: i32,
    /// Human-readable description
    pub description: &'static str,
    /// SQL statements for up migration
    pub up: &'static [&'static str],
    /// SQL statements for down migration (optional)
    pub down: &'static [&'static str],
}

/// List of all migrations in order
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema with entries, groups, entry_groups, FTS5, and indexes",
        up: &[],
        down: &[],
    },
];

/// Get the current schema version from the database
pub fn get_current_version(conn: &Connection) -> Result<Option<i32>> {
    let result = conn
        .query_row(GET_SCHEMA_VERSION, [], |row| row.get::<_, i32>(0))
        .optional()?;

    Ok(result)
}

/// Record a migration as applied
fn record_migration(conn: &Connection, version: i32) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(INSERT_SCHEMA_VERSION, rusqlite::params![version, now])?;
    Ok(())
}

/// Apply all pending migrations
pub fn apply_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_current_version(conn)?;

    match current_version {
        Some(version) => {
            info!("Current schema version: {}", version);

            if version < SCHEMA_VERSION {
                info!("Applying migrations from version {} to {}", version, SCHEMA_VERSION);
                apply_pending_migrations(conn, version)?;
            } else {
                info!("Schema is up to date");
            }
        }
        None => {
            info!("No schema version found, initializing fresh database");
            initialize_fresh_database(conn)?;
        }
    }

    Ok(())
}

/// Apply pending migrations from the given starting version
fn apply_pending_migrations(conn: &Connection, from_version: i32) -> Result<()> {
    for migration in MIGRATIONS {
        if migration.version > from_version {
            info!(
                "Applying migration v{}: {}",
                migration.version, migration.description
            );

            // Execute up migration statements
            for stmt in migration.up {
                conn.execute_batch(stmt)?;
            }

            // Record the migration
            record_migration(conn, migration.version)?;

            info!("Migration v{} applied successfully", migration.version);
        }
    }

    Ok(())
}

/// Initialize a fresh database with the current schema
fn initialize_fresh_database(conn: &Connection) -> Result<()> {
    info!("Initializing fresh database schema");

    // Create all tables
    for stmt in TABLE_STATEMENTS {
        conn.execute(stmt, [])?;
    }

    // Create all indexes
    for stmt in INDEX_STATEMENTS {
        conn.execute(stmt, [])?;
    }

    // Create all triggers
    for stmt in TRIGGER_STATEMENTS {
        conn.execute(stmt, [])?;
    }

    // Record the initial version
    record_migration(conn, SCHEMA_VERSION)?;

    info!("Fresh database initialized with schema version {}", SCHEMA_VERSION);

    Ok(())
}

/// Roll back migrations to a target version (for future use)
pub fn rollback_to_version(conn: &Connection, target_version: i32) -> Result<()> {
    let current_version = get_current_version(conn)?.unwrap_or(0);

    if target_version >= current_version {
        warn!("Target version {} is not less than current version {}", target_version, current_version);
        return Ok(());
    }

    // Apply down migrations in reverse order
    for migration in MIGRATIONS.iter().rev() {
        if migration.version > target_version && migration.version <= current_version {
            info!(
                "Rolling back migration v{}: {}",
                migration.version, migration.description
            );

            // Execute down migration statements
            for stmt in migration.down {
                conn.execute_batch(stmt)?;
            }

            // Remove the version record
            conn.execute(
                "DELETE FROM schema_version WHERE version = ?1",
                rusqlite::params![migration.version],
            )?;

            info!("Rollback v{} completed", migration.version);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_get_current_version_empty() {
        let conn = Connection::open_in_memory().unwrap();

        // Version table doesn't exist yet
        let result = get_current_version(&conn);
        // Should return None since table doesn't exist
        assert!(result.is_err() || result.unwrap().is_none());
    }

    #[test]
    fn test_apply_migrations_fresh() {
        let conn = Connection::open_in_memory().unwrap();

        // First create the version table
        conn.execute(super::super::schema::CREATE_VERSION_TABLE, []).unwrap();

        // Apply migrations
        let result = apply_migrations(&conn);
        assert!(result.is_ok());

        // Check version
        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, Some(SCHEMA_VERSION));
    }

    #[test]
    fn test_migrations_list_not_empty() {
        assert!(!MIGRATIONS.is_empty());
    }

    #[test]
    fn test_migrations_sequential() {
        for (i, migration) in MIGRATIONS.iter().enumerate() {
            assert_eq!(migration.version, (i + 1) as i32);
        }
    }
}