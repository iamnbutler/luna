//! This module implements a file-based logging system for Luna that writes log messages
//! to a file in the user's home directory at ~/.luna/logs/{run_metadata}/log.

use anyhow::{Context, Result};
use chrono::Local;
use dirs::home_dir;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Thread-local storage to hold the current run ID and log path
thread_local! {
    static CURRENT_RUN_ID: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
    static CURRENT_LOG_PATH: std::cell::RefCell<Option<PathBuf>> = std::cell::RefCell::new(None);
}

/// Structure representing the Luna logger
pub struct LunaLogger {
    level: LevelFilter,
    file: Arc<Mutex<File>>,
    run_id: String,
    log_path: PathBuf,
}

impl LunaLogger {
    /// Create a new LunaLogger with the specified log level
    ///
    /// This will create a log file at ~/.luna/logs/{timestamp}_{uuid}/log
    /// where {timestamp} is the current time in ISO 8601 format and {uuid}
    /// is a unique identifier for this run.
    pub fn new(level: LevelFilter) -> Result<Self> {
        // Generate run metadata (timestamp and UUID)
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let uuid_string = Uuid::new_v4().to_string();
        let uuid = uuid_string.split('-').next().unwrap_or("unknown");
        let run_id = format!("{timestamp}_{uuid}");

        // Create the log directory
        let log_dir = Self::get_log_dir(&run_id)?;
        create_dir_all(&log_dir)
            .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

        // Create the log file
        let log_path = log_dir.join("log");
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

        // Store the run ID and log path in thread-local storage
        CURRENT_RUN_ID.with(|cell| {
            *cell.borrow_mut() = Some(run_id.clone());
        });

        CURRENT_LOG_PATH.with(|cell| {
            *cell.borrow_mut() = Some(log_path.clone());
        });

        Ok(Self {
            level,
            file: Arc::new(Mutex::new(file)),
            run_id,
            log_path,
        })
    }

    /// Returns the path to the log directory for this run
    pub fn get_log_dir(run_id: &str) -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".luna").join("logs").join(run_id))
    }

    /// Initialize the logger with the specified log level
    pub fn init(level: LevelFilter) -> Result<()> {
        let logger = Self::new(level)?;
        let run_id = logger.run_id.clone();
        let log_path = logger.log_path.clone();

        log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(level))
            .map_err(|e| anyhow::anyhow!("Failed to set logger: {}", e))?;

        log::info!("Luna logger initialized. Run ID: {}", run_id);
        log::info!("Log file: {}", log_path.display());
        Ok(())
    }

    /// Get the current run ID if the logger has been initialized
    pub fn current_run_id() -> Option<String> {
        CURRENT_RUN_ID.with(|cell| cell.borrow().clone())
    }

    /// Get the current log file path if the logger has been initialized
    pub fn current_log_path() -> Option<PathBuf> {
        CURRENT_LOG_PATH.with(|cell| cell.borrow().clone())
    }
}

impl Log for LunaLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            let level = record.level();
            let target = record.target();
            let message = format!("{} {} [{}] {}", timestamp, level, target, record.args());

            if let Ok(mut file) = self.file.lock() {
                // Ignore errors when writing to log file; we don't want to crash the application
                let _ = writeln!(file, "{}", message);
                let _ = file.flush();
            }

            // Also print to stderr for visibility during development
            eprintln!("{}", message);
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

/// Returns the path to the current log file.
///
/// This function will return `None` if logging hasn't been initialized yet.
pub fn get_current_log_file_path() -> Option<String> {
    LunaLogger::current_log_path().map(|p| p.to_string_lossy().to_string())
}

/// Returns the current run ID.
///
/// This function will return `None` if logging hasn't been initialized yet.
pub fn get_current_run_id() -> Option<String> {
    LunaLogger::current_run_id()
}

/// Creates a log entry that separates sections in the log file.
///
/// This is useful for marking the beginning of a new operation or task.
pub fn log_section(name: &str) {
    let separator = "=".repeat(50);
    log::info!("{}", separator);
    log::info!("SECTION: {}", name);
    log::info!("{}", separator);
}

/// Dumps the contents of a file to the log.
///
/// This is useful for debugging purposes.
pub fn log_file_contents(path: &Path) -> Result<()> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    log::debug!("Contents of {}: ", path.display());
    for (i, line) in contents.lines().enumerate() {
        log::debug!("{:4}: {}", i + 1, line);
    }

    Ok(())
}
