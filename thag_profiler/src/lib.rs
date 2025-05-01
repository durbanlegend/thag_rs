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
//! use thag_profiler::{Profile, ProfileType};
//!
//! // Time profiling
//! #[cfg(feature = "time_profiling")]
//! {
//!     let _p = Profile::new(Some("my_function"), None, ProfileType::Time, false, false, file!(), None, None);
//!     // Code to profile...
//! }
//!
//! // Memory profiling (requires `full_profiling` feature)
//! #[cfg(feature = "full_profiling")]
//! {
//!     let _p = Profile::new(Some("memory_intensive_function"), None, ProfileType::Memory, false, false, file!(), None, None);
//!     // Code to profile memory usage...
//! }
//! ```
mod errors;
mod logging;

pub mod profiling;

#[cfg(feature = "full_profiling")]
pub mod mem_tracking;

#[cfg(feature = "full_profiling")]
pub mod mem_attribution;

use std::{fmt::Display, path::Path};

#[cfg(feature = "time_profiling")]
use std::sync::OnceLock;

// Re-exports
pub use {
    errors::{ProfileError, ProfileResult},
    logging::{flush_debug_log, get_debug_log_path, DebugLogger},
    profiling::{
        disable_profiling, enable_profiling, get_config_profile_type, get_global_profile_type,
        is_detailed_memory, is_profiling_enabled, parse_env_profile_config, strip_hex_suffix,
        Profile, /* ProfileSection,*/
        ProfileConfiguration, ProfileType,
    },
    thag_proc_macros::fn_name,
    // Only re-export what users need from mem_tracking
};

pub use paste; // Re-export paste crate

#[cfg(feature = "full_profiling")]
pub use {
    mem_attribution::{find_profile, register_profile, ProfileRef, PROFILE_REGISTRY},
    mem_tracking::{
        create_memory_task, find_matching_task_id, get_last_active_task, get_task_memory_usage,
        record_allocation, trim_backtrace, with_allocator, Allocator, Dispatcher,
        TaskAwareAllocator, TaskGuard, TaskMemoryContext, ALLOC_REGISTRY,
    },
    profiling::extract_path,
};

// #[cfg(feature = "time_profiling")]
pub use thag_proc_macros::{enable_profiling, end, profile, profiled};

#[cfg(feature = "time_profiling")]
pub use profiling::PROFILING_MUTEX;

#[cfg(feature = "time_profiling")]
pub static PROFILER: OnceLock<Profiler> = OnceLock::new();

#[cfg(feature = "time_profiling")]
#[derive(Debug)]
pub struct Profiler {
    base_location: &'static str,
}

#[cfg(feature = "time_profiling")]
impl Profiler {
    const fn new(base_location: &'static str) -> Self {
        Self { base_location }
    }
}

#[cfg(feature = "time_profiling")]
pub fn get_profiler() -> Option<&'static Profiler> {
    PROFILER.get()
}

#[cfg(feature = "time_profiling")]
pub fn get_base_location() -> Option<&'static str> {
    PROFILER.get().map(|profiler| profiler.base_location)
}

#[cfg(feature = "time_profiling")]
pub static PROFILEE: OnceLock<Profilee> = OnceLock::new();

#[cfg(feature = "time_profiling")]
#[derive(Debug)]
pub struct Profilee {
    root_module: &'static str,
}

#[cfg(feature = "time_profiling")]
impl Profilee {
    const fn new(root_module: &'static str) -> Self {
        Self { root_module }
    }
}

#[cfg(feature = "time_profiling")]
pub fn get_profilee() -> Option<&'static Profilee> {
    PROFILEE.get()
}

#[cfg(feature = "time_profiling")]
pub fn get_root_module() -> Option<&'static str> {
    PROFILEE.get().map(|profilee| profilee.root_module)
}

#[must_use]
pub fn file_stem_from_path_str(file_name: &'static str) -> String {
    file_stem_from_path(Path::new(file_name))
}

/// Extract the file stem from a Path.
///
/// # Panics
///
/// Panics if `Path::file_stem()`    does not return a valid file stem.
#[must_use]
pub fn file_stem_from_path(path: &Path) -> String {
    path.file_stem().unwrap().to_string_lossy().to_string()
}

#[cfg(feature = "time_profiling")]
pub const PROFILING_FEATURE_ENABLED: bool = true;

