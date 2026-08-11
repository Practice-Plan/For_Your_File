//! Search functionality module
//!
//! Provides full-text search capabilities using SQLite FTS5 with Pinyin support.

mod fts;
mod pinyin;
mod ranking;
mod utils;

pub use fts::*;
pub use pinyin::*;
pub use ranking::*;
pub use utils::*;

#[cfg(test)]
mod fts_tests;