#![allow(clippy::uninlined_format_args)]
use documented::{Documented, DocumentedVariants};
#[cfg(not(feature = "simplelog"))] // This will use env_logger if simplelog is not active
use env_logger::{Builder, Env};
use firestorm::{profile_fn, profile_method};
use serde::Deserialize;
#[cfg(feature = "simplelog")]
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
#[cfg(feature = "simplelog")]
use std::fs::File;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex,
};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::{config::maybe_config, debug_log, vlog, Cli, ThagResult};

static DEBUG_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initializes and returns the global verbosity setting.
///
/// # Panics
///
/// Will panic if it can't unwrap the lock on the mutex protecting the `LOGGER` static variable.
#[must_use]
pub fn get_verbosity() -> Verbosity {
    profile_fn!(get_verbosity);
    LOGGER.lock().unwrap().verbosity
}

#[allow(clippy::module_name_repetitions)]
pub fn enable_debug_logging() {
    profile_fn!(enable_debug_logging);
    DEBUG_LOG_ENABLED.store(true, Ordering::SeqCst);
}

pub fn is_debug_logging_enabled() -> bool {
    profile_fn!(is_debug_logging_enabled);
    DEBUG_LOG_ENABLED.load(Ordering::SeqCst)
}

/// Controls the detail level of logging messages
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Documented,
    DocumentedVariants,
    EnumIter,
    EnumString,
    IntoStaticStr,
    PartialEq,
    PartialOrd,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Verbosity {
    /// Minimal output, suitable for piping to another process
    Quieter = 0,
    /// Less detailed output
    Quiet = 1,
    /// Standard output level
    #[default]
    Normal = 2,
    /// More detailed output
    Verbose = 3,
    /// Maximum detail for debugging
    Debug = 4,
}

pub type V = Verbosity;

impl V {
    pub const QQ: Self = Self::Quieter;
    pub const Q: Self = Self::Quiet;
    pub const N: Self = Self::Normal;
    pub const V: Self = Self::Verbose;
    pub const VV: Self = Self::Debug;
    pub const D: Self = Self::Debug;
}

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
        profile_method!(log);
        if verbosity as u8 <= self.verbosity as u8 {
            println!("{}", message);
        }
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        profile_method!(set_verbosity);
        self.verbosity = verbosity;

        debug_log!("Verbosity set to {verbosity:?}");
    }

    /// Return the verbosity level
    pub fn verbosity(&mut self) -> Verbosity {
        profile_method!(verbosity);
        self.verbosity
    }
}

pub static LOGGER: LazyLock<Mutex<Logger>> = LazyLock::new(|| Mutex::new(Logger::new(V::N)));

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
        V::Quiet
    } else if args.quiet >= 2 {
        V::Quieter
    } else if args.normal {
        V::Normal
    } else if let Some(config) = maybe_config() {
        config.logging.default_verbosity
    } else {
        V::Normal
    };
    set_global_verbosity(verbosity)
}

/// Set the logging verbosity for the current execution.
/// # Errors
/// Will return `Err` if the logger mutex cannot be locked.
/// # Panics
/// Will panic in debug mode if the global verbosity value is not the value we just set.
pub fn set_global_verbosity(verbosity: Verbosity) -> ThagResult<()> {
    profile_fn!(set_global_verbosity);
    LOGGER.lock()?.set_verbosity(verbosity);
    #[cfg(debug_assertions)]
    assert_eq!(get_verbosity(), verbosity);
    // Enable debug logging if -vv is passed
    if verbosity as u8 == Verbosity::Debug as u8 {
        enable_debug_logging(); // Set the runtime flag
    }

    Ok(())
}

/// Configure log level
#[cfg(feature = "env_logger")]
pub fn configure_log() {
    profile_fn!(configure_log);

    let env = Env::new().filter("RUST_LOG");
    Builder::new().parse_env(env).init();
    info!("Initialized env_logger");
}

/// Configure log level
///
/// # Panics
///
/// Panics if it can't create the log file app.log in the current working directory.
#[cfg(not(feature = "env_logger"))]
pub fn configure_log() {
    profile_fn!(configure_log);

    configure_simplelog();
    // info!("Initialized simplelog");  // interferes with testing
    vlog!(V::V, "Initialized simplelog");
}

/// Configure log level
///
/// # Panics
///
/// Panics if it can't create the log file app.log in the current working directory.
#[cfg(not(feature = "env_logger"))]
fn configure_simplelog() {
    profile_fn!(configure_simplelog);
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("app.log").unwrap(),
        ),
    ])
    .unwrap();
}

#[macro_export]
macro_rules! vlog {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            $crate::logging::LOGGER.lock().unwrap().log($verbosity, &format!($($arg)*))
        }
    };
}