#[cfg(not(feature = "time_profiling"))]
pub const PROFILING_FEATURE_ENABLED: bool = false;

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
            #[allow(clippy::missing_panics_doc)]
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

/// Initialize the profiling system.
/// This should be called at the start of your program to set up profiling.
///
/// # Panics
///
/// This function panics if profiling cannot be enabled.
#[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
#[fn_name]
pub fn init_profiling(root_module: &'static str, profile_config: ProfileConfiguration) {
    // Only set PROFILEE if it hasn't been set already
    // This allows multiple test functions to call init_profiling
    if PROFILEE.get().is_none() {
        PROFILEE.set(Profilee::new(root_module)).unwrap();
    } else if PROFILEE.get().unwrap().root_module != root_module {
        // If already set but with a different root_module, just log it and continue
        eprintln!(
            "Warning: PROFILEE already set with root_module={}, not changing to {}",
            PROFILEE.get().unwrap().root_module,
            root_module
        );
    }

    set_base_location(file!(), fn_name, line!());
    enable_profiling(true, profile_config.profile_type()).expect("Failed to enable profiling");
    eprintln!("Exiting init_profiling");
}

/// Initialize the profiling system.
/// This should be called at the start of your program to set up profiling.
///
/// # Panics
///
/// This function panics if profiling cannot be enabled.
#[cfg(feature = "full_profiling")]
#[fn_name]
pub fn init_profiling(root_module: &'static str, profile_config: ProfileConfiguration) {
    with_allocator(Allocator::System, || {
        eprintln!("root_module={root_module}, profile_config={profile_config:#?}");

        // Only set PROFILEE if it hasn't been set already
        // This allows multiple test functions to call init_profiling
        if PROFILEE.get().is_none() {
            PROFILEE.set(Profilee::new(root_module)).unwrap();
        } else if PROFILEE.get().unwrap().root_module != root_module {
            // If already set but with a different root_module, just log it and continue
            eprintln!(
                "Warning: PROFILEE already set with root_module={}, not changing to {}",
                PROFILEE.get().unwrap().root_module,
                root_module
            );
        }

        set_base_location(file!(), fn_name, line!());

        enable_profiling(true, profile_config.profile_type()).expect("Failed to enable profiling");

        let global_profile_type = get_global_profile_type();

        eprintln!(
            "In init_profiling with global_profile_type={:?}",
            global_profile_type
        );

        if global_profile_type == ProfileType::Time {
            eprintln!(
                "Skipping memory profiling initialization because global_profile_type={:?}",
                global_profile_type
            );
        } else {
            eprintln!("Initializing memory profiling");
            mem_tracking::initialize_memory_profiling();
        }
    });
    eprintln!("Exiting init_profiling");
}

// Provide no-op versions when profiling is disabled
#[cfg(not(feature = "time_profiling"))]
pub fn init_profiling(_root_module: &str, _profile_config: ProfileConfiguration) {}

#[cfg(feature = "time_profiling")]
fn set_base_location(file_name: &'static str, fn_name: &str, _line_no: u32) {
    let base_loc = format!("{file_name}::{fn_name}");
    let base_location = Box::leak(base_loc.into_boxed_str());

    // Only set PROFILER if it hasn't been set already
    // This allows multiple test functions to call set_base_location
    if PROFILER.get().is_none() {
        PROFILER.set(Profiler::new(base_location)).unwrap();
    } else if PROFILER.get().unwrap().base_location != base_location {
        // If already set but with a different base_location, just log it and continue
        eprintln!(
            "Warning: PROFILER already set with base_location={}, not changing to {}",
            PROFILER.get().unwrap().base_location,
            base_location
        );
    }
    // eprintln!("base_location={base_location}");
}

/// Finalize profiling and write out data files.
/// This should be called at the end of your program.
///
/// # Panics
///
/// This function panics if profiling cannot be enabled.
#[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
pub fn finalize_profiling() {
    // Ensure debug log is flushed before we disable profiling

    flush_debug_log();

    // Disable profiling
    enable_profiling(false, None).expect("Failed to finalize profiling");

    // Determine profile type based on features
    // let global_profile_type = get_global_profile_type();

    // Final flush to ensure all data is written
    flush_debug_log();

    // Add a delay to ensure flush completes before program exit
    std::thread::sleep(std::time::Duration::from_millis(10));
}

