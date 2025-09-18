//! Common types, macros, and utilities shared across `thag_rs` subcrates.
//!
//! This crate provides the foundational components that multiple `thag_rs` subcrates depend on,
//! including verbosity control, color support detection, terminal background luminance,
//! utility macros, and common error handling patterns.

#![warn(clippy::pedantic, missing_docs)]

/// Configuration management module
#[cfg(feature = "config")]
pub mod config;

/// Terminal detection and capabilities module
#[cfg(feature = "color_detect")]
pub mod terminal;

use documented::{Documented, DocumentedVariants};
use parking_lot::ReentrantMutex;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::LazyLock;
use std::{path::PathBuf, time::Instant};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "config")]
pub use crate::config::{ConfigError, ConfigResult};

/// Result type alias for `thag_common` operations
pub type ThagCommonResult<T> = Result<T, ThagCommonError>;

/// Error types for `thag_common` operations
#[derive(Debug, thiserror::Error)]
pub enum ThagCommonError {
    /// Error acquiring mutex lock
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

impl<T> From<std::sync::PoisonError<T>> for ThagCommonError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::LockError(err.to_string())
    }
}

/// Controls the detail level of user messaging
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Serialize,
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
#[serde(rename_all = "snake_case")]
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

/// Type alias for Verbosity to provide a shorter name for convenience
pub type V = Verbosity;

impl V {
    /// Shorthand for `Verbosity::Quieter`
    pub const QQ: Self = Self::Quieter;
    /// Shorthand for `Verbosity::Quiet`
    pub const Q: Self = Self::Quiet;
    /// Shorthand for `Verbosity::Normal`
    pub const N: Self = Self::Normal;
    /// Shorthand for `Verbosity::Verbose`
    pub const V: Self = Self::Verbose;
    /// Shorthand for `Verbosity::Debug`
    pub const VV: Self = Self::Debug;
    /// Shorthand for `Verbosity::Debug`
    pub const D: Self = Self::Debug;
}

/// An enum to categorise the current terminal's level of colour support as detected, configured
/// or defaulted.
///
/// We fold `TrueColor` into Xterm256 as we're not interested in more than 256
/// colours just for messages.
#[derive(
    Clone,
    Copy,
    Debug,
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
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ColorSupport {
    /// Still to be determined or defaulted
    Undetermined = 0,
    /// No color support
    None = 1,
    /// Basic 16-color support
    #[serde(alias = "ansi16")] // Accept old "ansi16" value
    Basic = 2,
    /// Full color support, suitable for color palettes of 256 colours (8 bit) or higher.
    #[serde(alias = "xterm256")] // Accept old "256" value
    Color256 = 3,
    /// Full color support, 24 bits -> 16 million colors.
    TrueColor = 4,
}

impl Default for ColorSupport {
    fn default() -> Self {
        Self::Basic // Safe default when detection isn't available
    }
}

/// Terminal background luminance detection and specification
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    Documented,
    DocumentedVariants,
    EnumIter,
    EnumString,
    Eq,
    Hash,
    IntoStaticStr,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TermBgLuma {
    /// Light background terminal
    Light,
    /// Dark background terminal
    Dark,
    /// Let `thag` autodetect the background luminosity
    Undetermined,
}

impl Default for TermBgLuma {
    fn default() -> Self {
        Self::Dark // Safe default when detection isn't available
    }
}

/// Manages user message output with verbosity control and thread-safe locking
#[derive(Debug)]
pub struct OutputManager {
    /// The current verbosity level for this output manager
    verbosity: std::cell::UnsafeCell<Verbosity>,
}

impl OutputManager {
    /// Construct a new `OutputManager` with the given Verbosity level.
    #[must_use]
    pub const fn new(verbosity: Verbosity) -> Self {
        Self {
            verbosity: std::cell::UnsafeCell::new(verbosity),
        }
    }

    /// Output a message whether or not it passes the verbosity filter.
    pub fn prtln(&self, message: &str) {
        println!("{message}");
    }

    /// Output a message if it passes the verbosity filter.
    pub fn vprtln(&self, verbosity: Verbosity, message: &str) {
        if verbosity as u8 <= self.verbosity() as u8 {
            println!("{message}");
        }
    }

    /// Set the verbosity level.
    /// Updates the verbosity setting and logs the change
    pub fn set_verbosity(&self, verbosity: Verbosity) {
        // SAFETY: We use UnsafeCell for interior mutability since we need to modify
        // verbosity from within a ReentrantMutex which only provides immutable access.
        // This is safe because the ReentrantMutex ensures exclusive access per thread.
        unsafe {
            *self.verbosity.get() = verbosity;
        }
        debug_log!("Verbosity set to {verbosity:?}");
    }

