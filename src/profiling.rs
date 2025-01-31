#![allow(clippy::module_name_repetitions)]
//!
//! Basic profiling for `thag_rs`, also (TODO) intended as an option for user scripts.
//!
//! Placing the following instruction at the start of your code will allow profiling to be enabled by running with `--features=profiling`.
//!
//! ```ignore
//!     if cfg!(feature = "profiling") {
//!         println!("Enabling profiling..."); // Debug output
//!         profiling::enable_profiling(true)?;
//!     }
//! ```
//!
//! Output will be in the form of a file called `thag-profile.folded` in the current working directory, and may be
//! displayed as statistics or as an `inferno` [flamechart](https://medium.com/performance-engineering-for-the-ordinary-barbie/profiling-flame-chart-vs-flame-graph-7b212ddf3a83)
//! using the `demo/thag_profile.rs` script, which you may wish to compile to a command first by using the `-x` option.
//!
//! `demo/thag_profile.rs` also allows you to filter out unwanted events, e.g. to make it easier to drill down into the flamechart.
//!
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Result, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, SystemTime};
pub use thag_proc_macros::profile;

use crate::{ThagError, ThagResult};

static FIRST_WRITE: AtomicBool = AtomicBool::new(true);

thread_local! {
    static PROFILE_STACK: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

static PROFILING_ENABLED: AtomicBool = AtomicBool::new(false);
static START_TIME: AtomicU64 = AtomicU64::new(0);

/// Enable `thag` profiling.
///
/// # Errors
///
/// This function will return an error if there's an overflow due to the time elapsed being too large for the field.
pub fn enable_profiling(enabled: bool) -> ThagResult<()> {
    PROFILING_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled {
        FIRST_WRITE.store(true, Ordering::SeqCst);
        // Store start time when profiling is enabled
        let Ok(now) = u64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros(),
        ) else {
            return Err(ThagError::FromStr("Time value too large".into()));
        };
        START_TIME.store(now, Ordering::SeqCst);
    }
    Ok(())
}

pub fn is_profiling_enabled() -> bool {
    PROFILING_ENABLED.load(Ordering::SeqCst)
}

// pub struct ProfileContext {
//     script_name: String,
//     start_time: SystemTime,
//     session_id: String, // Could be useful for multiple runs
// }

pub struct Profile {
    start: Option<Instant>,
    name: &'static str,
}

impl Profile {
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        let start = if is_profiling_enabled() {
            PROFILE_STACK.with(|stack| {
                stack.borrow_mut().push(name);
            });
            Some(Instant::now())
        } else {
            None
        };

        Self { start, name }
    }

    fn get_parent_stack() -> String {
        PROFILE_STACK.with(|stack| {
            let stack = stack.borrow();
            stack
                .iter()
                .take(stack.len().saturating_sub(1))
                .copied()
                .collect::<Vec<_>>()
                .join(";")
        })
    }

    fn write_trace_event(&self, duration: std::time::Duration) -> Result<()> {
        if !is_profiling_enabled() {
            return Ok(());
        }

        let micros = duration.as_micros();
        if micros == 0 {
            return Ok(());
        }

        let first_write = FIRST_WRITE.swap(false, Ordering::SeqCst);
        let file = File::options()
            .create(true)
            .write(true)
            .truncate(first_write)
            .append(!first_write)
            .open("thag-profile.folded")?;

        let mut writer = std::io::BufWriter::new(file);

        if first_write {
            // Write metadata as comments
            writeln!(writer, "# Format: Inferno folded stacks")?;
            writeln!(
                writer,
                "# Script: {}",
                std::env::current_exe().unwrap_or_default().display()
            )?;
            writeln!(writer, "# Started: {}", START_TIME.load(Ordering::SeqCst))?;
            writeln!(writer, "# Version: {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(writer, "# Platform: {}", std::env::consts::OS)?;
            // Add blank line after metadata
            writeln!(writer)?;
        }

        let stack = Self::get_parent_stack();
        let entry = if stack.is_empty() {
            self.name.to_string()
        } else {
            format!("{stack};{}", self.name)
        };

        writeln!(writer, "{entry} {micros}")?;
        writer.flush()?;
        Ok(())
    }
}

