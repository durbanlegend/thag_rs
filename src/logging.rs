#![allow(clippy::uninlined_format_args)]

/// An enum of the supported verbosity levels.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum Verbosity {
    Quieter,
    Quiet,
    Normal,
    Verbose,
}

/// Define the Logger.
pub struct Logger {
    pub verbosity: Verbosity,
}

impl Logger {
    /// Construct a new Logger with the given Verbosity level.
    #[must_use]
    pub fn new(verbosity: Verbosity) -> Self {
        Logger { verbosity }
    }

    /// Log a message if it passes the verbosity filter.
    pub fn log(&self, verbosity: Verbosity, message: &str) {
        if verbosity as u8 <= self.verbosity as u8 {
            println!("{}", message);
        }
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }
}

use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Mutex;

lazy_static! {
    /// The common Logger instance to use.
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new(Verbosity::Normal)); // Default to Normal
}

/// Set the logging verbosity for the current execution.
/// # Panics
/// Will panic if the logger mutex cannot be unlocked..
pub fn set_global_verbosity(verbosity: Verbosity) {
    let mut logger = LOGGER.lock().unwrap();
    logger.set_verbosity(verbosity);
}

#[macro_export]
macro_rules! log {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            let logger = $crate::logging::LOGGER.lock().unwrap();
            logger.log($verbosity, &format!($($arg)*));
        }
    };
}
