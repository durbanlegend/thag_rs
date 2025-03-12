//! # `thag_profiler`
//!
//! A Rust profiling library for measuring and analyzing code performance.
//!
//! ## Basic Usage
//!
//! ```
//! use thag_profiler::{profile, profiled};
//!
//! #[profiled]
//! fn my_function() {
//!     // Function code
//!
//!     let section = profile_section!("expensive part");
//!     // Expensive operation
//!     section.end();
//! }
//! ```

mod errors;
pub mod profiling;

use std::fmt::Display;

// Re-exports
pub use {
    errors::{ProfileError, ProfileResult},
    profiling::{get_global_profile_type, Profile, ProfileSection, ProfileType},
    thag_proc_macros::{enable_profiling, profiled},
};

#[cfg(feature = "profiling")]
pub const PROFILING_ENABLED: bool = true;

#[cfg(not(feature = "profiling"))]
pub const PROFILING_ENABLED: bool = false;

#[macro_export]
#[doc(hidden)] // Makes it not appear in documentation
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

/// Formats a given positive integer with thousands separators (commas).
///
/// This function takes any unsigned integer type (`u8`, `u16`, `u32`, `u64`, `u128`, `usize`)
/// and returns a `String` representation where groups of three digits are separated by commas.
///
/// # Examples
///
/// ```
/// use thag_profiler::thousands;
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