/// Finalize profiling and write out data files.
/// This should be called at the end of your program.
///
/// # Panics
///
/// This function panics if profiling cannot be disabled.
#[cfg(feature = "full_profiling")]
pub fn finalize_profiling() {
    with_allocator(Allocator::System, || {
        // Ensure debug log is flushed before we disable profiling
        flush_debug_log();

        let global_profile_type = get_global_profile_type();

        // Disable profiling
        enable_profiling(false, None).expect("Failed to finalize profiling");

        if global_profile_type != ProfileType::Time {
            mem_tracking::finalize_memory_profiling();
        }

        // Final flush to ensure all data is written
        flush_debug_log();

        // Add a delay to ensure flush completes before program exit
        std::thread::sleep(std::time::Duration::from_millis(10));
    });
}

#[cfg(not(feature = "time_profiling"))]
pub const fn finalize_profiling() {}

// /// Resets profiling configuration state for tests.
// ///
// /// This function should be used at the beginning of tests that need to control
// /// profiling configuration. It ensures that the profiling system reads the
// /// latest environment variables rather than using cached configurations.
// #[cfg(feature = "time_profiling")]
// pub fn reset_profiling_config_for_tests() {
//     // We need different implementation paths for unit tests vs integration tests
//     #[cfg(test)]
//     {
//         // Unit tests can directly call the internal function
//         profiling::reset_profile_config_for_tests();
//     }

//     // For integration tests which are compiled as separate crates and
//     // can't access the internal implementation
//     #[cfg(not(test))]
//     {
//         // Implement reset logic directly here for integration tests
//         use std::env;
//         use std::sync::atomic::Ordering;

//         eprintln!("Integration test: Resetting profile configuration from environment variables");

//         // First, parse the environment configuration
//         let env_var = env::var("THAG_PROFILE").ok();
//         let profile_type = if let Some(env_var) = env_var {
//             let parts: Vec<&str> = env_var.split(',').collect();
//             if !parts.is_empty() && !parts[0].trim().is_empty() {
//                 match parts[0].trim() {
//                     "time" => Some(profiling::ProfileType::Time),
//                     "memory" => Some(profiling::ProfileType::Memory),
//                     "both" => Some(profiling::ProfileType::Both),
//                     _ => None,
//                 }
//             } else {
//                 None
//             }
//         } else {
//             None
//         };

//         // Update the global profile type to match
//         if let Some(profile_type) = profile_type {
//             let value = profiling::ProfileCapability::from_profile_type(profile_type).0;
//             profiling::GLOBAL_PROFILE_TYPE.store(value, Ordering::SeqCst);
//             eprintln!("Set global profile type to {:?}", profile_type);
//         }
//     }
// }

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

#[cfg(test)]
mod config_tests {
    use super::*;
    use std::env;

    /// Tests that resetting profile config picks up environment variable changes
    /// This test is isolated to avoid interfering with other tests
    #[test]
    fn test_profile_config_picks_up_env_changes() {
        // Save original env var if it exists
        let original = env::var("THAG_PROFILE").ok();

        // First set to "time"
        env::set_var("THAG_PROFILE", "time,.,none,false");

        // Clear the cache to force reloading from environment
        profiling::clear_profile_config_cache();

        // Check that it's set to Time
        assert_eq!(
            profiling::get_config_profile_type(),
            profiling::ProfileType::Time
        );
        assert_eq!(get_config_profile_type(), profiling::ProfileType::Time);

        // Verify that the global type was also updated
        assert_eq!(
            profiling::get_global_profile_type(),
            profiling::ProfileType::Time
        );

        // Now change to "both"
        env::set_var("THAG_PROFILE", "both,.,none,false");

        // Clear the cache again to force reloading
        profiling::clear_profile_config_cache();

        // Check that it picked up the change
        assert_eq!(
            profiling::get_config_profile_type(),
            profiling::ProfileType::Both
        );
        assert_eq!(get_config_profile_type(), profiling::ProfileType::Both);

        // Verify that the global type was also updated
        assert_eq!(
            profiling::get_global_profile_type(),
            profiling::ProfileType::Both
        );

        // Restore original env var or remove it
        if let Some(val) = original {
            env::set_var("THAG_PROFILE", val);
        } else {
            env::remove_var("THAG_PROFILE");
        }

        // Clear the cache once more to restore state
        profiling::clear_profile_config_cache();
    }
}
