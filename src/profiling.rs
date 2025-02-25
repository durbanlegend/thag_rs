#![allow(clippy::module_name_repetitions)]
//!
//! Basic profiling for `thag_rs`, also (TODO) intended as an option for user scripts.
//!
//! Placing the following instruction at the start of your code will allow profiling to be enabled by running with `--features=profiling`.
//!
//! ```ignore
//! profiling::enable_profiling(true, ProfileType::Both)?; // Could feature-gate this
//! ```
//!
//! Output will be in the form of a file called `thag-profile.folded` in the current working directory, and may be
//! displayed as statistics or as an `inferno` [flamechart](https://medium.com/performance-engineering-for-the-ordinary-barbie/profiling-flame-chart-vs-flame-graph-7b212ddf3a83)
//! using the `demo/thag_profile.rs` script, which you may wish to compile to a command first by using the `-x` option.
//!
//! `demo/thag_profile.rs` also allows you to filter out unwanted events, e.g. to make it easier to drill down into the flamechart.
//!
use crate::{lazy_static_var, static_lazy, ThagError, ThagResult, Verbosity};
use chrono::Local;
use memory_stats::memory_stats;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::ptr;
use std::sync::{
    atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicU8, AtomicUsize, Ordering},
    Mutex,
};
use std::time::{Instant, SystemTime};
// Single atomic for runtime profiling state
static PROFILING_STATE: AtomicBool = AtomicBool::new(false);

// Compile-time feature check
#[cfg(feature = "profiling")]
const PROFILING_FEATURE: bool = true;

#[cfg(not(feature = "profiling"))]
const PROFILING_FEATURE: bool = false;

static PROFILE_TYPE: AtomicU8 = AtomicU8::new(0); // 0 = None, 1 = Time, 2 = Memory, 3 = Both

static_lazy! {
    ProfilePaths: ProfileFilePaths = {
        let script_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("unknown"));
        let script_stem = script_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let base = format!("{script_stem}-{timestamp}");

        ProfileFilePaths {
            time: format!("{base}.folded"),
            memory: format!("{base}-memory.folded"),
        }
    }
}