pub fn end_profile_section(section_name: &'static str) -> Option<Profile> {
    PROFILE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(pos) = stack.iter().position(|&name| name == section_name) {
            // Remove this section and all nested sections after it
            stack.truncate(pos);
            Some(Profile {
                start: None, // Profile is already ended
                name: section_name,
            })
        } else {
            None
        }
    })
}
impl Drop for Profile {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            let elapsed = start.elapsed();
            let _ = self.write_trace_event(elapsed);
            PROFILE_STACK.with(|stack| {
                stack.borrow_mut().pop();
            });
        }
    }
}

/// Profile the enclosing function if profiling is enabled.
///
/// Normally code this at the start of the function, after any declarations.
/// Pass the function name (or alternative identifier if you know what you're
/// doing) as a string literal argument.
///
/// E.g.:
///
/// ```Rust
/// fn foo(bar) {
///     profile!("foo");
///     ...
/// }
///
/// ```
#[macro_export]
macro_rules! profile {
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new($name);
    };
}

/// Profile a specific section of code if profiling is enabled.
/// Pass a descriptive name as a string literal argument.
///
/// The scope of the section will include all following profiled
/// sections until the end of the function, or the end of the enclosing
/// block. Unfortunately you can't just enclose the section of code in
/// a block at will without hiding them from the surrounding code, because
/// the normal Rust rules apply. So it's strongly recommended that the
/// section names be chosen to reflect the fact that the scope also includes
/// the following named sections, e.g. `bar_and_baz` in the example below.
///
/// E.g.:
///
/// ```Rust
/// fn foo() {
///     profile!("foo");
///     ...
///     profile_section!("bar_and_baz");
///     ...
///     profile_section!("baz");
///     ...
/// }
///
/// ```
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new($name);
    };
}

/// Profile the enclosing method if profiling is enabled. Pass a descriptive name
/// as a string literal argument.
///
/// E.g.:
///
/// ```Rust
/// impl Foo {}
///     fn new() {
///         profile_method!("Foo::new");
///         ...
///     }
/// }
///
/// ```
#[macro_export]
macro_rules! profile_method {
    () => {
        const NAME: &'static str = concat!(module_path!(), "::", stringify!(profile_method));
        let _profile = $crate::profiling::Profile::new(NAME);
    };
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new($name);
    };
}

// TODO: Maybe implement
// Optional: A more detailed version that includes file and line information
// #[macro_export]
// macro_rules! profile_method_detailed {
//     () => {
//         let _profile = $crate::profiling::Profile::new(concat!(
//             module_path!(),
//             "::",
//             function_name!(),
//             " at ",
//             file!(),
//             ":",
//             line!()
//         ));
//     };
// }

#[derive(Default)]
pub struct ProfileStats {
    pub calls: HashMap<String, u64>,
    pub total_time: HashMap<String, u128>, // Change to u128 for microseconds
    pub async_boundaries: HashSet<String>,
    // Keep existing fields for backwards compatibility
    count: u64,
    duration_total: std::time::Duration,
    min_time: Option<std::time::Duration>,
    max_time: Option<std::time::Duration>,
}

impl ProfileStats {
    pub fn record(&mut self, func_name: &str, duration: std::time::Duration) {
        *self.calls.entry(func_name.to_string()).or_default() += 1;
        *self.total_time.entry(func_name.to_string()).or_default() += duration.as_micros();
    }

    pub fn mark_async(&mut self, func_name: &str) {
        self.async_boundaries.insert(func_name.to_string());
    }

    #[must_use]
    pub fn average(&self) -> Option<std::time::Duration> {
        if self.count > 0 {
            let count = u32::try_from(self.count).unwrap_or(u32::MAX);
            Some(self.duration_total / count)
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_async_boundary(&self, func_name: &str) -> bool {
        self.async_boundaries.contains(func_name)
    }

    // Getter methods for private fields if needed
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }

    #[must_use]
    pub const fn total_duration(&self) -> std::time::Duration {
        self.duration_total
    }

    #[must_use]
    pub const fn min_time(&self) -> Option<std::time::Duration> {
        self.min_time
    }

    #[must_use]
    pub const fn max_time(&self) -> Option<std::time::Duration> {
        self.max_time
    }
}