    /// Return the verbosity level
    #[allow(clippy::missing_const_for_fn)]
    pub fn verbosity(&self) -> Verbosity {
        // SAFETY: Reading from UnsafeCell is safe when we have any kind of lock
        unsafe { *self.verbosity.get() }
    }
}

/// Global output manager instance protected by a reentrant mutex for thread-safe access
pub static OUTPUT_MANAGER: LazyLock<ReentrantMutex<OutputManager>> =
    LazyLock::new(|| ReentrantMutex::new(OutputManager::new(V::N)));

/// Set the output verbosity for the current execution.
/// # Panics
/// Will panic in debug mode if the global verbosity value is not the value we just set.
pub fn set_global_verbosity(verbosity: Verbosity) {
    OUTPUT_MANAGER.lock().set_verbosity(verbosity);
    #[cfg(debug_assertions)]
    assert_eq!(get_verbosity(), verbosity);
}

/// Initializes and returns the global verbosity setting.
///
/// # Panics
///
/// Will panic if it can't unwrap the lock on the mutex protecting the `OUTPUT_MANAGER` static variable.
#[must_use]
pub fn get_verbosity() -> Verbosity {
    let verbosity = OUTPUT_MANAGER.lock().verbosity();
    verbosity
}

/// Ungated print line macro for user messages
#[macro_export]
macro_rules! prtln {
    ($($arg:tt)*) => {
        {
            $crate::OUTPUT_MANAGER.lock().prtln(&format!($($arg)*));
        }
    };
}

/// Verbosity-gated print line macro for user messages
#[macro_export]
macro_rules! vprtln {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            $crate::OUTPUT_MANAGER.lock().vprtln($verbosity, &format!($($arg)*));
        }
    };
}

/// Debugging logger.
///
/// Logs if the `debug_logging` feature is enabled.
/// Should only be used within `thag_rs` subcrates due to feature dependencies.
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        // If the `debug_logging` feature is enabled, always log
        #[cfg(feature = "debug_logging")]
        {
            log::debug!($($arg)*);
        }

        #[cfg(not(feature = "debug_logging"))]
        {
            // Avoid unused variable warnings in release mode if logging isn't enabled
            let _ = format_args!($($arg)*);
        }
    };
}

/// Lazy-static variable generator.
///
/// Syntax:
/// ```ignore
/// use thag_common::lazy_static_var;
/// let my_var = lazy_static_var!(<T>, expr<T>); // for static ref
/// // or
/// let my_var = lazy_static_var!(<T>, deref, expr<T>); // for Deref value (not guaranteed)
/// ```
///
/// NB: In order to avoid fighting the compiler, it is not recommended to make `my_var` uppercase.
#[macro_export]
macro_rules! lazy_static_var {
    ($type:ty, deref, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        *GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    ($type:ty, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
}

/// Lazy-static regular expression generator.
///
/// From burntsushi at `https://github.com/rust-lang/regex/issues/709`
/// Syntax:
/// ```ignore
/// use thag_common::re;
/// let re = re!(<string literal>);
/// ```
///
/// NB: In order to avoid fighting the compiler, it is not recommended to make `re` uppercase.
#[macro_export]
macro_rules! re {
    ($re:literal $(,)?) => {{
        use {regex::Regex, std::sync::OnceLock};

        static RE: OnceLock<Regex> = OnceLock::new();
        RE.get_or_init(|| Regex::new($re).unwrap())
    }};
}

/// Creates a lazily-initialized static variable.
///
/// This macro generates a wrapper type that provides thread-safe lazy initialization
/// of static variables. It's used internally by the profiling system for managing
/// global state.
///
/// # Examples
///
/// ```ignore
/// use thag_profiler::static_lazy;
///
/// static_lazy! {
///     GLOBAL_CONFIG: Option<String> = Some("default".to_string())
/// }
/// ```
#[macro_export]
macro_rules! static_lazy {
    ($name:ident: Option<$inner_type:ty> = $init:expr) => {
        #[doc = stringify!($name)]
        #[doc = r" struct generated by `static_lazy`"]
        #[cfg_attr(not(feature = "internal_docs"), doc(hidden))]
        pub struct $name;

        impl $name {
            /// A getter for the static value
            pub fn get() -> Option<&'static $inner_type> {
                static INSTANCE: std::sync::OnceLock<Option<$inner_type>> =
                    std::sync::OnceLock::new();
                INSTANCE.get_or_init(|| $init).as_ref()
            }

            /// An initializer for the static value
            #[allow(dead_code)]
            pub fn init() {
                let _ = Self::get();
            }
        }
    };

    ($name:ident: $type:ty = $init:expr) => {
        #[doc = stringify!($name)]
        #[doc = r" struct generated by `static_lazy`"]
        pub struct $name;

        impl $name {
            /// A getter for the static value
            #[allow(clippy::missing_panics_doc)]
            pub fn get() -> &'static $type {
                static INSTANCE: std::sync::OnceLock<$type> = std::sync::OnceLock::new();
                INSTANCE.get_or_init(|| $init)
            }

            /// An initializer for the static value
            #[allow(dead_code)]
            pub fn init() {
                let _ = Self::get();
            }
        }
    };
}

