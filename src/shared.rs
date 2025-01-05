#![allow(clippy::uninlined_format_args)]
use crate::{debug_log, profile, ThagResult};
use std::{path::PathBuf, time::Instant};

/// Reassemble an Iterator of lines from the disentangle function to a string of text.
#[inline]
pub fn reassemble<'a>(map: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    profile!("reassemble");
    map.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers to convert a string of raw text to readable lines.
#[inline]
#[must_use]
pub fn disentangle(text_wall: &str) -> String {
    profile!("disentangle");
    reassemble(text_wall.lines())
}

// Helper function to sort out the issues caused by Windows using the escape character as
// the file separator.
#[must_use]
#[inline]
#[cfg(target_os = "windows")]
pub fn escape_path_for_windows(path_str: &str) -> String {
    profile!("escape_path_for_windows");
    path_str.replace('\\', "/")
}

#[must_use]
#[cfg(not(target_os = "windows"))]
pub fn escape_path_for_windows(path_str: &str) -> String {
    profile!("escape_path_for_windows");
    path_str.to_string()
}

/// Developer method to log method timings.
#[inline]
pub fn debug_timings(start: &Instant, process: &str) {
    profile!("debug_timings");
    let dur = start.elapsed();
    debug_log!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
}

/// Debugging logger. Logs if the `debug-logs` feature is enabled or if runtime debug logging is enabled (e.g., via `-vv`)
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
/// let my_var = lazy_static_var!(<T>, expr<T>, deref) // for Deref value (not guaranteed)
/// ```
///
/// NB: In order to avoid fighting the compiler, it is not recommended to make `my_var` uppercase.
#[macro_export]
macro_rules! lazy_static_var {
    ($type:ty, $init_fn:expr, deref) => {{
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

/// Get the user's home directory as a `String`.
///
/// # Errors
///
/// This function will return an error if it can't resolve the user directories.
pub fn get_home_dir_string() -> ThagResult<String> {
    let home_dir = &get_home_dir()?;
    Ok(home_dir.display().to_string())
}

/// Get the user's home directory as a `PathBuf`.
///
/// # Errors
///
/// This function will return an error if it can't resolve the user directories.
pub fn get_home_dir() -> ThagResult<PathBuf> {
    let user_dirs = directories::UserDirs::new().ok_or("Can't resolve user directories")?;
    let home_dir = user_dirs.home_dir();
    Ok(home_dir.to_owned())
}
