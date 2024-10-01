#![allow(clippy::uninlined_format_args)]
use env_logger::{Builder, Env, WriteStyle};
use firestorm::profile_fn;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use strum::EnumString;

use crate::{debug_log, Cli, ThagResult, MAYBE_CONFIG};

static DEBUG_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initializes and returns the global verbosity setting.
pub fn get_verbosity() -> Verbosity {
    LOGGER.lock().unwrap().verbosity.clone()
}

#[allow(clippy::module_name_repetitions)]
pub fn enable_debug_logging() {
    DEBUG_LOG_ENABLED.store(true, Ordering::SeqCst);
}

pub fn is_debug_logging_enabled() -> bool {
    DEBUG_LOG_ENABLED.load(Ordering::SeqCst)
}

/// An enum of the supported verbosity levels.
#[derive(Clone, Copy, Debug, Default, Deserialize, EnumString, PartialEq, PartialOrd, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Verbosity {
    Quieter = 0,
    Quiet = 1,
    #[default]
    Normal = 2,
    Verbose = 3,
    Debug = 4,
}

pub type V = Verbosity;

/// Define the Logger.
#[derive(Debug)]
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

        debug_log!("Verbosity set to {verbosity:?}");
    }

    /// Return the verbosity level
    pub fn verbosity(&mut self) -> Verbosity {
        self.verbosity
    }
}

lazy_static! {
    /// The common Logger instance to use.
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new(Verbosity::Normal)); // Default to Normal
}

#[inline]
/// Determine the desired logging verbosity for the current execution.
/// # Errors
/// Will return `Err` if the logger mutex cannot be locked.
pub fn set_verbosity(args: &Cli) -> ThagResult<()> {
    profile_fn!(set_verbosity);

    let verbosity = if args.verbose >= 2 {
        Verbosity::Debug
    } else if args.verbose == 1 {
        Verbosity::Verbose
    } else if args.quiet == 1 {
        Verbosity::Quiet
    } else if args.quiet >= 2 {
        Verbosity::Quieter
    } else if args.normal {
        Verbosity::Normal
    } else if let Some(config) = &*MAYBE_CONFIG {
        config.logging.default_verbosity
    } else {
        Verbosity::Normal
    };
    set_global_verbosity(verbosity)
}

/// Set the logging verbosity for the current execution.
/// # Errors
/// Will return `Err` if the logger mutex cannot be locked.
pub fn set_global_verbosity(verbosity: Verbosity) -> ThagResult<()> {
    LOGGER.lock()?.set_verbosity(verbosity);
    let v = get_verbosity();
    #[cfg(debug_assertions)]
    assert_eq!(v, verbosity);
    // Enable debug logging if -vv is passed
    if verbosity as u8 == Verbosity::Debug as u8 {
        enable_debug_logging(); // Set the runtime flag
    }

    Ok(())
}

// Configure log level
pub fn configure_log() {
    profile_fn!(configure_log);
    let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    let mut binding = Builder::new();
    let builder = binding.parse_env(env);
    builder.write_style(WriteStyle::Always);
    let _ = builder.try_init();

    // Builder::new().filter_level(log::LevelFilter::Debug).init();
}

#[macro_export]
macro_rules! log {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            $crate::logging::LOGGER.lock().unwrap().log($verbosity, &format!($($arg)*))
        }
    };
}
