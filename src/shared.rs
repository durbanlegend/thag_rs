#![allow(clippy::uninlined_format_args)]
use crate::profile;

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

/// Control debug logging
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
