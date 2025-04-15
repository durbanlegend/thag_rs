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
//! {
//!     let _p = Profile::new(Some("my_function"), None, ProfileType::Time, false, false, false);
//!     // Code to profile...
//! }
//!
//! // Memory profiling (requires `full_profiling` feature)
//! #[cfg(feature = "full_profiling")]
//! {
//!     let _p = Profile::new(Some("memory_intensive_function"), None, ProfileType::Memory, false, false, true);
//!     // Code to profile memory usage...
//! }
//! ```
mod errors;
mod logging;

pub mod profiling;

#[cfg(feature = "full_profiling")]
mod task_allocator;

#[cfg(feature = "full_profiling")]
mod mem_alloc;

use std::fmt::Display;

#[cfg(feature = "time_profiling")]
use backtrace::{Backtrace, BacktraceFrame};

// #[cfg(feature = "time_profiling")]
// pub use crate::profiling::{disable_profiling, enable_profiling};

#[cfg(feature = "time_profiling")]
use std::sync::OnceLock;

// Re-exports
pub use {
    errors::{ProfileError, ProfileResult},
    logging::{flush_debug_log, get_debug_log_path, DebugLogger},
    profiling::{
        disable_profiling, enable_profiling, get_config_profile_type, get_global_profile_type,
        is_detailed_memory, is_profiling_enabled, strip_hex_suffix, Profile, ProfileSection,
        ProfileType,
    },
    thag_proc_macros::fn_name,
    // Only re-export what users need from task_allocator
};

#[cfg(feature = "full_profiling")]
pub use {
    mem_alloc::{find_profile, record_allocation, register_profile, ProfileRef, PROFILE_REGISTRY},
    profiling::extract_path,
    task_allocator::{
        create_memory_task, find_matching_task_id, get_last_active_task, get_task_memory_usage,
        trim_backtrace, with_allocator, Allocator, Dispatcher, TaskAwareAllocator, TaskGuard,
        TaskMemoryContext, ALLOC_REGISTRY,
    },
};

#[cfg(feature = "time_profiling")]
pub use thag_proc_macros::{enable_profiling, profiled};

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
pub fn init_profiling(root_module: &'static str, maybe_profile_type: Option<ProfileType>) {
    PROFILEE.set(Profilee::new(root_module)).unwrap();

    set_base_location(fn_name);
    enable_profiling(true, maybe_profile_type).expect("Failed to enable profiling");
}

/// Initialize the profiling system.
/// This should be called at the start of your program to set up profiling.
///
/// # Panics
///
/// This function panics if profiling cannot be enabled.
#[cfg(feature = "full_profiling")]
#[fn_name]
pub fn init_profiling(root_module: &'static str, maybe_profile_type: Option<ProfileType>) {
    with_allocator(Allocator::System, || {
        PROFILEE.set(Profilee::new(root_module)).unwrap();

        set_base_location(fn_name);
        enable_profiling(true, maybe_profile_type).expect("Failed to enable profiling");

        let global_profile_type = get_global_profile_type();

        debug_log!(
            "In init_profiling with global_profile_type={:?}",
            global_profile_type
        );

        if global_profile_type == ProfileType::Time {
            debug_log!(
                "Skipping memory profiling initialization because global_profile_type={:?}",
                global_profile_type
            );
        } else {
            debug_log!("Initializing memory profiling");
            task_allocator::initialize_memory_profiling();
        }
    });
}

// Provide no-op versions when profiling is disabled
#[cfg(not(feature = "time_profiling"))]
pub const fn init_profiling(_root_module: &str, _maybe_profile_type: Option<ProfileType>) {}

#[cfg(feature = "time_profiling")]
fn set_base_location(fn_name: &str) {
    // eprintln!("module_path!()={}", module_path!());
    // TODO replace by function_name attribute macro
    let this_function = format!("{}::{fn_name}", module_path!());
    // eprintln!("this_function={this_function}");
    let base_location = Box::leak(
        Backtrace::frames(&Backtrace::new())
            .iter()
            .flat_map(BacktraceFrame::symbols)
            .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
            .skip_while(|frame| {
                !(frame.contains(&this_function)
                    && strip_hex_suffix(frame.to_string()) == this_function)
            })
            .take(1)
            .last()
            .unwrap()
            .into_boxed_str(),
    );
    PROFILER.set(Profiler::new(base_location)).unwrap();
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
/// This function panics if profiling cannot be enabled.
#[cfg(feature = "full_profiling")]
pub fn finalize_profiling() {
    with_allocator(Allocator::System, || {
        // Ensure debug log is flushed before we disable profiling
        flush_debug_log();

        let global_profile_type = get_global_profile_type();

        // Disable profiling
        enable_profiling(false, None).expect("Failed to finalize profiling");

        if global_profile_type != ProfileType::Time {
            task_allocator::finalize_memory_profiling();
        }

        // Final flush to ensure all data is written
        flush_debug_log();

        // Add a delay to ensure flush completes before program exit
        std::thread::sleep(std::time::Duration::from_millis(10));
    });
}

#[cfg(not(feature = "time_profiling"))]
pub const fn finalize_profiling() {}
