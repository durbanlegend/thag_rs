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
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering},
    Mutex,
};
use std::time::{Instant, SystemTime};

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
            alloc: format!("{base}-alloc.log")
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

static_lazy! {
    AllocationLogFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

thread_local! {
    static PROFILE_STACK: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

static PROFILING_ENABLED: AtomicBool = AtomicBool::new(false);
static START_TIME: AtomicU64 = AtomicU64::new(0);

#[global_allocator]
static ALLOCATOR: AllocationProfiler = AllocationProfiler::new();

struct AllocationProfiler {
    inner: System,
    total_allocated: AtomicUsize,
}

impl AllocationProfiler {
    const fn new() -> Self {
        Self {
            inner: System,
            total_allocated: AtomicUsize::new(0),
        }
    }

    fn get() -> &'static Self {
        &ALLOCATOR
    }

    #[allow(clippy::unused_self)]
    fn write_log(&self, size: usize, is_alloc: bool) -> Result<(), Box<dyn std::error::Error>> {
        static IS_LOGGING: AtomicBool = AtomicBool::new(false);

        if IS_LOGGING.load(Ordering::SeqCst) {
            return Ok(());
        }

        IS_LOGGING.store(true, Ordering::SeqCst);

        let result = (|| {
            let mut buffer = [0u8; 1024];
            let mut pos = 0;

            // Write fixed-length header first
            let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            let timestamp_bytes = duration.as_micros().to_ne_bytes();
            buffer[pos..pos + 16].copy_from_slice(&timestamp_bytes);
            pos += 16;

            buffer[pos] = if is_alloc { b'+' } else { b'-' };
            pos += 1;

            buffer[pos..pos + 8].copy_from_slice(&size.to_ne_bytes());
            pos += 8;

            // Reserve space for stack length - we'll write it later
            let stack_len_pos = pos;
            pos += 4;

            // Write stack frames and count them
            let stack_len = PROFILE_STACK
                .try_with(|s| {
                    s.try_borrow().map_or(0, |stack| {
                        let mut len = 0u32;
                        for frame in stack.iter() {
                            let frame_bytes = frame.as_bytes();
                            if frame_bytes.len() <= 256
                                && (pos + frame_bytes.len() + 1) < buffer.len()
                            {
                                buffer[pos..pos + frame_bytes.len()].copy_from_slice(frame_bytes);
                                pos += frame_bytes.len();
                                buffer[pos] = b';';
                                pos += 1;
                                len += 1;
                            }
                        }
                        len
                    })
                })
                .unwrap_or(0);

            // Now write the stack length
            buffer[stack_len_pos..stack_len_pos + 4].copy_from_slice(&stack_len.to_ne_bytes());

            buffer[pos] = b'\n';
            pos += 1;

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&ProfilePaths::get().alloc)?;

            file.write_all(&buffer[..pos])?;
            Ok(())
        })();

        IS_LOGGING.store(false, Ordering::SeqCst);
        result
    }
}

