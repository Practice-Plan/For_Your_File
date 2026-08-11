//! Expiration reminder system for LNK File Management Center
//!
//! Provides time-based notifications for temporary files.

mod manager;
mod timer;

#[cfg(test)]
mod manager_tests;

pub use manager::*;
// Timer module is not currently used, but kept for future use
// pub use timer::*;
