#![allow(clippy::uninlined_format_args)]
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Mutex;
use strum::EnumString;

#[cfg(debug_assertions)]
use crate::debug_log;
use crate::ThagError;

/// An enum of the supported verbosity levels.
#[derive(Clone, Copy, Debug, Default, Deserialize, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Verbosity {
    Quieter,
    Quiet,
    #[default]
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
    pub const fn new(verbosity: Verbosity) -> Self {
        Self { verbosity }
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
        #[cfg(debug_assertions)]
        debug_log!("Verbosity set to {verbosity:?}");
    }
}

lazy_static! {
    /// The common Logger instance to use.
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new(Verbosity::Normal)); // Default to Normal
}

/// Set the logging verbosity for the current execution.
/// # Errors
/// Will return `Err` if the logger mutex cannot be locked.
pub fn set_global_verbosity(verbosity: Verbosity) -> Result<(), ThagError> {
    LOGGER.lock()?.set_verbosity(verbosity);
    Ok(())
}

#[macro_export]
macro_rules! log {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            $crate::logging::LOGGER.lock().unwrap().log($verbosity, &format!($($arg)*))
        }
    };
}