unsafe impl GlobalAlloc for AllocationProfiler {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() && is_profiling_enabled() {
            self.total_allocated
                .fetch_add(layout.size(), Ordering::SeqCst);
            if let Err(e) = self.write_log(layout.size(), true) {
                eprintln!("Error writing allocation log: {e}");
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
        if is_profiling_enabled() {
            if let Err(e) = self.write_log(layout.size(), false) {
                eprintln!("Error writing allocation log: {e}");
            }
        }
    }
}

#[derive(Clone)]
struct ProfileFilePaths {
    time: String,
    memory: String,
    alloc: String,
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
/// This function handles the initialization sequence for all profiling files:
/// - For Time profiling: creates and initializes the time profile file
/// - For Memory profiling: creates and initializes both memory and allocation log files
/// - For Both: initializes all three files
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
            AllocationLogFile::init();
            reset_profile_file(MemoryProfileFile::get(), "memory")?;
            reset_profile_file(AllocationLogFile::get(), "allocation")?;
            initialize_profile_file(&paths.memory, "Memory Profile")?;
            initialize_profile_file(&paths.alloc, "Allocation Log")?;
        }
        ProfileType::Both => {
            // Initialize all files
            TimeProfileFile::init();
            MemoryProfileFile::init();
            AllocationLogFile::init();

            // Reset all files
            reset_profile_file(TimeProfileFile::get(), "time")?;
            reset_profile_file(MemoryProfileFile::get(), "memory")?;
            reset_profile_file(AllocationLogFile::get(), "allocation")?;

            // Initialize all files with headers
            initialize_profile_file(&paths.time, "Time Profile")?;
            initialize_profile_file(&paths.memory, "Memory Profile")?;
            initialize_profile_file(&paths.alloc, "Allocation Log")?;
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

    PROFILING_ENABLED.store(enabled, Ordering::SeqCst);
    Ok(())
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
pub fn is_profiling_enabled() -> bool {
    PROFILING_ENABLED.load(Ordering::SeqCst)
}

#[derive(Debug, Clone, Copy)]
pub enum ProfileType {
    Time, // Wall clock/elapsed time
    Memory,
    Both,
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

    #[must_use]
    pub fn new(name: &'static str, requested_type: ProfileType) -> Self {
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

        if is_profiling_enabled() {
            PROFILE_STACK.with(|stack| {
                if let Ok(mut guard) = stack.try_borrow_mut() {
                    guard.push(name);
                }
            });
        }

        Self {
            start: Some(Instant::now()),
            name,
            profile_type,
            initial_memory: Some(
                AllocationProfiler::get()
                    .total_allocated // Changed from current_allocated
                    .load(Ordering::SeqCst),
            ),
            _not_send: PhantomData,
        }
    }

    /// Returns the current profiling stack as a semicolon-separated string.
    ///
    /// The stack represents the current call hierarchy, with each level
    /// separated by semicolons, matching the format expected by flamegraph
    /// generation tools.
    ///
    /// # Returns
    /// A string representing the current profiling stack, excluding the
    /// most recent addition (which will be added separately in the event logging)
    fn get_parent_stack() -> String {
        PROFILE_STACK
            .try_with(|stack| {
                stack.try_borrow().map_or_else(
                    |_| String::new(),
                    |stack_ref| {
                        stack_ref
                            .iter()
                            .take(stack_ref.len().saturating_sub(1))
                            .copied()
                            .collect::<Vec<_>>()
                            .join(";")
                    },
                )
            })
            .unwrap_or_default()
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
        if !is_profiling_enabled() {
            return Ok(());
        }

        let micros = duration.as_micros();
        if micros == 0 {
            return Ok(());
        }

        let stack = Self::get_parent_stack();
        let entry = if stack.is_empty() {
            self.name.to_string()
        } else {
            format!("{stack};{}", self.name)
        };

        let paths = ProfilePaths::get();
        Self::write_profile_event(
            &paths.time,
            TimeProfileFile::get(),
            &format!("{entry} {micros}"),
        )
    }

    /// Records a memory profiling event.
    ///
    /// Writes the memory usage delta for a profiled section along with its
    /// stack trace to the memory profile file.
    ///
    /// # Arguments
    /// * `delta` - The change in memory usage for the profiled section
    ///
    /// # Errors
    /// Returns a `ThagError` if writing to the profile file fails
    fn write_memory_event(&self, delta: usize) -> ThagResult<()> {
        if !is_profiling_enabled() || delta == 0 {
            return Ok(());
        }

        // Get stack trace in same format as allocation log
        let stack_data = PROFILE_STACK
            .try_with(|s| {
                s.try_borrow().ok().map(|stack| {
                    if stack.is_empty() {
                        self.name.to_string()
                    } else {
                        format!("{};{}", stack.join(";"), self.name)
                    }
                })
            })
            .unwrap_or(None) // Handle the Result
            .unwrap_or_else(|| self.name.to_string()); // Handle the Option

        // Write to .folded file
        let entry = format!("{stack_data} {delta}");
        let paths = ProfilePaths::get();
        Self::write_profile_event(&paths.memory, MemoryProfileFile::get(), &entry)
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        if let Some(start) = self.start.take() {
            // Handle time profiling first
            match self.profile_type {
                ProfileType::Time | ProfileType::Both => {
                    let elapsed = start.elapsed();
                    let _ = self.write_time_event(elapsed);
                }
                ProfileType::Memory => (),
            }
        }

        // Handle memory profiling and stack maintenance together
        if matches!(self.profile_type, ProfileType::Memory | ProfileType::Both) {
            // Write memory event if we have initial memory
            if let Some(initial) = self.initial_memory {
                let final_memory = AllocationProfiler::get()
                    .total_allocated
                    .load(Ordering::SeqCst);
                let delta = final_memory.saturating_sub(initial);

                if delta > 0 {
                    let _ = self.write_memory_event(delta);
                }
            }

            // Pop from profile stack
            let _ = PROFILE_STACK.try_with(|s| {
                if let Ok(mut stack) = s.try_borrow_mut() {
                    stack.pop();
                }
            });
        }
    }
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
#[must_use]
pub fn end_profile_section(section_name: &'static str) -> Option<Profile> {
    PROFILE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack
            .iter()
            .position(|&name| name == section_name)
            .map_or_else(
                || None,
                |pos| {
                    // Remove this section and all nested sections after it
                    stack.truncate(pos);
                    Some(Profile {
                        start: Some(Instant::now()),
                        name: section_name,
                        profile_type: ProfileType::Time,
                        initial_memory: None,
                        _not_send: PhantomData,
                    })
                },
            )
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
        let _profile = $crate::profiling::Profile::new($name, $crate::profiling::get_global_profile_type());
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
        let _profile = $crate::profiling::Profile::new($name, $crate::profiling::get_global_profile_type());
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
        let _profile = $crate::profiling::Profile::new($name, $crate::profiling::get_global_profile_type());
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
    use std::fs;

    fn setup_test() -> Result<(), Box<dyn std::error::Error>> {
        enable_profiling(true, ProfileType::Memory)?;
        Ok(())
    }

    fn cleanup_test() -> Result<(), Box<dyn std::error::Error>> {
        enable_profiling(false, ProfileType::Memory)?;
        Ok(())
    }

    #[test]
    fn test_basic_allocation() -> Result<(), Box<dyn std::error::Error>> {
        enable_profiling(true, ProfileType::Memory)?;

        let before = ALLOCATOR.total_allocated.load(Ordering::SeqCst);
        let x = Box::new(42i32);
        let after = ALLOCATOR.total_allocated.load(Ordering::SeqCst);

        assert!(after > before, "Allocation not tracked");
        drop(x);

        enable_profiling(false, ProfileType::Memory)?;
        Ok(())
    }
}