// File handles
static_lazy! {
    TimeProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

static_lazy! {
    MemoryProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

const MAX_PROFILE_DEPTH: usize = 100;

static PROFILE_STACK: [AtomicPtr<&'static str>; MAX_PROFILE_DEPTH] =
    [const { AtomicPtr::new(ptr::null_mut()) }; MAX_PROFILE_DEPTH];
static STACK_DEPTH: AtomicUsize = AtomicUsize::new(0);

// Safe interface for stack operations
pub(crate) fn push_profile(name: &'static str) -> bool {
    let idx = STACK_DEPTH.load(Ordering::SeqCst);
    if idx >= MAX_PROFILE_DEPTH {
        return false;
    }

    let name_ptr = Box::into_raw(Box::new(name));
    PROFILE_STACK[idx].store(name_ptr, Ordering::SeqCst);
    STACK_DEPTH.store(idx + 1, Ordering::SeqCst);
    true
}

pub fn pop_profile() {
    let idx = STACK_DEPTH.load(Ordering::SeqCst);
    if idx > 0 {
        let old_ptr = PROFILE_STACK[idx - 1].swap(ptr::null_mut(), Ordering::SeqCst);
        if !old_ptr.is_null() {
            // Clean up the Box we created
            unsafe {
                drop(Box::from_raw(old_ptr));
            }
        }
        STACK_DEPTH.store(idx - 1, Ordering::SeqCst);
    }
}

pub(crate) fn get_profile_stack() -> Vec<&'static str> {
    let depth = STACK_DEPTH.load(Ordering::SeqCst);
    // println!("get_profile_stack: depth = {depth}"); // Debug
    let mut result = Vec::with_capacity(depth);

    for frame in PROFILE_STACK.iter().take(depth) {
        let ptr = frame.load(Ordering::SeqCst);
        if !ptr.is_null() {
            unsafe {
                let name = *ptr;
                // println!("  stack[{i}] = {name}"); // Debug
                result.push(name);
            }
        }
    }
    // println!("get_profile_stack returning: {result:?}"); // Debug
    result
}

// For validation in debug builds
// #[cfg(debug_assertions)]
#[allow(dead_code)]
fn validate_profile_stack() -> Option<String> {
    let depth = STACK_DEPTH.load(Ordering::SeqCst);
    if depth >= MAX_PROFILE_DEPTH {
        return Some(format!("Stack depth {depth} exceeds limit"));
    }
    // Removed duplicate entry check - recursive calls are valid
    None
}

static START_TIME: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
struct ProfileFilePaths {
    time: String,
    memory: String,
}

/// Resets a profile file by clearing its buffer writer.
///
/// # Arguments
/// * `file` - The mutex-protected buffer writer to reset
/// * `file_type` - A description of the file type for error messages (e.g., "time", "memory")
///
/// # Errors
/// Returns a `ThagError` if the mutex lock fails
fn reset_profile_file(file: &Mutex<Option<BufWriter<File>>>, file_type: &str) -> ThagResult<()> {
    *file
        .lock()
        .map_err(|_| ThagError::Profiling(format!("Failed to lock {file_type} profile file")))? =
        None;
    Ok(())
}

/// Initializes profile files based on the specified profile type.
///
/// This function handles the initialization sequence for both profiling files:
/// - For Time profiling: creates and initializes the time profile file
/// - For Memory profiling: creates and initializes memory profile file
/// - For Both: initializes both files
///
/// # Arguments
/// * `profile_type` - The type of profiling to initialize files for
///
/// # Errors
/// Returns a `ThagError` if any file operations fail
fn initialize_profile_files(profile_type: ProfileType) -> ThagResult<()> {
    let paths = ProfilePaths::get();

    match profile_type {
        ProfileType::Time => {
            TimeProfileFile::init();
            reset_profile_file(TimeProfileFile::get(), "time")?;
            initialize_profile_file(&paths.time, "Time Profile")?;
        }
        ProfileType::Memory => {
            MemoryProfileFile::init();
            reset_profile_file(MemoryProfileFile::get(), "memory")?;
            initialize_profile_file(&paths.memory, "Memory Profile")?;
        }
        ProfileType::Both => {
            // Initialize all files
            TimeProfileFile::init();
            MemoryProfileFile::init();

            // Reset all files
            reset_profile_file(TimeProfileFile::get(), "time")?;
            reset_profile_file(MemoryProfileFile::get(), "memory")?;

            // Initialize all files with headers
            initialize_profile_file(&paths.time, "Time Profile")?;
            initialize_profile_file(&paths.memory, "Memory Profile")?;
        }
    }
    Ok(())
}

pub fn get_global_profile_type() -> ProfileType {
    match PROFILE_TYPE.load(Ordering::SeqCst) {
        2 => ProfileType::Memory,
        3 => ProfileType::Both,
        _ => ProfileType::Time,
    }
}
fn set_profile_type(profile_type: ProfileType) {
    let value = match profile_type {
        ProfileType::Time => 1,
        ProfileType::Memory => 2,
        ProfileType::Both => 3,
    };
    PROFILE_TYPE.store(value, Ordering::SeqCst);
}

/// Enables or disables profiling with the specified profile type.
///
/// When enabling profiling, this function:
/// 1. Initializes path information
/// 2. Records the start time
/// 3. Sets up appropriate profile files based on the profile type
///
/// When disabling profiling, it ensures all profiling operations are stopped.
///
/// # Arguments
/// * `enabled` - Whether to enable or disable profiling
/// * `profile_type` - The type of profiling to enable (Time, Memory, or Both)
///
/// # Errors
/// Returns a `ThagError` if:
/// - Time value conversion fails
/// - File operations fail
/// - Mutex operations fail
pub fn enable_profiling(enabled: bool, profile_type: ProfileType) -> ThagResult<()> {
    if enabled {
        set_profile_type(profile_type);

        let Ok(now) = u64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros(),
        ) else {
            return Err(ThagError::Profiling("Time value too large".into()));
        };
        START_TIME.store(now, Ordering::SeqCst);

        initialize_profile_files(profile_type)?;
    }

    set_profiling_enabled(enabled); // Using the new function instead of direct atomic access
    Ok(())
}

// Function to set runtime profiling state
pub fn set_profiling_enabled(enabled: bool) {
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
}

/// Creates and initializes a single profile file with header information.
///
/// Creates the file if it doesn't exist, truncates it if it does, and writes
/// standard header information including:
/// - Profile type
/// - Script path
/// - Start timestamp
/// - Version information
///
/// # Arguments
/// * `path` - The path where the file should be created
/// * `profile_type` - A description of the profile type for the header
///
/// # Errors
/// Returns a `ThagError` if file creation or writing fails
fn initialize_profile_file(path: &str, profile_type: &str) -> ThagResult<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    writeln!(file, "# {profile_type}")?;
    writeln!(
        file,
        "# Script: {}",
        std::env::current_exe().unwrap_or_default().display()
    )?;
    writeln!(file, "# Started: {}", START_TIME.load(Ordering::SeqCst))?;
    writeln!(file, "# Version: {}", env!("CARGO_PKG_VERSION"))?;
    if path.ends_with("alloc.log") {
        writeln!(file, "# Format: operation|size")?;
    }
    writeln!(file)?;

    Ok(())
}

