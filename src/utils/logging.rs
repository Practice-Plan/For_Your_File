//! Logging utilities

use log::{Level, LevelFilter};

/// Initialize the logger with the specified level
pub fn init_logger(level: LevelFilter) {
    env_logger::Builder::new()
        .filter_level(level)
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                buf,
                "[{} {:5}] {}: {}",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}

/// Initialize the logger from environment variable
pub fn init_logger_from_env() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                buf,
                "[{} {:5}] {}: {}",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}