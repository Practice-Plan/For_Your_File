//! LNK file operations module
//!
//! Handles creation, reading, and management of Windows .lnk shortcut files.

mod parser;
mod creator;
mod validator;
mod shell;
mod manager;

#[cfg(test)]
mod lnk_tests;

pub use parser::*;
pub use creator::*;
pub use validator::*;
pub use shell::*;
pub use manager::*;