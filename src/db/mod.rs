//! Database operations module
//!
//! Handles all SQLite database interactions including FTS5 search.

mod database;
mod migrations;
mod operations;
mod schema;
mod tags;
mod usage;
mod batch;
mod groups;

#[cfg(test)]
mod database_tests;

#[cfg(test)]
mod operations_tests;

pub use database::{Database, DbConnection, PoolStatus};
pub use migrations::{apply_migrations, get_current_version, MIGRATIONS};
pub use operations::*;
pub use schema::*;
pub use tags::TagOperations;
pub use usage::{UsageOperations, EntryUsage};
pub use batch::{BatchOperations, BatchResult};
pub use groups::{GroupOperations, GroupWithCount, GroupExport};