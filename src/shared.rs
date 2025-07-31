#![allow(clippy::uninlined_format_args)]
use crate::{debug_log, ThagResult};
use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::{LazyLock, Mutex};
use std::{path::PathBuf, time::Instant};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use thag_profiler::profiled;

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
        #[cfg(feature = "color_detect")]
        {
            Self::Undetermined
        }

        #[cfg(not(feature = "color_detect"))]
        {
            Self::Basic // Safe default when detection isn't available
        }
    }
}

/// Manages user message output with verbosity control and thread-safe locking
#[derive(Debug)]
pub struct OutputManager {
    /// The current verbosity level for this output manager
    pub verbosity: Verbosity,
}

impl OutputManager {
    /// Construct a new `OutputManager` with the given Verbosity level.
    #[must_use]
    pub const fn new(verbosity: Verbosity) -> Self {
        Self { verbosity }
    }

    /// Output a message if it passes the verbosity filter.
    #[profiled]
    pub fn prtln(&self, verbosity: Verbosity, message: &str) {
        eprintln!("verbosity={verbosity}, self.verbosity={}", self.verbosity);
        if verbosity as u8 <= self.verbosity as u8 {
            println!("{}", message);
        }
    }

    /// Set the verbosity level.
    #[profiled]
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
        debug_log!("Verbosity set to {verbosity:?}");
    }

    /// Return the verbosity level
    #[allow(clippy::missing_const_for_fn)]
    #[profiled]
    pub fn verbosity(&mut self) -> Verbosity {
        self.verbosity
    }
}

/// Global output manager instance protected by a mutex for thread-safe access
pub static OUTPUT_MANAGER: LazyLock<Mutex<OutputManager>> =
    LazyLock::new(|| Mutex::new(OutputManager::new(V::N)));

/// Set the output verbosity for the current execution.
/// # Errors
/// Will return `Err` if the output manager mutex cannot be locked.
/// # Panics
/// Will panic in debug mode if the global verbosity value is not the value we just set.
#[profiled]
pub fn set_global_verbosity(verbosity: Verbosity) -> ThagResult<()> {
    OUTPUT_MANAGER.lock()?.set_verbosity(verbosity);
    #[cfg(debug_assertions)]
    assert_eq!(get_verbosity(), verbosity);
    // Enable debug logging if -vv is passed
    if verbosity as u8 == Verbosity::Debug as u8 {
        crate::logging::enable_debug_logging(); // Set the runtime flag
    }

    Ok(())
}

/// Initializes and returns the global verbosity setting.
///
/// # Panics
///
/// Will panic if it can't unwrap the lock on the mutex protecting the `OUTPUT_MANAGER` static variable.
#[must_use]
#[profiled]
pub fn get_verbosity() -> Verbosity {
    OUTPUT_MANAGER.lock().unwrap().verbosity
}

/// A line print macro that prints a styled and coloured message.
///
/// Format: `cprtln!(style: Style, "Lorem ipsum dolor {} amet", content: &str);`
#[macro_export]
macro_rules! cprtln {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
        let painted = $style.paint(content);
        let verbosity = $crate::shared::get_verbosity();
        $crate::vprtln!(verbosity, "{painted}");
    }};
}

/// Verbosity-gated print line macro for user messages
#[macro_export]
macro_rules! vprtln {
    ($verbosity:expr, $($arg:tt)*) => {
        {
            $crate::shared::OUTPUT_MANAGER.lock().unwrap().prtln($verbosity, &format!($($arg)*))
        }
    };
}

/// Reassemble an Iterator of lines from the disentangle function to a string of text.
#[inline]
#[profiled]
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
#[profiled]
pub fn disentangle(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

/// Helper function to sort out the issues caused by Windows using the escape character as
/// the file separator.
#[must_use]
#[inline]
#[cfg(target_os = "windows")]
#[profiled]
pub fn escape_path_for_windows(path_str: &str) -> String {
    path_str.replace('\\', "/")
}

/// No-op function for non-Windows platforms.
#[must_use]
#[cfg(not(target_os = "windows"))]
#[inline]
#[profiled]
pub fn escape_path_for_windows(path_str: &str) -> String {
    path_str.to_string()
}

/// Developer method to log method timings.
#[inline]
#[profiled]
pub fn debug_timings(start: &Instant, process: &str) {
    let dur = start.elapsed();
    debug_log!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
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
    #[profiled]
    fn default() -> Self {
        #[cfg(feature = "color_detect")]
        {
            Self::Undetermined
        }

        #[cfg(not(feature = "color_detect"))]
        {
            Self::Dark // Safe default when detection isn't available
        }
    }
}

/// Debugging logger.
///
/// Logs if the `debug-logs` feature is enabled or if runtime debug logging is enabled (e.g., via `-vv`).
/// Should not be used outside the `thag_rs` crat due to the feature dependencies. Note that per Rust, messages,
/// "using a cfg inside a macro will use the cfgs from the destination crate and not the ones from the defining crate".
///
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        // If the `debug-logs` feature is enabled, always log
        #[cfg(any(feature = "debug-logs", feature = "simplelog"))]
        {
            log::debug!($($arg)*);
        }

        // In all builds, log if runtime debug logging is enabled (e.g., via `-vv`)
        #[cfg(not(any(feature = "debug-logs", feature = "simplelog")))]
        {
            if $crate::logging::is_debug_logging_enabled() {
                log::debug!($($arg)*);
            } else {
                // Avoid unused variable warnings in release mode if logging isn't enabled
                let _ = format_args!($($arg)*);
            }
        }
    };
}

/// Lazy-static variable generator.
///
/// Syntax:
/// ```Rust
/// let my_var = lazy_static_var!(<T>, expr<T>) // for static ref
/// // or
/// let my_var = lazy_static_var!(<T>, deref, expr<T>) // for Deref value (not guaranteed)
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
/// ```Rust
/// let re = re!(<string literal>)
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

#[macro_export]
/// Lazy-static variable generator with struct-based interface.
///
/// Creates a struct that provides lazy initialization of a static value.
/// The struct provides `get()` method to access the value and `init()` method
/// to force initialization.
///
/// # Examples
///
/// ```rust
/// use thag_rs::static_lazy;
///
/// static_lazy!(MY_CONFIG: String = "default_config".to_string());
///
/// // Access the value
/// let config = MY_CONFIG::get();
///
/// // Force initialization
/// MY_CONFIG::init();
/// ```
macro_rules! static_lazy {
    ($name:ident: $type:ty = $init:expr) => {
        struct $name;

        impl $name {
            pub fn get() -> &'static $type {
                static INSTANCE: std::sync::OnceLock<$type> = std::sync::OnceLock::new();
                INSTANCE.get_or_init(|| $init)
            }

            #[allow(dead_code)]
            pub fn init() {
                let _ = Self::get();
            }
        }
    };
}

/// Get the user's home directory as a `String`.
///
/// # Errors
///
/// This function will return an error if it can't resolve the user directories.
#[profiled]
pub fn get_home_dir_string() -> ThagResult<String> {
    let home_dir = &get_home_dir()?;
    Ok(home_dir.display().to_string())
}

/// Get the user's home directory as a `PathBuf`.
///
/// # Errors
///
/// This function will return an error if it can't resolve the user directories.
#[profiled]
pub fn get_home_dir() -> ThagResult<PathBuf> {
    let home_dir = dirs::home_dir().ok_or("Can't resolve user home directory")?;
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
/// use thag_rs::thousands;
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
#[profiled]
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