/// Checks if profiling is currently enabled.
///
/// This is used throughout the profiling system to determine whether
/// profiling operations should be performed. It's atomic and thread-safe.
///
/// # Returns
/// `true` if profiling is enabled, `false` otherwise
#[inline(always)]
#[allow(clippy::inline_always)]
pub fn is_profiling_enabled() -> bool {
    PROFILING_FEATURE || PROFILING_STATE.load(Ordering::SeqCst)
}

#[derive(Debug, Clone, Copy)]
pub enum ProfileType {
    Time, // Wall clock/elapsed time
    Memory,
    Both,
}

impl ProfileType {
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "time" => Some(Self::Time),
            "memory" => Some(Self::Memory),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

pub struct Profile {
    start: Option<Instant>,
    name: &'static str,
    profile_type: ProfileType,
    initial_memory: Option<usize>,     // For memory delta
    _not_send: PhantomData<*const ()>, // Makes Profile !Send
}

impl Profile {
    #[must_use]
    pub const fn new_raw(name: &'static str) -> Self {
        Self {
            start: None,
            name,
            profile_type: ProfileType::Time, // Default to time profiling
            initial_memory: None,
            _not_send: PhantomData,
        }
    }

    /// Creates a new `Profile` to profile a section of code.
    ///
    /// # Panics
    ///
    /// Panics if stack validation fails.
    #[must_use]
    #[inline(always)]
    #[allow(clippy::inline_always)]
    pub fn new(name: &'static str, requested_type: ProfileType) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }
        // println!("Profile::new called with name: {name} and type: {requested_type:?}");
        // TODO: make the requested type a true override once the proc macro is implemented
        let global_type = match PROFILE_TYPE.load(Ordering::SeqCst) {
            2 => ProfileType::Memory,
            3 => ProfileType::Both,
            _ => ProfileType::Time, // default
        };

        // Use the more comprehensive of the two types
        let profile_type = match (requested_type, global_type) {
            (ProfileType::Both, _) | (_, ProfileType::Both) => ProfileType::Both,
            (ProfileType::Memory, _) | (_, ProfileType::Memory) => ProfileType::Memory,
            _ => ProfileType::Time,
        };

        let initial_memory = if matches!(requested_type, ProfileType::Memory | ProfileType::Both) {
            // Get initial memory snapshot
            memory_stats().map(|stats| stats.physical_mem)
        } else {
            None
        };

        // #[cfg(debug_assertions)]
        if let Some(err) = validate_profile_stack() {
            panic!("Stack validation failed: {err}");
        }

        // Push to stack as before
        push_profile(name);

        Some(Self {
            name,
            profile_type,
            start: Some(Instant::now()),
            initial_memory,
            _not_send: PhantomData,
        })
    }

