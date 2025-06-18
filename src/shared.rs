#![allow(clippy::uninlined_format_args)]
use crate::{debug_log, ThagResult};
use std::fmt::Display;
use std::{path::PathBuf, time::Instant};
use thag_profiler::profiled;

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
            $crate::log::debug!($($arg)*);
        }

        // In all builds, log if runtime debug logging is enabled (e.g., via `-vv`)
        #[cfg(not(any(feature = "debug-logs", feature = "simplelog")))]
        {
            if $crate::logging::is_debug_logging_enabled() {
                $crate::log::debug!($($arg)*);
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
/// let re = regex!(<string literal>)
/// ```
///
/// NB: In order to avoid fighting the compiler, it is not recommended to make `re` uppercase.
#[macro_export]
macro_rules! regex {
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
