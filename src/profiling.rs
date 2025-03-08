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
use backtrace::Backtrace;
use chrono::Local;
use memory_stats::memory_stats;
use once_cell::sync::Lazy;
use rustc_demangle::demangle;
use std::collections::{HashMap, HashSet};
use std::convert::Into;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
    Mutex,
};
use std::time::{Instant, SystemTime};

// Single atomic for runtime profiling state
static PROFILING_STATE: AtomicBool = AtomicBool::new(false);

// Mutex to protect profiling state changes
static PROFILING_MUTEX: Mutex<()> = Mutex::new(());

// Compile-time feature check
#[cfg(feature = "profiling")]
const PROFILING_FEATURE: bool = true;

#[cfg(not(feature = "profiling"))]
const PROFILING_FEATURE: bool = false;

static PROFILE_TYPE: AtomicU8 = AtomicU8::new(0); // 0 = None, 1 = Time, 2 = Memory, 3 = Both

// Global registry of profiled functions
static PROFILED_FUNCTIONS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
    // Acquire the mutex to ensure only one thread can enable/disable profiling at a time
    let _guard = PROFILING_MUTEX
        .lock()
        .map_err(|_| ThagError::Profiling("Failed to acquire profiling mutex".into()))?;

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

    // Whether enabling or disabling, set the state
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
    Ok(())
}

/// Disable profiling and reset the profiling stack.
pub fn disable_profiling() {
    PROFILING_STATE.store(false, Ordering::SeqCst);
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
    // id: u64,
    start: Option<Instant>,
    // name: &'static str,
    profile_type: ProfileType,
    initial_memory: Option<usize>,     // For memory delta
    path: Vec<String>,                 // Full call stack (only profiled functions)
    _not_send: PhantomData<*const ()>, // Makes Profile !Send
}

impl Profile {
    /// Creates a new `Profile` to profile a section of code.
    ///
    /// # Panics
    ///
    /// Panics if stack validation fails.
    #[allow(clippy::inline_always, clippy::too_many_lines, unused_variables)]
    pub fn new(
        name: String,
        requested_type: ProfileType,
        is_async: bool,
        is_method: bool,
    ) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        // eprintln!("Current function: {name}");

        // Get the current backtrace
        let mut current_backtrace = Backtrace::new_unresolved();
        current_backtrace.resolve();
        let mut is_within_target_range = false;

        // First, collect all relevant frames
        let mut raw_frames: Vec<String> = Vec::new();

        // If this is a method, we'll capture the calling class
        let mut maybe_method_name: Option<String> = None;
        let mut first_frame_after_profile = false;