    /// Writes a profiling event to the specified profile file.
    ///
    /// This is a low-level function used by both time and memory profiling
    /// to write events in a consistent format. It handles file creation,
    /// buffering, and error handling.
    ///
    /// # Arguments
    /// * `path` - The path to the profile file
    /// * `file` - The mutex-protected buffer writer for the file
    /// * `entry` - The formatted entry to write (including stack trace and measurement)
    ///
    /// # Errors
    /// Returns a `ThagError` if:
    /// * The mutex lock fails
    /// * File operations fail
    /// * Writing to the file fails
    fn write_profile_event(
        path: &str,
        file: &Mutex<Option<BufWriter<File>>>,
        entry: &str,
    ) -> ThagResult<()> {
        let mut guard = file
            .lock()
            .map_err(|_| ThagError::Profiling("Failed to lock profile file".into()))?;

        if guard.is_none() {
            *guard = Some(BufWriter::new(
                OpenOptions::new().create(true).append(true).open(path)?,
            ));
        }

        if let Some(writer) = guard.as_mut() {
            writeln!(writer, "{entry}")?;
            writer.flush()?;
        }
        drop(guard);
        Ok(())
    }

    /// Records a time profiling event.
    ///
    /// Writes the elapsed time for a profiled section along with its stack trace
    /// to the time profile file.
    ///
    /// # Arguments
    /// * `duration` - The elapsed time of the profiled section
    ///
    /// # Errors
    /// Returns a `ThagError` if writing to the profile file fails
    fn write_time_event(&self, duration: std::time::Duration) -> ThagResult<()> {
        // Profile must exist and profiling must be enabled if we got here
        // Only keep the business logic checks

        let micros = duration.as_micros();
        if micros == 0 {
            return Ok(());
        }

        let stack = get_profile_stack();
        let entry = if stack.is_empty() {
            format!("{} {micros}", self.name)
        } else {
            format!("{} {micros}", stack.join(";"))
        };

        let paths = ProfilePaths::get();
        Self::write_profile_event(&paths.time, TimeProfileFile::get(), &entry)
    }

    fn write_memory_event_with_op(&self, delta: usize, op: char) -> ThagResult<()> {
        if delta == 0 {
            // Keep this as it's a business logic check
            return Ok(());
        }

        // Get current stack as string
        let stack_data = {
            let stack = get_profile_stack();
            if stack.is_empty() {
                self.name.to_string()
            } else {
                stack.join(";")
            }
        };
        let entry = format!("{stack_data} {op}{delta}");

        let paths = ProfilePaths::get();
        Self::write_profile_event(&paths.memory, MemoryProfileFile::get(), &entry)
    }

    // fn write_memory_event(&self, delta: usize) -> ThagResult<()> {
    //     if delta == 0 {
    //         // Keep this as it's a business logic check
    //         return Ok(());
    //     }

    //     // Get current stack as string
    //     let stack_data = {
    //         let stack = get_profile_stack();
    //         if stack.is_empty() {
    //             self.name.to_string()
    //         } else {
    //             format!("{};{}", stack.join(";"), self.name)
    //         }
    //     };
    //     let entry = format!("{stack_data} {delta}");

    //     let paths = ProfilePaths::get();
    //     Self::write_profile_event(&paths.memory, MemoryProfileFile::get(), &entry)
    // }