/// Convenient macro for setting global verbosity with short syntax.
///
/// # Examples
///
/// ```rust
/// use thag_common::{set_verbosity, V};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Set to verbose
///     set_verbosity!(verbose);
///
///     // Set to debug
///     set_verbosity!(debug);
///
///     // Set to quiet
///     set_verbosity!(quiet);
///
///     // Use the V constants directly
///     set_verbosity!(V::V);
///
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! set_verbosity {
    (verbose) => {
        $crate::set_global_verbosity($crate::V::V);
    };
    (debug) => {
        $crate::set_global_verbosity($crate::V::D);
    };
    (quiet) => {
        $crate::set_global_verbosity($crate::V::Q);
    };
    (quieter) => {
        $crate::set_global_verbosity($crate::V::QQ);
    };
    (normal) => {
        $crate::set_global_verbosity($crate::V::N);
    };
    ($level:expr) => {
        $crate::set_global_verbosity($level);
    };
}

/// Initialize verbosity with a convenient function that handles common patterns.
///
/// # Examples
///
/// ```rust
/// use thag_common::{init_verbosity, V};
///
/// // Initialize with verbose for debugging
/// init_verbosity(V::V).expect("Failed to set verbosity");
///
/// // Initialize with debug
/// init_verbosity(V::D).expect("Failed to set verbosity");
/// ```
///
/// # Errors
/// Returns an error if the global verbosity cannot be set.
pub fn init_verbosity(verbosity: Verbosity) -> ThagCommonResult<()> {
    set_global_verbosity(verbosity);
    Ok(())
}

/// Reassemble an Iterator of lines from the disentangle function to a string of text.
#[inline]
pub fn reassemble<'a>(map: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    map.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers to convert a string of raw text to readable lines.
#[inline]
#[must_use]
pub fn disentangle(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

/// Helper function to sort out the issues caused by Windows using the escape character as
/// the file separator.
#[must_use]
#[inline]
#[cfg(target_os = "windows")]
pub fn escape_path_for_windows(path_str: &str) -> String {
    path_str.replace('\\', "/")
}

/// No-op function for non-Windows platforms.
#[must_use]
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn escape_path_for_windows(path_str: &str) -> String {
    path_str.to_string()
}

/// Developer method to log method timings.
#[inline]
pub fn debug_timings(start: &Instant, process: &str) {
    let dur = start.elapsed();
    debug_log!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
}

/// Get the user's home directory as a `String`.
///
/// # Errors
///
/// This function will return an error if it can't resolve the user directories.
pub fn get_home_dir_string() -> ThagCommonResult<String> {
    let home_dir = &get_home_dir()?;
    Ok(home_dir.display().to_string())
}

/// Get the user's home directory as a `PathBuf`.
///
/// # Errors
///
/// This function will return an error if it can't resolve the user directories.
pub fn get_home_dir() -> ThagCommonResult<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| ThagCommonError::Generic("Can't resolve user home directory".to_string()))?;
    Ok(home_dir)
}

/// Formats a given positive integer with thousands separators (commas).
///
/// This function takes any unsigned integer type (`u8`, `u16`, `u32`, `u64`, `u128`, `usize`)
/// and returns a `String` representation where groups of three digits are separated by commas.
///
/// # Examples
///
/// ```
/// use thag_common::thousands;
/// assert_eq!(thousands(1234567u32), "1,234,567");
/// assert_eq!(thousands(9876u16), "9,876");
/// assert_eq!(thousands(42u8), "42");
/// assert_eq!(thousands(12345678901234567890u128), "12,345,678,901,234,567,890");
/// ```
///
/// # Panics
///
/// This function panics if `std::str::from_utf8()` fails,
/// which is highly unlikely since the input is always a valid ASCII digit string.
///
/// # Complexity
///
/// Runs in **O(d)** time complexity, where `d` is the number of digits in the input number.
///
/// # Note
///
/// If you need to format signed integers, you'll need a modified version
/// that correctly handles negative numbers.
pub fn thousands<T: Display>(n: T) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}