        for frame in Backtrace::frames(&current_backtrace) {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();
                    // eprintln!("Symbol name: {name_str}");

                    // Check if we've reached the start condition
                    if !is_within_target_range
                        && name_str.contains("thag_rs::profiling::Profile::new")
                    {
                        is_within_target_range = true;
                        first_frame_after_profile = true;
                        continue;
                    }

                    // Collect frames within our target range
                    if is_within_target_range {
                        // If this is the first frame after Profile::new and it's a method,
                        // then we can extract the class name
                        if first_frame_after_profile && is_method {
                            first_frame_after_profile = false;
                            let demangled = demangle(&name_str).to_string();
                            // Clean the demangled name
                            let cleaned = clean_function_name(&demangled);
                            maybe_method_name = extract_class_method(&cleaned);
                            // eprintln!("class_method name: {maybe_method_name:?}");
                        }

                        // Skip tokio::runtime functions
                        if name_str.starts_with("tokio::") {
                            continue;
                        }

                        // Demangle the symbol
                        let demangled = demangle(&name_str).to_string();
                        raw_frames.push(demangled);

                        // Check if we've reached the end condition
                        if name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                            is_within_target_range = false;
                            break;
                        }
                    }
                }
            }
        }

        // Register this function
        let fn_name = maybe_method_name.as_ref().map_or(name, Into::into);
        let desc_fn_name = if is_async {
            format!("async::{fn_name}")
        } else {
            fn_name.to_string()
        };
        // eprintln!("name={name}, is_method={is_method}, maybe_method_name={maybe_method_name:?}, fn_name={fn_name}");
        register_profiled_function(&fn_name, desc_fn_name);

        // Process the collected frames to collapse patterns and clean up
        let cleaned_stack = clean_stack_trace(raw_frames);
        // eprintln!("cleaned_stack={cleaned_stack:?}");

        // Filter to only profiled functions
        let mut path: Vec<String> = Vec::new();

        // Add self and ancestors that are profiled functions
        for fn_name_str in cleaned_stack {
            // println!("Function name: {fn_name_str}");

            let desc_fn_name = extract_class_method(&fn_name_str).map_or_else(
                || get_fn_desc_name(&fn_name_str),
                |class_method| {
                    get_reg_desc_name(&class_method).map_or_else(
                        || get_fn_desc_name(&fn_name_str),
                        |desc_fn_name| desc_fn_name,
                    )
                },
            );
            path.push(desc_fn_name);
        }

        // Reverse the path so it goes from root caller to current function
        path.reverse();

        // Create the profile with filtered path
        // let _id = NEXT_PROFILE_ID.fetch_add(1, Ordering::SeqCst);

        // In test mode with our test wrapper active, skip creating profile for #[profile] attribute
        #[cfg(test)]
        if is_test_mode_active() {
            // If this is from an attribute in a test, don't create a profile
            // Our safe wrapper will handle profiling instead
            return None;
        }

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

        Some(Self {
            // id,
            // name,
            profile_type,
            start: Some(Instant::now()),
            initial_memory,
            path,
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

        let stack = &self.path;

        if stack.is_empty() {
            return Err(ThagError::Profiling("Stack is empty".into()));
        }
        let stack_str = stack.join(";");
        let entry = format!("{stack_str} {micros}");

        let paths = ProfilePaths::get();
        Self::write_profile_event(&paths.time, TimeProfileFile::get(), &entry)
    }

    fn write_memory_event_with_op(&self, delta: usize, op: char) -> ThagResult<()> {
        if delta == 0 {
            // Keep this as it's a business logic check
            return Ok(());
        }

        let stack = &self.path;

        if stack.is_empty() {
            return Err(ThagError::Profiling("Stack is empty".into()));
        }
        let stack_str = stack.join(";");

        let entry = format!("{stack_str} {op}{delta}");

        let paths = ProfilePaths::get();
        Self::write_profile_event(&paths.memory, MemoryProfileFile::get(), &entry)
    }

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

fn get_fn_desc_name(fn_name_str: &String) -> String {
    extract_fn_only(fn_name_str).map_or_else(
        || fn_name_str.to_string(),
        |fn_only| get_reg_desc_name(&fn_only).map_or(fn_only, |desc_fn_name| desc_fn_name),
    )
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
    }
}

pub struct ProfileSection {
    profile: Option<Profile>,
}

impl ProfileSection {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(
                name.to_string(),
                get_global_profile_type(),
                false, // log_to_file
                false, // log_to_stdout
            ),
        }
    }

    pub fn end(self) {
        // Profile (if any) will be dropped here
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}

/// Register a function name with the profiling registry
///
/// # Panics
///
/// Panics if it finds the name "new", which shows that the inclusion of the
/// type in the method name is not working.
pub fn register_profiled_function(name: &str, desc_name: String) {
    #[cfg(debug_assertions)]
    assert!(name != "new");
    if let Ok(mut registry) = PROFILED_FUNCTIONS.lock() {
        registry.insert(name.to_string(), desc_name);
    }
}

// Check if a function is registered for profiling
pub fn is_profiled_function(name: &str) -> bool {
    PROFILED_FUNCTIONS
        .lock()
        .is_ok_and(|registry| registry.contains_key(name))
}

// Get the descriptive name of a profiledfunction
pub fn get_reg_desc_name(name: &str) -> Option<String> {
    PROFILED_FUNCTIONS
        .lock()
        .ok()
        .and_then(|registry| registry.get(name).cloned())
}

// Extract the class::method part from a fully qualified function name
fn extract_class_method(qualified_name: &str) -> Option<String> {
    // Split by :: and get the last two components
    // eprintln!("Extracting class::method from {}", qualified_name);
    let parts: Vec<&str> = qualified_name.split("::").collect();
    if parts.len() >= 2 {
        let class = parts[parts.len() - 2];
        let method = parts[parts.len() - 1];
        // eprintln!("Returning `{class}::{method}`");
        Some(format!("{class}::{method}"))
    } else {
        eprintln!("Returning `None`");
        None
    }
}

// Extract just the method name from a fully qualified function name
fn extract_fn_only(qualified_name: &str) -> Option<String> {
    // Split by :: and get the last component
    qualified_name.split("::").last().map(ToString::to_string)
}

