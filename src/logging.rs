#![allow(clippy::uninlined_format_args)]
use crate::{debug_log, profile, profile_method, ThagResult};
use documented::{Documented, DocumentedVariants};
use serde::Deserialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex,
};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "simplelog")]
use crate::vlog;
#[cfg(feature = "simplelog")]
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
#[cfg(feature = "simplelog")]
use std::fs::File;

#[cfg(not(feature = "simplelog"))] // This will use env_logger if simplelog is not active
use env_logger::{Builder, Env};

static DEBUG_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initializes and returns the global verbosity setting.
///
/// # Panics
///
/// Will panic if it can't unwrap the lock on the mutex protecting the `LOGGER` static variable.
#[must_use]
pub fn get_verbosity() -> Verbosity {
    profile!("get_verbosity");
    LOGGER.lock().unwrap().verbosity
}

#[allow(clippy::module_name_repetitions)]
pub fn enable_debug_logging() {
    profile!("enable_debug_logging");
    DEBUG_LOG_ENABLED.store(true, Ordering::SeqCst);
}

pub fn is_debug_logging_enabled() -> bool {
    profile!("is_debug_logging_enabled");
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
        profile_method!("log");
        if verbosity as u8 <= self.verbosity as u8 {
            println!("{}", message);
        }
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        profile_method!("set_verbosity");
        self.verbosity = verbosity;

        debug_log!("Verbosity set to {verbosity:?}");
    }

    /// Return the verbosity level
    pub fn verbosity(&mut self) -> Verbosity {
        profile_method!("verbosity");
        self.verbosity
    }
}

pub static LOGGER: LazyLock<Mutex<Logger>> = LazyLock::new(|| Mutex::new(Logger::new(V::N)));

/// Set the logging verbosity for the current execution.
/// # Errors
/// Will return `Err` if the logger mutex cannot be locked.
/// # Panics
/// Will panic in debug mode if the global verbosity value is not the value we just set.
pub fn set_global_verbosity(verbosity: Verbosity) -> ThagResult<()> {
    profile!("set_global_verbosity");
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
    use log::info;
    profile!("configure_log");

    let env = Env::new().filter("RUST_LOG");
    Builder::new().parse_env(env).init();
    info!("Initialized env_logger");
}

/// Configure log level
///
/// # Panics
///
/// Panics if it can't create the log file app.log in the current working directory.
#[cfg(feature = "simplelog")]
pub fn configure_log() {
    profile!("configure_log");

    configure_simplelog();
    // info!("Initialized simplelog");  // interferes with testing
    vlog!(V::V, "Initialized simplelog");
}

/// Configure log level
///
/// # Panics
///
/// Panics if it can't create the log file app.log in the current working directory.
#[cfg(feature = "simplelog")]
fn configure_simplelog() {
    profile!("configure_simplelog");
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

/// A line print macro that prints a styled and coloured message.
///
/// Format: `cprtln!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);`
#[macro_export]
macro_rules! cprtln {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
        let style: &nu_ansi_term::Style = $style;
        // Qualified form to avoid imports in calling code.
        let painted = style.paint(content);
        let verbosity = $crate::logging::get_verbosity();
        $crate::vlog!(verbosity, "{painted}");
    }};
}
/// Logs a message provided the verbosity value passed in is at least as great as the current
/// verbosity level.
///
/// The current (global) verbosity level can be thought of as a cutoff level. This cutoff level
/// is either specified by the user via `-v`, `-vv`, `-n`, `-q` or `-qq` or their long-form
/// equivalents, or failing that, by the user's configured `default_verbosity` setting, or
/// failing *that*, by the system default verbosity setting of `Normal`.
///
/// How this works may still seem counterintuitive depending on your intuitions, so here are
/// some examples:
///
/// E.g. `vlog!(V::Q), "Hairy Rotter and the Philosopher's Stone Axe")` is an instruction
/// to print at verbosity (V) settings down to and including `Quiet (Q)` level, so it will
/// log the output as long as the user specified or defaulted to a verbosity other than
/// `Quieter (QQ) (-qq)` for the current `thag` execution.
///
/// Conversely, specifying `vlog(V::V)` (or in long form, `vlog!(Verbosity::Verbose)`, is an
/// instruction to print at verbosities down to and including `Verbose (V)`, so it will only
/// log the output if the user specified or defaulted to verbosity `Verbose (V) (-v)` or
/// `Debug (VV) (-vv)` for the current `thag` execution.
#[macro_export]
macro_rules! vlog {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            $crate::logging::LOGGER.lock().unwrap().log($verbosity, &format!($($arg)*))
        }
    };
}