    fn record_memory_change(&self, delta: usize) -> ThagResult<()> {
        if delta == 0 {
            return Ok(());
        }

        // Record allocation
        self.write_memory_event_with_op(delta, '+')?;

        // Record corresponding deallocation
        // Store both events atomically to maintain pairing
        self.write_memory_event_with_op(delta, '-')?;

        Ok(())
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        if let Some(start) = self.start.take() {
            // Handle time profiling as before
            match self.profile_type {
                ProfileType::Time | ProfileType::Both => {
                    let elapsed = start.elapsed();
                    let _ = self.write_time_event(elapsed);
                }
                ProfileType::Memory => (),
            }
        }

        // Handle memory profiling
        if matches!(self.profile_type, ProfileType::Memory | ProfileType::Both) {
            if let Some(initial) = self.initial_memory {
                if let Some(stats) = memory_stats() {
                    let final_memory = stats.physical_mem;
                    let delta = final_memory.saturating_sub(initial);

                    if delta > 0 {
                        let _ = self.record_memory_change(delta);
                    }
                }
            }
        }

        // Pop from stack as before
        pop_profile();
    }
}

// Optional: add memory info to error handling
#[derive(Debug)]
pub enum MemoryError {
    StatsUnavailable,
    DeltaCalculationFailed,
}

#[allow(dead_code)]
fn get_memory_delta(initial: usize) -> Result<usize, MemoryError> {
    memory_stats()
        .ok_or(MemoryError::StatsUnavailable)
        .and_then(|stats| {
            let final_memory = stats.physical_mem;
            if final_memory >= initial {
                Ok(final_memory - initial)
            } else {
                Err(MemoryError::DeltaCalculationFailed)
            }
        })
}

/// Run a function or closure only if the global verbosity is Verbose or higher.
/// Intended to run an eprintln! of a message. This is meant as an equivalent to `vlog!`
/// but without risking an infinite recursion with profiling trying to log and logging
/// trying to profile.
///
/// Uses a function or closure as its argument rather than accept a pre-formatted message
/// argument, so that we only do the formatting if we need to.
#[allow(dead_code)]
fn verbose_only(fun: impl Fn()) {
    let verbosity = lazy_static_var!(Verbosity, crate::get_verbosity());
    if *verbosity as u8 >= Verbosity::Verbose as u8 {
        fun();
    }
}

// #[must_use]
// /// Validates that the stack truncation will be valid
// #[cfg(debug_assertions)]
// fn validate_stack_truncation(pos: usize) -> Option<String> {
//     let depth = STACK_DEPTH.load(Ordering::SeqCst);
//     if pos >= depth {
//         return Some(format!(
//             "Truncation position {pos} exceeds stack depth {depth}"
//         ));
//     }
//     None
// }

/// Ends profiling for a named section early by removing it and all nested
/// sections after it from the profiling stack.
///
/// This is useful when you want to stop profiling a section before its
/// natural scope ends.
///
/// # Arguments
/// * `section_name` - The name of the section to end
///
/// # Returns
/// Some(Profile) if the section was found and ended, None if the section
/// wasn't found in the current stack
/// # Panics
/// If the stack truncation position exceeds the current stack depth
#[must_use]
pub fn end_profile_section(section_name: &'static str) -> Option<Profile> {
    let depth = STACK_DEPTH.load(Ordering::SeqCst);
    let mut pos = None;

    // Find position of section
    for (i, frame) in PROFILE_STACK.iter().enumerate().take(depth) {
        let ptr = frame.load(Ordering::SeqCst);
        if !ptr.is_null() {
            unsafe {
                if *ptr == section_name {
                    pos = Some(i);
                    break;
                }
            }
        }
    }

    pos.map(|p| {
        // Clean up everything after position p
        for frame in PROFILE_STACK.iter().take(depth).skip(p) {
            let ptr = frame.swap(ptr::null_mut(), Ordering::SeqCst);
            if !ptr.is_null() {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
        }

        // Update depth
        STACK_DEPTH.store(p, Ordering::SeqCst);

        Profile {
            start: Some(Instant::now()),
            name: section_name,
            profile_type: ProfileType::Time,
            initial_memory: None,
            _not_send: PhantomData,
        }
    })
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
///     profile_fn!("foo");
///     ...
/// }
///
/// ```
#[macro_export]
macro_rules! profile_fn {
    ($name:expr) => {
        let _profile =
            $crate::profiling::Profile::new($name, $crate::profiling::get_global_profile_type());
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
        let _profile =
            $crate::profiling::Profile::new($name, $crate::profiling::get_global_profile_type());
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
        let _profile =
            $crate::profiling::Profile::new(NAME, $crate::profiling::get_global_profile_type());
    };
    ($name:expr) => {
        let _profile =
            $crate::profiling::Profile::new($name, $crate::profiling::get_global_profile_type());
    };
}

/// Profiles memory usage in the enclosing function or scope.
///
/// Records memory allocation patterns and changes in memory usage
/// for the duration of the scope.
///
/// # Example
/// ```
/// use thag_rs::profile_memory;
/// fn allocate_buffer() {
///     profile_memory!("allocate_buffer");
///     let buffer = vec![0; 1024];
///     // Memory usage will be tracked
/// }
/// ```
#[macro_export]
macro_rules! profile_memory {
    ($name:expr) => {
        let _profile =
            $crate::profiling::Profile::new($name, $crate::profiling::ProfileType::Memory);
    };
}

/// Profiles both execution time and memory usage in the enclosing
/// function or scope.
///
/// Combines time and memory profiling to provide a complete picture
/// of both performance and memory usage.
///
/// # Example
/// ```
/// use thag_rs::profile_both;
/// fn process_data() {
///     profile_both!("process_data");
///     // Both time and memory usage will be tracked
/// }
/// ```
#[macro_export]
macro_rules! profile_both {
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new($name, $crate::profiling::ProfileType::Both);
    };
}

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
    /// Records a profiling measurement for a function.
    ///
    /// Maintains running statistics including:
    /// * Call count
    /// * Total time
    /// * Minimum and maximum durations
    ///
    /// # Arguments
    /// * `func_name` - The name of the function being profiled
    /// * `duration` - The duration of this particular call
    pub fn record(&mut self, func_name: &str, duration: std::time::Duration) {
        *self.calls.entry(func_name.to_string()).or_default() += 1;
        *self.total_time.entry(func_name.to_string()).or_default() += duration.as_micros();
    }

    /// Marks a function as an async boundary.
    ///
    /// Async boundaries are points where asynchronous operations occur,
    /// which can be useful for understanding async execution patterns.
    ///
    /// # Arguments
    /// * `func_name` - The name of the function to mark as an async boundary
    pub fn mark_async(&mut self, func_name: &str) {
        self.async_boundaries.insert(func_name.to_string());
    }

    /// Calculates the average duration of all recorded calls.
    ///
    /// # Returns
    /// Some(Duration) containing the average if there are any recorded calls,
    /// None if no calls have been recorded
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
    /// Checks if a function is marked as an async boundary.
    ///
    /// # Arguments
    /// * `func_name` - The name of the function to check
    ///
    /// # Returns
    /// `true` if the function is marked as an async boundary, `false` otherwise
    pub fn is_async_boundary(&self, func_name: &str) -> bool {
        self.async_boundaries.contains(func_name)
    }

    /// Returns the total number of times a function was called.
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }

