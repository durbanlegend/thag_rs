//! # `thag_profiler`
//!
//! A performance profiling library for Rust applications.
//!
//! ## Features
//!
//! - `time_profiling`: Enable time-based performance profiling (default)
//! - `full_profiling`: Enable comprehensive profiling including time and memory usage
//!
//! ## Examples
//!
//! ```toml
//! # Time profiling only (default)
//! thag_profiler = { version = "0.1.0" }
//!
//! # Full profiling with memory tracking
//! thag_profiler = { version = "0.1.0", features = ["full_profiling"] }
//! ```
//!
//! ## Basic Usage
//!
//! ```rust
//! use thag_profiler::Profile;
//!
//! // Time profiling
//! {
//!     let _p = Profile::new("my_function");
//!     // Code to profile...
//! }
//!
//! // Memory profiling (requires `full_profiling` feature)
//! #[cfg(feature = "full_profiling")]
//! {
//!     let _p = Profile::new_memory("memory_intensive_function");
//!     // Code to profile memory usage...
//! }
//! ```
mod errors;
mod logging;

pub mod profiling;

#[cfg(feature = "full_profiling")]
mod okaoka;

#[cfg(feature = "full_profiling")]
mod task_allocator;

use std::fmt::Display;

// Re-exports
pub use {
    errors::{ProfileError, ProfileResult},
    logging::{flush_debug_log, DebugLogger},
    profiling::{
        get_global_profile_type, is_profiling_enabled, Profile, ProfileSection, ProfileType,
    },
    thag_proc_macros::{enable_profiling, profiled},
    // Only re-export what users need from task_allocator
};

#[cfg(test)]
mod feature_tests {
    use crate::profiling::is_profiling_enabled;

    #[test]
    fn test_profiling_feature_flag_behavior() {
        // This test verifies the behavior of profiling features

        #[cfg(feature = "time_profiling")]
        {
            // When compiled with the "time_profiling" feature but profiling is disabled at runtime,
            // is_profiling_enabled() should return false in test mode due to our special handling
            assert!(!is_profiling_enabled(),
                "With profiling feature enabled but disabled at runtime, is_profiling_enabled() should return false in test mode");

            // We can enable profiling and it should work
            // Force set the state directly rather than using enable_profiling which might have side effects
            crate::profiling::force_set_profiling_state(true);
            assert!(
                is_profiling_enabled(),
                "After enabling profiling, is_profiling_enabled() should return true"
            );

            // Clean up
            crate::profiling::force_set_profiling_state(false);
        }

        #[cfg(not(feature = "time_profiling"))]
        {
            // When compiled without the "time_profiling" feature, is_profiling_enabled() should always return false
            assert!(
                !is_profiling_enabled(),
                "Without profiling feature, is_profiling_enabled() should always return false"
            );
        }
    }
}

#[cfg(feature = "time_profiling")]
pub const PROFILING_ENABLED: bool = true;

#[cfg(not(feature = "time_profiling"))]
pub const PROFILING_ENABLED: bool = false;

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
#[doc(hidden)] // Makes it not appear in documentation
macro_rules! static_lazy {
    ($name:ident: Option<$inner_type:ty> = $init:expr) => {
        pub struct $name;

        impl $name {
            pub fn get() -> Option<&'static $inner_type> {
                static INSTANCE: std::sync::OnceLock<Option<$inner_type>> =
                    std::sync::OnceLock::new();
                INSTANCE.get_or_init(|| $init).as_ref()
            }

            #[allow(dead_code)]
            pub fn init() {
                let _ = Self::get();
            }
        }
    };

    ($name:ident: $type:ty = $init:expr) => {
        pub struct $name;

        impl $name {
            fn get() -> &'static $type {
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

/// Initialize the profiling system.
/// This should be called at the start of your program to set up profiling.
///
/// # Panics
///
/// This function panics if profiling cannot be enabled.
#[cfg(feature = "time_profiling")]
pub fn init_profiling() {
    #[cfg(feature = "full_profiling")]
    use task_allocator::run_with_system_alloc;

    use crate::profiling::{enable_profiling, ProfileType};

    // Determine profile type based on features
    #[cfg(feature = "full_profiling")]
    let profile_type = ProfileType::Both;

    #[cfg(not(feature = "full_profiling"))]
    let profile_type = ProfileType::Time;

    // Enable profiling
    #[cfg(not(feature = "full_profiling"))]
    {
        if profile_type == ProfileType::Memory {
            panic!("Memory profiling requested but `full_profiling` feature is not enabled");
        }
        enable_profiling(true, profile_type).expect("Failed to enable profiling");
    }

    #[cfg(feature = "full_profiling")]
    run_with_system_alloc(|| {
        enable_profiling(true, profile_type).expect("Failed to enable profiling");
        task_allocator::initialize_memory_profiling();
    });
}

/// Finalize profiling and write out data files.
/// This should be called at the end of your program.
///
/// # Panics
///
/// This function panics if profiling cannot be enabled.
#[cfg(feature = "time_profiling")]
pub fn finalize_profiling() {
    use crate::profiling::{enable_profiling, ProfileType};

    // Determine profile type based on features
    #[cfg(feature = "full_profiling")]
    let profile_type = ProfileType::Both;

    #[cfg(not(feature = "full_profiling"))]
    let profile_type = ProfileType::Time;

    // Disable profiling, which will finalize and write data
    enable_profiling(false, profile_type).expect("Failed to finalize profiling");

    #[cfg(feature = "full_profiling")]
    task_allocator::finalize_memory_profiling();

    flush_debug_log();
}

// Provide no-op versions when profiling is disabled
#[cfg(not(feature = "time_profiling"))]
pub fn init_profiling() {}

#[cfg(not(feature = "time_profiling"))]
pub fn finalize_profiling() {}