fn clean_stack_trace(raw_frames: Vec<String>) -> Vec<String> {
    // First, filter out standard library infrastructure we don't care about
    let filtered_frames: Vec<String> = raw_frames
        .into_iter()
        .filter(|frame| {
            !frame.contains("core::ops::function::FnOnce::call_once")
                && !frame.contains("std::sys::backtrace::__rust_begin_short_backtrace")
                && !frame.contains("std::rt::lang_start")
                && !frame.contains("std::panicking")
        })
        .collect();

    // These are patterns we want to remove from the stack
    let scaffolding_patterns: Vec<&str> = vec![
        "::main::",
        "::poll::",
        "::poll_next_unpin",
        "alloc::",
        "core::",
        "<F as core::future::future::Future>::poll",
        "FuturesOrdered<Fut>",
        "FuturesUnordered<Fut>",
        "Profile::new",
        "ProfiledFuture",
        "{{closure}}::{{closure}}",
    ];

    // Create a new cleaned stack, filtering out scaffolding
    let mut cleaned_frames = Vec::new();
    let mut i = 0;
    let mut already_seen = HashSet::new();
    let mut seen_main = false;

    while i < filtered_frames.len() {
        let current_frame = &filtered_frames[i];

        // Check if this is scaffolding we want to skip
        let is_scaffolding = scaffolding_patterns
            .iter()
            .any(|pattern| current_frame.contains(pattern));

        if is_scaffolding {
            i += 1;
            continue;
        }

        // Clean the function name
        let clean_name = clean_function_name(current_frame);

        // Handle main function special case
        if clean_name.ends_with("::main") || clean_name == "main" {
            if !seen_main {
                cleaned_frames.push("main".to_string());
                seen_main = true;
            }
            i += 1;
            continue;
        }

        // Skip duplicate function calls (helps with the {{closure}} pattern)
        if already_seen.contains(&clean_name) {
            i += 1;
            continue;
        }

        already_seen.insert(clean_name.clone());
        cleaned_frames.push(clean_name);
        i += 1;
    }

    cleaned_frames
}

fn clean_function_name(demangled: &str) -> String {
    // Remove hash suffixes and closure markers
    let mut clean_name = demangled.to_string();

    // Find and remove hash suffixes (::h followed by hex digits)
    if let Some(hash_pos) = clean_name.find("::h") {
        if clean_name[hash_pos + 3..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        {
            clean_name = clean_name[..hash_pos].to_string();
        }
    }

    // Remove closure markers
    clean_name = clean_name.replace("{{closure}}", "");

    // Clean up any double colons that might be left
    while clean_name.contains("::::") {
        clean_name = clean_name.replace("::::", "::");
    }
    if clean_name.ends_with("::") {
        clean_name = clean_name[..clean_name.len() - 2].to_string();
    }

    clean_name
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
    ($name:expr, $is_async:expr) => {
        let _profile = $crate::profiling::Profile::new(
            $name.to_string(),
            $crate::profiling::get_global_profile_type(),
            $is_async,
            false,
        );
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
    ($name:expr) => {{
        let section = $crate::profiling::ProfileSection::new($name);
        section // Return the section
    }};
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
static TEST_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
/// Checks if we're in test mode to avoid duplicate profiling
/// This is used by the Profile::new function to avoid creating duplicate profiles
#[inline]
pub fn is_test_mode_active() -> bool {
    TEST_MODE_ACTIVE.load(Ordering::SeqCst)
}

#[cfg(test)]
/// Sets up profiling for a test in a safe manner by first clearing the stack
pub fn safely_setup_profiling_for_test() -> ThagResult<()> {
    // Set test mode active to prevent #[profile] from creating duplicate entries
    TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

    // Then enable profiling
    enable_profiling(true, ProfileType::Time)
}

#[cfg(test)]
/// Safely cleans up profiling after a test
pub fn safely_cleanup_profiling_after_test() -> ThagResult<()> {
    // First disable profiling
    let result = enable_profiling(false, ProfileType::Time);

    // Finally reset test mode flag
    TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::panic;
    use std::time::Duration;
    use thag_proc_macros::profile;

    struct TestGuard;

    impl Drop for TestGuard {
        fn drop(&mut self) {
            // Since set_profiling_enabled is now private, we need to use enable_profiling
            let _ = enable_profiling(false, ProfileType::Time);
        }
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() + panic::UnwindSafe,
    {
        // Enable profiling using the proper interface
        let _ = enable_profiling(true, ProfileType::Time);
        // Make sure test mode is off for profiling tests
        // This allows Profile::new to create profiles directly
        TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);

        // Create guard that will clean up even if test panics
        let _guard = TestGuard;

        // Run the test, catching any panics to ensure our guard runs
        let result = panic::catch_unwind(test);

        // Re-throw any panic after our guard has cleaned up
        if let Err(e) = result {
            panic::resume_unwind(e);
        }
    }

    // This function uses the #[profile] attribute which will invoke generate_async_wrapper
    #[profile]
    async fn run_async_profiled() -> u32 {
        // Simulate some async work
        tokio::time::sleep(Duration::from_millis(50)).await;
        42
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