    /// Returns the total duration spent in a function across all calls.
    #[must_use]
    pub const fn total_duration(&self) -> std::time::Duration {
        self.duration_total
    }

    /// Returns the minimum time spent in any single call to a function.
    #[must_use]
    pub const fn min_time(&self) -> Option<std::time::Duration> {
        self.min_time
    }

    /// Returns the maximum time spent in any single call to a function.
    #[must_use]
    pub const fn max_time(&self) -> Option<std::time::Duration> {
        self.max_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sequential_test::sequential;
    use std::panic;

    struct TestGuard;

    impl Drop for TestGuard {
        fn drop(&mut self) {
            reset_stack();
            set_profiling_enabled(false);
        }
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() + panic::UnwindSafe,
    {
        // Setup
        reset_stack();
        set_profiling_enabled(true);

        // Create guard that will clean up even if test panics
        let _guard = TestGuard;

        // Run the test, catching any panics to ensure our guard runs
        let result = panic::catch_unwind(test);

        // Re-throw any panic after our guard has cleaned up
        if let Err(e) = result {
            panic::resume_unwind(e);
        }
    }

    fn reset_stack() {
        // First get and clear the depth
        let old_depth = STACK_DEPTH.swap(0, Ordering::SeqCst);

        // Then clean up all entries up to the old depth
        for i in 0..old_depth {
            let ptr = PROFILE_STACK[i].swap(ptr::null_mut(), Ordering::SeqCst);
            if !ptr.is_null() {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
        }

        // Extra safety: clear any remaining entries
        for i in old_depth..MAX_PROFILE_DEPTH {
            PROFILE_STACK[i].store(ptr::null_mut(), Ordering::SeqCst);
        }
    }

    #[test]
    #[sequential]
    fn test_profiling_stack_basic() {
        println!("\n--- test_profiling_stack_basic starting ---"); // Debug

        run_test(|| {
            assert!(push_profile("first"));
            println!("After push:"); // Debug
            let depth = STACK_DEPTH.load(Ordering::SeqCst);
            println!("STACK_DEPTH = {}", depth);

            let stack = get_profile_stack();
            println!("Stack = {:?}", stack);
            assert_eq!(stack.len(), 1);
            assert_eq!(stack[0], "first");

            pop_profile();
            assert_eq!(STACK_DEPTH.load(Ordering::SeqCst), 0);
            assert!(get_profile_stack().is_empty());
        });
    }

    #[test]
    #[sequential]
    fn test_profiling_stack_nesting() {
        run_test(|| {
            assert!(push_profile("outer"));
            assert!(push_profile("inner"));

            let stack = get_profile_stack();
            assert_eq!(stack.len(), 2);
            assert_eq!(stack[0], "outer");
            assert_eq!(stack[1], "inner");

            pop_profile();
            let stack = get_profile_stack();
            assert_eq!(stack.len(), 1);
            assert_eq!(stack[0], "outer");

            pop_profile();
            assert!(get_profile_stack().is_empty());
        });
    }

    #[test]
    #[sequential]
    fn test_profiling_stack_capacity() {
        run_test(|| {
            // Fill stack to capacity
            for _ in 0..MAX_PROFILE_DEPTH {
                assert!(push_profile("test"));
            }

            // Try to push one more
            assert!(!push_profile("overflow"));

            // Verify stack depth didn't change
            assert_eq!(STACK_DEPTH.load(Ordering::SeqCst), MAX_PROFILE_DEPTH);
        });
    }

    #[test]
    #[sequential]
    fn test_profiling_stack_empty_pop() {
        run_test(|| {
            // Try to pop empty stack
            pop_profile();
            assert_eq!(STACK_DEPTH.load(Ordering::SeqCst), 0);
        });
    }

    #[test]
    #[sequential]
    #[should_panic(expected = "Stack validation failed: Duplicate stack entry")]
    fn test_profiling_duplicate_stack_entries() {
        run_test(|| {
            let _p1 = Profile::new("same", ProfileType::Time);
            let _p2 = Profile::new("same", ProfileType::Time); // Should panic
        });
    }

    #[test]
    #[sequential]
    fn test_profiling_type_stack_management() {
        run_test(|| {
            {
                let _p1 = Profile::new("time_prof", ProfileType::Time);
                let _p2 = Profile::new("mem_prof", ProfileType::Memory);
                let _p3 = Profile::new("both_prof", ProfileType::Both);

                let stack = get_profile_stack();
                assert_eq!(stack.len(), 3);
                assert_eq!(&stack[..], &["time_prof", "mem_prof", "both_prof"]);
            } // All profiles should be dropped here

            // Verify clean stack after drops
            let final_depth = STACK_DEPTH.load(Ordering::SeqCst);
            println!("Final stack:"); // Debug
            let final_stack = get_profile_stack();
            println!("Depth: {}, Stack: {:?}", final_depth, final_stack);
            assert_eq!(final_depth, 0, "Stack not empty after drops");
            assert!(get_profile_stack().is_empty(), "Stack should be empty");
        });
    }

    #[test]
    #[sequential]
    fn test_profiling_end_profile_section() {
        run_test(|| {
            assert!(push_profile("outer"));
            assert!(push_profile("middle"));
            assert!(push_profile("inner"));

            let stack = get_profile_stack();
            assert_eq!(&stack[..], &["outer", "middle", "inner"]);

            let profile = end_profile_section("middle");
            assert!(profile.is_some());
            assert_eq!(profile.unwrap().name, "middle");

            let stack = get_profile_stack();
            assert_eq!(stack.len(), 1);
            assert_eq!(stack[0], "outer");
        });
    }

    // // Optional: debug helper
    // #[cfg(test)]
    // fn print_memory_info(context: &str) {
    //     if let Some(stats) = memory_stats() {
    //         println!(
    //             "{}: Physical: {} bytes, Virtual: {} bytes",
    //             context, stats.physical_mem, stats.virtual_mem
    //         );
    //     }
    // }
}
