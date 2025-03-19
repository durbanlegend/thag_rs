use crate::task_allocator::{create_memory_task, TaskGuard, TaskMemoryContext};
use crate::{lazy_static_var, static_lazy, ProfileError};
use chrono::Local;
use once_cell::sync::Lazy;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    env,
    fs::File,
    io::BufWriter,
    ops::Deref,
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Mutex,
    },
    time::Instant,
};

#[cfg(feature = "time_profiling")]
use std::sync::OnceLock;

#[cfg(feature = "full_profiling")]
use crate::task_allocator::create_memory_guard;

// #[cfg(feature = "full_profiling")]
// use crate::task_allocator::get_allocator;

// #[cfg(feature = "full_profiling")]
// #[global_allocator]
// static ALLOC: re_memory::accounting_allocator::AccountingAllocator<std::alloc::System> =
//     re_memory::accounting_allocator::AccountingAllocator::new(std::alloc::System);

use backtrace::Backtrace;

#[cfg(feature = "time_profiling")]
use crate::ProfileResult;

// #[cfg(feature = "time_profiling")]
use rustc_demangle::demangle;

#[cfg(feature = "time_profiling")]
use std::{
    convert::Into,
    fs::OpenOptions,
    io::Write,
    // path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU64},
    time::SystemTime,
};

// Single atomic for runtime profiling state
#[cfg(feature = "time_profiling")]
static PROFILING_STATE: AtomicBool = AtomicBool::new(false);

// Mutex to protect profiling state changes
#[cfg(feature = "time_profiling")]
static PROFILING_MUTEX: Mutex<()> = Mutex::new(());

// Compile-time feature check - always use the runtime state in tests
#[cfg(all(feature = "time_profiling", not(test)))]
const PROFILING_FEATURE: bool = true;

#[allow(dead_code)]
#[cfg(all(feature = "time_profiling", test))]
const PROFILING_FEATURE: bool = false;

static PROFILE_TYPE: AtomicU8 = AtomicU8::new(0); // 0 = None, 1 = Time, 2 = Memory, 3 = Both

static_lazy! {
    ProfileConfig: ProfileConfiguration = {
        let env_profile = env::var("THAG_PROFILE")
            .ok()
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(0);

        let env_profile_type = env::var("THAG_PROFILE_TYPE")
            .ok()
            .map(|v| v.to_lowercase())
            .and_then(|v| ProfileType::from_str(&v))
            .unwrap_or({
                if cfg!(feature = "full_profiling") {
                    ProfileType::Both
                } else {
                    ProfileType::Time
                }
            });

        let env_profile_dir = env::var("THAG_PROFILE_DIR").ok();

        ProfileConfiguration {
            enabled: env_profile > 0,
            profile_type: env_profile_type,
            output_dir: env_profile_dir,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProfileConfiguration {
    enabled: bool,
    profile_type: ProfileType,
    output_dir: Option<String>,
}

// Global registry of profiled functions
static PROFILED_FUNCTIONS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// #[cfg(feature = "time_profiling")]
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
// #[cfg(feature = "time_profiling")]
static_lazy! {
    TimeProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

// #[cfg(feature = "time_profiling")]
static_lazy! {
    MemoryProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

#[cfg(feature = "time_profiling")]
static START_TIME: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
#[allow(dead_code)]
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
/// Returns a `ProfileError` if the mutex lock fails
#[cfg(feature = "time_profiling")]
fn reset_profile_file(file: &Mutex<Option<BufWriter<File>>>, file_type: &str) -> ProfileResult<()> {
    *file
        .lock()
        .map_err(|_| ProfileError::General(format!("Failed to lock {file_type} profile file")))? =
        None;
    Ok(())
}

#[cfg(feature = "time_profiling")]
fn get_time_path() -> ProfileResult<&'static str> {
    struct TimePathHolder;
    impl TimePathHolder {
        fn get() -> ProfileResult<&'static str> {
            static PATH_RESULT: OnceLock<Result<String, ProfileError>> = OnceLock::new();

            let result = PATH_RESULT.get_or_init(|| {
                let paths = ProfilePaths::get();
                let config = ProfileConfig::get();

                let path = if let Some(dir) = &config.output_dir {
                    let dir_path = PathBuf::from(dir);
                    if !dir_path.exists() {
                        match std::fs::create_dir_all(&dir_path) {
                            Ok(()) => {}
                            Err(e) => return Err(ProfileError::from(e)),
                        }
                    }

                    let time_file =
                        dir_path.join(paths.time.split('/').last().unwrap_or(&paths.time));
                    time_file.to_string_lossy().to_string()
                } else {
                    paths.time.clone()
                };

                Ok(path)
            });

            // Convert to static reference
            match result {
                Ok(s) => Ok(Box::leak(s.clone().into_boxed_str())),
                Err(e) => Err(e.clone()),
            }
        }
    }

    TimePathHolder::get()
}

#[cfg(feature = "time_profiling")]
fn get_memory_path() -> ProfileResult<&'static str> {
    struct MemoryPathHolder;
    impl MemoryPathHolder {
        fn get() -> ProfileResult<&'static str> {
            static PATH_RESULT: OnceLock<Result<String, ProfileError>> = OnceLock::new();

            let result = PATH_RESULT.get_or_init(|| {
                let paths = ProfilePaths::get();
                let config = ProfileConfig::get();

                let path = if let Some(dir) = &config.output_dir {
                    let dir_path = PathBuf::from(dir);
                    if !dir_path.exists() {
                        match std::fs::create_dir_all(&dir_path) {
                            Ok(()) => {}
                            Err(e) => return Err(ProfileError::from(e)),
                        }
                    }

                    let memory_file =
                        dir_path.join(paths.memory.split('/').last().unwrap_or(&paths.memory));

                    memory_file.to_string_lossy().to_string()
                } else {
                    paths.memory.clone()
                };

                Ok(path)
            });

            // Convert to static reference
            match result {
                Ok(s) => Ok(Box::leak(s.clone().into_boxed_str())),
                Err(e) => Err(e.clone()),
            }
        }
    }

    MemoryPathHolder::get()
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
/// Returns a `ProfileError` if any file operations fail
#[cfg(feature = "time_profiling")]
fn initialize_profile_files(profile_type: ProfileType) -> ProfileResult<()> {
    match profile_type {
        ProfileType::Time => {
            let time_path = get_time_path()?;
            TimeProfileFile::init();
            reset_profile_file(TimeProfileFile::get(), "time")?;
            initialize_profile_file(time_path, "Time Profile")?;
            eprintln!("Time profile will be written to {time_path}");
        }
        ProfileType::Memory => {
            let memory_path = get_memory_path()?;
            MemoryProfileFile::init();
            reset_profile_file(MemoryProfileFile::get(), "memory")?;
            initialize_profile_file(memory_path, "Memory Profile")?;
            eprintln!("Memory profile will be written to {memory_path}");
        }
        ProfileType::Both => {
            let time_path = get_time_path()?;
            let memory_path = get_memory_path()?;
            // Initialize all files
            TimeProfileFile::init();
            MemoryProfileFile::init();

            // Reset all files
            reset_profile_file(TimeProfileFile::get(), "time")?;
            reset_profile_file(MemoryProfileFile::get(), "memory")?;

            // Initialize all files with headers
            initialize_profile_file(time_path, "Time Profile")?;
            initialize_profile_file(memory_path, "Memory Profile")?;

            eprintln!("Time profile will be written to {time_path}");
            eprintln!("Memory profile will be written to {memory_path}");
        }
    }
    Ok(())
}

/// Returns the global profile type.
///
/// # Panics
///
/// Panics if `enable_profiling` fails.
// Modify get_global_profile_type to use the config:
pub fn get_global_profile_type() -> ProfileType {
    // eprintln!("profile_type={profile_type:?}");

    lazy_static_var!(ProfileType, deref, {
        let profile_type = match PROFILE_TYPE.load(Ordering::SeqCst) {
            2 => ProfileType::Memory,
            3 => ProfileType::Both,
            _ => {
                // Then check environment variables
                ProfileConfig::get().profile_type
            }
        };
        enable_profiling(true, profile_type).expect("Failed to enable profiling");
        if profile_type == ProfileType::Memory {
            eprintln!("Memory profiling enabled");
        } else if profile_type == ProfileType::Both {
            eprintln!("Both time and memory profiling enabled");
        }
        profile_type
    })
}

#[cfg(feature = "time_profiling")]
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
/// Returns a `ProfileError` if:
/// - Time value conversion fails
/// - File operations fail
/// - Mutex operations fail
#[cfg(feature = "time_profiling")]
pub fn enable_profiling(enabled: bool, profile_type: ProfileType) -> ProfileResult<()> {
    // Acquire the mutex to ensure only one thread can enable/disable profiling at a time
    let _guard = PROFILING_MUTEX
        .lock()
        .map_err(|_| ProfileError::General("Failed to acquire profiling mutex".into()))?;

    // Check if the operation is a no-op due to environment settings
    let config = ProfileConfig::get();
    if !enabled && config.enabled {
        // If trying to disable but env var says enabled, log a warning but continue
        // (environment settings take precedence)
        println!(
            "Warning: Attempt to disable profiling overridden by THAG_PROFILE environment variable"
        );
        return Ok(());
    }

    if enabled {
        // Respect environment variable for profile type if one wasn't explicitly provided
        let final_profile_type = if profile_type == ProfileType::Time
            && matches!(config.profile_type, ProfileType::Memory | ProfileType::Both)
        {
            config.profile_type
        } else {
            profile_type
        };

        set_profile_type(final_profile_type);

        let Ok(now) = u64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros(),
        ) else {
            return Err(ProfileError::General("Time value too large".into()));
        };
        START_TIME.store(now, Ordering::SeqCst);

        initialize_profile_files(final_profile_type)?;
    }

    // Whether enabling or disabling, set the state
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
    Ok(())
}

/// No-op version when profiling feature is disabled.
///
/// # Errors
/// None
#[cfg(not(feature = "time_profiling"))]
pub const fn enable_profiling(
    _enabled: bool,
    _profile_type: ProfileType,
) -> Result<(), ProfileError> {
    // No-op implementation
    Ok(())
}

/// Disable profiling and reset the profiling stack.
#[cfg(feature = "time_profiling")]
pub fn disable_profiling() {
    let _ = enable_profiling(false, ProfileType::Both);
}

#[cfg(not(feature = "time_profiling"))]
pub const fn disable_profiling() {
    // No-op implementation
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
/// Returns a `ProfileError` if file creation or writing fails
#[cfg(feature = "time_profiling")]
fn initialize_profile_file(path: &str, profile_type: &str) -> ProfileResult<()> {
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
#[cfg(feature = "time_profiling")]
pub fn is_profiling_enabled() -> bool {
    // eprintln!(
    //     r#"cfg!(test)={}, cfg(feature = "time_profiling")={}"#,
    //     cfg!(test),
    //     cfg!(feature = "time_profiling")
    // );
    // In test mode, only use the runtime state to allow enable/disable testing
    #[cfg(test)]
    let enabled = PROFILING_STATE.load(Ordering::SeqCst);

    // In normal operation, use both feature flag and runtime state
    #[cfg(not(test))]
    let enabled = PROFILING_FEATURE || PROFILING_STATE.load(Ordering::SeqCst);

    enabled
}

/// Checks if profiling state is currently enabled.
///
/// This is used in integration testing to determine whether profiling
/// operations should be performed. We use this function because we don't
/// want to check `PROFILING_FEATURE`, which is always true when testing with
/// feature=profiling. It's atomic and thread-safe.
///
/// # Returns
/// `true` if profiling state is enabled, `false` otherwise
#[inline(always)]
#[allow(clippy::inline_always)]
#[cfg(feature = "time_profiling")]
pub fn is_profiling_state_enabled() -> bool {
    // eprintln!(
    //     r#"cfg!(test)={}, cfg(feature = "time_profiling")={}"#,
    //     cfg!(test),
    //     cfg!(feature = "time_profiling")
    // );
    PROFILING_STATE.load(Ordering::SeqCst)
}

#[cfg(not(feature = "time_profiling"))]
#[must_use]
pub const fn is_profiling_enabled() -> bool {
    false
}

#[cfg(not(feature = "time_profiling"))]
#[must_use]
pub const fn is_profiling_state_enabled() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct Profile {
    start: Option<Instant>,
    profile_type: ProfileType,
    path: Vec<String>,           // Full call stack (only profiled functions)
    custom_name: Option<String>, // Custom section name when provided via profile!("name") macro
    registered_name: String,
    #[cfg(feature = "full_profiling")]
    memory_task: Option<TaskMemoryContext>,
    #[cfg(feature = "full_profiling")]
    memory_guard: Option<TaskGuard>,
}

impl Profile {
    /// Creates a new `Profile` to profile a section of code.
    ///
    /// This will track execution time by default. When the `full_profiling` feature
    /// is enabled, it will also track memory usage if requested via `ProfileType`.
    ///
    /// # Examples
    ///
    /// ```
    /// use thag_profiler::{Profile, ProfileType};
    ///
    /// // Time profiling only
    /// {
    ///     let _p = Profile::new("time_only_function", ProfileType::Time);
    ///     // Code to profile...
    /// }
    ///
    /// // With memory profiling (requires `full_profiling` feature)
    /// #[cfg(feature = "full_profiling")]
    /// {
    ///     let _p = Profile::new("memory_tracking_function", ProfileType::Memory);
    ///     // Code to profile with memory tracking...
    /// }
    /// ```
    /// # Panics
    ///
    /// Panics if stack validation fails.
    #[allow(clippy::inline_always, clippy::too_many_lines, unused_variables)]
    pub fn new(
        name: Option<&str>,
        requested_type: ProfileType,
        is_async: bool,
        is_method: bool,
    ) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        // eprintln!("Current function/section: {name:?}, requested_type: {requested_type:?}, full_profiling?: {}", cfg!(feature = "full_profiling"));

        // Get the current backtrace
        let mut current_backtrace = Backtrace::new_unresolved();
        current_backtrace.resolve();

        // if name == Some("new") {
        //     eprintln!("Current backtrace: {current_backtrace:?}");
        // }

        let mut is_within_target_range = false;

        // First, collect all relevant frames
        let mut raw_frames: Vec<String> = Vec::new();

        // If this is a method, we'll capture the calling class
        let mut maybe_cleaned_name: Option<String> = None;
        let maybe_qualified_name: Option<String> = None;
        let maybe_bare_name: Option<String> = None;
        let mut first_frame_after_profile = false;

        for frame in Backtrace::frames(&current_backtrace) {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();
                    // eprintln!("Symbol name: {name_str}");

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("Profile::new") {
                        is_within_target_range = true;
                        first_frame_after_profile = true;
                        continue;
                    }

                    // Collect frames within our target range
                    if is_within_target_range {
                        // If this is the first frame after Profile::new and it's a method,
                        // then we can extract the class name
                        if first_frame_after_profile {
                            first_frame_after_profile = false;
                            let demangled = demangle(&name_str);
                            // Clean the demangled name
                            let cleaned_name = clean_function_name(demangled.to_string().as_str());
                            // eprintln!("name_str: {name_str}; cleaned name: {cleaned_name:?}");
                            maybe_cleaned_name = Some(cleaned_name);
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
        let fn_name = maybe_cleaned_name
            .as_ref()
            .map_or_else(|| name.unwrap_or("unknown"), Deref::deref);
        let desc_fn_name = if is_async {
            format!("async::{fn_name}")
        } else {
            fn_name.to_string()
        };
        // eprintln!("fn_name={fn_name}, is_method={is_method}, maybe_method_name={maybe_method_name:?}, maybe_function_name={maybe_function_name:?}, desc_fn_name={desc_fn_name}");
        register_profiled_function(fn_name, desc_fn_name);

        // Process the collected frames to collapse patterns and clean up
        let cleaned_stack = clean_stack_trace(raw_frames);
        // eprintln!("cleaned_stack={cleaned_stack:?}");

        // Filter to only profiled functions
        let mut path: Vec<String> = Vec::new();

        // Add self and ancestors that are profiled functions
        for fn_name_str in cleaned_stack {
            if let Some(name) = get_reg_desc_name(&fn_name_str) {
                // eprintln!("Registered desc name: {}", name);
                path.push(name);
                continue;
            }
            // }
            let key = get_fn_desc_name(&fn_name_str);
            // eprintln!("Function desc name: {}", key);
            if let Some(name) = get_reg_desc_name(&key) {
                // eprintln!("Registered desc name: {}", name);
                path.push(name);
            }
        }

        // Reverse the path so it goes from root caller to current function
        path.reverse();

        // In test mode with our test wrapper active, skip creating profile for #[profiled] attribute
        #[cfg(test)]
        if is_test_mode_active() {
            // If this is from an attribute in a test, don't create a profile
            // Our safe wrapper will handle profiling instead
            return None;
        }

        let global_type = match PROFILE_TYPE.load(Ordering::SeqCst) {
            2 => ProfileType::Memory,
            3 => ProfileType::Both,
            _ => {
                if cfg!(feature = "full_profiling") {
                    ProfileType::Both
                } else {
                    ProfileType::Time
                }
            } // default
        };

        // // Use the more comprehensive of the two types
        // let profile_type = match (requested_type, global_type) {
        //     (ProfileType::Both, _) | (_, ProfileType::Both) => ProfileType::Both,
        //     (ProfileType::Memory, _) | (_, ProfileType::Memory) => ProfileType::Memory,
        //     _ => ProfileType::Time,
        // };

        // Try allowing overrides
        let profile_type = requested_type;

        #[cfg(not(feature = "full_profiling"))]
        if matches!(profile_type, ProfileType::Memory | ProfileType::Both) {
            eprintln!("Memory profiling requested but the 'full_profiling' feature is not enabled. Only time will be profiled.");
        }

        // Determine if we should keep the custom name
        let custom_name = name.and_then(|n| {
            if n.is_empty() {
                return None; // Empty names are never useful
            }

            // For methods (with Class::method syntax), check if custom name matches just the method part
            if let Some(method_name) = &maybe_qualified_name {
                // Extract the method part after the last ::
                if let Some(pos) = method_name.rfind("::") {
                    let method_part = &method_name[(pos + 2)..];
                    // Don't use custom name if it exactly matches just the method part
                    if n == method_part {
                        return None;
                    }
                }
            }
            // For regular functions, check if custom name matches the entire function name
            else if let Some(function_name) = &maybe_bare_name {
                if n == function_name {
                    return None;
                }
            }
            // If we get here, the custom name adds value, so keep it
            Some(n.to_string())
        });

        // Debug output can be turned back on if needed for troubleshooting
        // println!(
        //     "DEBUG: Profile::new with name='{name}', fn_name='{fn_name}', custom_name={custom_name:?}, requested_type={requested_type:?}, profile_type={profile_type:?}, initial_memory={initial_memory:?}"
        // );

        // Create a basic profile structure that works for all configurations
        #[cfg(not(feature = "full_profiling"))]
        let profile = Self {
            profile_type,
            start: Some(Instant::now()),
            path,
            custom_name,
            registered_name: fn_name.to_string(),
        };

        // For full profiling, we need to handle memory task and guard creation
        #[cfg(feature = "full_profiling")]
        let profile = {
            // First, determine if we need memory profiling
            if matches!(profile_type, ProfileType::Memory | ProfileType::Both) {
                // Create a task to track memory usage
                let task = create_memory_task();

                // Create an owned TaskGuard directly from the task ID
                // This avoids the 'static lifetime requirement
                let guard_result = create_memory_guard(task.id());
                match &guard_result {
                    Ok(_) => (),
                    Err(e) => println!("Failed to create memory guard: {e}"),
                }

                // Create the profile with necessary components
                Self {
                    profile_type,
                    start: Some(Instant::now()),
                    path,
                    custom_name,
                    registered_name: fn_name.to_string(),
                    memory_task: Some(task),
                    memory_guard: guard_result.ok(),
                }
            } else {
                // No memory profiling needed
                Self {
                    profile_type,
                    start: Some(Instant::now()),
                    path,
                    custom_name,
                    registered_name: fn_name.to_string(),
                    memory_task: None,
                    memory_guard: None,
                }
            }
        };

        Some(profile)
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
    /// Returns a `ProfileError` if:
    /// * The mutex lock fails
    /// * File operations fail
    /// * Writing to the file fails
    #[cfg(feature = "time_profiling")]
    fn write_profile_event(
        path: &str,
        file: &Mutex<Option<BufWriter<File>>>,
        entry: &str,
    ) -> ProfileResult<()> {
        let mut guard = file
            .lock()
            .map_err(|_| ProfileError::General("Failed to lock profile file".into()))?;

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
    /// Returns a `ProfileError` if writing to the profile file fails
    #[cfg(feature = "time_profiling")]
    fn write_time_event(&self, duration: std::time::Duration) -> ProfileResult<()> {
        // Profile must exist and profiling must be enabled if we got here
        // Only keep the business logic checks

        let micros = duration.as_micros();
        if micros == 0 {
            println!(
                "DEBUG: Not writing time event for stack: {:?} due to zero duration",
                self.path
            );
            return Ok(());
        }

        let stack = &self.path;

        if stack.is_empty() {
            println!("DEBUG: Stack is empty for {:?}", self.custom_name);
            return Err(ProfileError::General("Stack is empty".into()));
        }

        // println!("DEBUG: write_time_event for stack: {:?}", stack);

        // Create a copy of the stack for our modified output
        let mut stack_with_custom_name = stack.clone();

        // Add our custom section name to the end of the stack path if present
        if let Some(name) = &self.custom_name {
            // println!("DEBUG: Adding custom name '{}' to time stack", name);

            // If the stack is not empty, get the last function name
            if let Some(last_fn) = stack_with_custom_name.last_mut() {
                // Append the custom name to the last function name
                *last_fn = format!("{last_fn}:{name}");
                // println!("DEBUG: Modified stack entry to '{}'", last_fn);
            }
        }

        let stack_str = stack_with_custom_name.join(";");
        let entry = format!("{stack_str} {micros}");

        // let paths = ProfilePaths::get();
        let time_path = get_time_path()?;
        Self::write_profile_event(time_path, TimeProfileFile::get(), &entry)
    }

    // TODO remove op as redundant
    #[cfg(feature = "full_profiling")]
    fn write_memory_event_with_op(&self, delta: usize, op: char) -> ProfileResult<()> {
        if delta == 0 {
            // Keep this as it's a business logic check
            println!(
                "DEBUG: Not writing memory event for stack: {:?} due to zero delta",
                self.path
            );
            return Ok(());
        }

        let stack = &self.path;

        if stack.is_empty() {
            return Err(ProfileError::General("Stack is empty".into()));
        }

        // println!("DEBUG: write_memory_event for stack: {:?}", stack);

        // Create a copy of the stack for our modified output
        let mut stack_with_custom_name = stack.clone();

        // Add our custom section name to the end of the stack path if present
        if let Some(name) = &self.custom_name {
            // println!("DEBUG: Adding custom name '{}' to memory stack", name);

            // If the stack is not empty, get the last function name
            if let Some(last_fn) = stack_with_custom_name.last_mut() {
                // Append the custom name to the last function name
                *last_fn = format!("{last_fn}:{name}");
                // println!("DEBUG: Modified stack entry to '{}'", last_fn);
            }
        }
        let stack_str = stack_with_custom_name.join(";");
        let entry = format!("{stack_str} {op}{delta}");

        // let paths = ProfilePaths::get();
        let memory_path = get_memory_path()?;
        Self::write_profile_event(memory_path, MemoryProfileFile::get(), &entry)
    }

    #[cfg(feature = "full_profiling")]
    fn record_memory_change(&self, delta: usize) -> ProfileResult<()> {
        if delta == 0 {
            return Ok(());
        }

        // Record allocation
        self.write_memory_event_with_op(delta, '+')?;

        // // Record corresponding deallocation
        // // Store both events atomically to maintain pairing
        // self.write_memory_event_with_op(delta, '-')?;

        Ok(())
    }
}

// Global thread-safe BTreeSet
static GLOBAL_CALL_STACK_ENTRIES: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));

/// Prints all entries in the global `BTreeSet`.
/// Entries are printed in sorted order (alphabetically).
pub fn print_all_call_stack_entries() {
    match GLOBAL_CALL_STACK_ENTRIES.lock() {
        Ok(parts) => {
            println!("All entries in the global set (sorted):");
            if parts.is_empty() {
                println!("  (empty set)");
            } else {
                for part in parts.iter() {
                    println!("  {part}");
                }
            }
            println!("Total entries: {}", parts.len());
        }
        Err(e) => {
            eprintln!("Error accessing the global set: {e:?}");
        }
    }
}

fn get_fn_desc_name(fn_name_str: &String) -> String {
    // extract_fn_only(fn_name_str).map_or_else(|| fn_name_str.to_string(), |fn_only| fn_only)
    extract_fn_only(fn_name_str).unwrap_or_else(|| fn_name_str.to_string())
}

#[cfg(feature = "time_profiling")]
impl Drop for Profile {
    fn drop(&mut self) {
        // println!("In drop for Profile {:?}", self);
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
        #[cfg(feature = "full_profiling")]
        if matches!(self.profile_type, ProfileType::Memory | ProfileType::Both) {
            eprintln!(
                "In drop for Profile with memory profiling: {}",
                self.registered_name
            );
            // First drop the guard to exit the task context
            self.memory_guard = None;
            // Now get memory usage from our task
            if let Some(ref task) = self.memory_task {
                if let Some(memory_usage) = task.memory_usage() {
                    // eprintln!("memory_usage={memory_usage}");
                    if memory_usage > 0 {
                        let _ = self.record_memory_change(memory_usage);
                    }
                }
            }
        }
    }
}

// Helper function to check if a backtrace contains any of the specified patterns
#[cfg(feature = "full_profiling")]
#[allow(dead_code)]
fn backtrace_contains_any(backtrace: &str, patterns: &[&str]) -> bool {
    // Split the backtrace into lines for easier processing
    let lines = backtrace.lines();

    // Check if any line contains any of our patterns
    for line in lines {
        for &pattern in patterns {
            if line.contains(pattern) {
                // eprintln!("Backtrace contains pattern: {pattern}, line: {line}");
                return true;
            }
        }
    }

    false
}

// More sophisticated helper function to check if a backtrace contains any of the specified patterns
// #[cfg(feature = "full_profiling")]
// fn backtrace_contains_any_sophisticated(backtrace: &str, patterns: &[&str]) -> bool {
//     let lines = backtrace.lines();

//     for line in lines {
//         // Skip lines that don't look like function references
//         if !line.contains(" at ") && !line.trim().starts_with(|c: char| c.is_ascii_digit() || c == ' ') {
//             continue;
//         }

//         for &pattern in patterns {
//             if line.contains(pattern) {
//                 return true;
//             }
//         }
//     }

//     false
// }
#[cfg(feature = "time_profiling")]
pub struct ProfileSection {
    pub profile: Option<Profile>,
}

#[cfg(feature = "time_profiling")]
impl ProfileSection {
    #[must_use]
    pub fn new(name: Option<&str>) -> Self {
        let profile_type = get_global_profile_type();
        // eprintln!("profile_type={profile_type:?}");
        Self {
            profile: Profile::new(
                name,
                profile_type,
                false, // is_async
                false, // is_method
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

// Dummy implementation when profiling is disabled
#[cfg(not(feature = "time_profiling"))]
pub struct ProfileSection;

#[cfg(not(feature = "time_profiling"))]
impl ProfileSection {
    #[must_use]
    pub const fn new(_name: Option<&str>) -> Self {
        Self
    }
    pub const fn end(self) {}
    #[must_use]
    pub const fn is_active(&self) -> bool {
        false
    }
}

// This is just for clarity - removing the PhantomData marker should be enough
unsafe impl Send for Profile {}
unsafe impl Send for ProfileSection {}

/// Register a function name with the profiling registry
///
/// # Panics
///
/// Panics if it finds the name "new", which shows that the inclusion of the
/// type in the method name is not working.
pub fn register_profiled_function(name: &str, desc_name: String) {
    #[cfg(all(debug_assertions, not(test)))]
    assert!(
        name != "new",
        "Logic error: `new` is not an accepted function name on its own. It must be qualified with the type name: `<Type>::new`. desc_name={desc_name}"
    );
    if let Ok(mut registry) = PROFILED_FUNCTIONS.lock() {
        // eprintln!("Registering function: {name}::{desc_name}",);
        registry.insert(name.to_string(), desc_name);
    }
}

// Check if a function is registered for profiling
pub fn is_profiled_function(name: &str) -> bool {
    PROFILED_FUNCTIONS
        .lock()
        .is_ok_and(|registry| registry.contains_key(name))
}

// Get the descriptive name of a profiled function
pub fn get_reg_desc_name(name: &str) -> Option<String> {
    PROFILED_FUNCTIONS
        .lock()
        .ok()
        .and_then(|registry| registry.get(name).cloned())
}

// Extract just the base function name from a fully qualified function name
// #[cfg(feature = "time_profiling")]
fn extract_fn_only(qualified_name: &str) -> Option<String> {
    // Split by :: and get the last component
    qualified_name.split("::").last().map(ToString::to_string)
}

// #[cfg(feature = "time_profiling")]
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
        // "{{closure}}::{{closure}}",
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

// #[cfg(feature = "time_profiling")]
fn clean_function_name(demangled: &str) -> String {
    // Remove hash suffixes and closure markers
    let mut clean_name = demangled.to_string();

    // Find and remove hash suffixes (::h followed by hex digits)
    // from the last path segment
    if let Some(hash_pos) = clean_name.rfind("::h") {
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

/// Profile a section of code with customizable options
///
/// # Examples
///
/// ```
/// // Basic usage (profiles time, sync function)
/// let section = profile!("expensive_operation");
/// // ... code to profile
/// section.end();
///
/// // With explicit type (profile memory)
/// let section = profile!("allocation_heavy", memory);
///
/// // Method profiling
/// let section = profile!(method);
///
/// // Async method with explicit type
/// let section = profile!(method, both, async);
/// ```
#[macro_export]
#[cfg(feature = "time_profiling")]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Time, false, false)
    };

    // profile!(name, type)
    ($name:expr, time) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Time, false, false)
    };
    ($name:expr, memory) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Memory, false, false)
    };
    ($name:expr, both) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Both, false, false)
    };

    // profile!(name, async)
    ($name:expr, async) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Time, true, false)
    };

    // profile!(method) - no custom name
    (method) => {
        $crate::profile_internal!(None, $crate::ProfileType::Time, false, true)
    };

    // profile!(method, type) - no custom name
    (method, time) => {
        $crate::profile_internal!(None, $crate::ProfileType::Time, false, true)
    };
    (method, memory) => {
        $crate::profile_internal!(None, $crate::ProfileType::Memory, false, true)
    };
    (method, both) => {
        $crate::profile_internal!(None, $crate::ProfileType::Both, false, true)
    };

    // profile!(method, async) - no custom name
    (method, async) => {
        $crate::profile_internal!(None, $crate::ProfileType::Time, true, true)
    };

    // profile!(method, type, async) - no custom name
    (method, time, async) => {
        $crate::profile_internal!(None, $crate::ProfileType::Time, true, true)
    };
    (method, memory, async) => {
        $crate::profile_internal!(None, $crate::ProfileType::Memory, true, true)
    };
    (method, both, async) => {
        $crate::profile_internal!(None, $crate::ProfileType::Both, true, true)
    };

    // profile!(name, type, async)
    ($name:expr, time, async) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Time, true, false)
    };
    ($name:expr, memory, async) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Memory, true, false)
    };
    ($name:expr, both, async) => {
        $crate::profile_internal!(Some($name), $crate::ProfileType::Both, true, false)
    };
}

// No-op implementation for when profiling is disabled
#[cfg(not(feature = "time_profiling"))]
#[macro_export]
macro_rules! profile {
    // The implementations are all identical for the no-op version
    // Basic variants
    ($name:expr) => {{
        $crate::ProfileSection {}
    }};

    // profile!(name, type)
    ($name:expr, time) => {{
        $crate::ProfileSection {}
    }};
    ($name:expr, memory) => {{
        $crate::ProfileSection {}
    }};
    ($name:expr, both) => {{
        $crate::ProfileSection {}
    }};

    // profile!(name, async)
    ($name:expr, async) => {{
        $crate::ProfileSection {}
    }};

    // profile!(method)
    (method) => {{
        $crate::ProfileSection {}
    }};

    // profile!(method, type)
    (method, time) => {{
        $crate::ProfileSection {}
    }};
    (method, memory) => {{
        $crate::ProfileSection {}
    }};
    (method, both) => {{
        $crate::ProfileSection {}
    }};

    // profile!(method, async)
    (method, async) => {{
        $crate::ProfileSection {}
    }};

    // profile!(method, type, async)
    (method, time, async) => {{
        $crate::ProfileSection {}
    }};
    (method, memory, async) => {{
        $crate::ProfileSection {}
    }};
    (method, both, async) => {{
        $crate::ProfileSection {}
    }};

    // profile!(name, type, async)
    ($name:expr, time, async) => {{
        $crate::ProfileSection {}
    }};
    ($name:expr, memory, async) => {{
        $crate::ProfileSection {}
    }};
    ($name:expr, both, async) => {{
        $crate::ProfileSection {}
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! profile_internal {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        // Within the crate itself, we should use relative paths
        // #[cfg(not(any(test, doctest)))]
        {
            if $crate::PROFILING_ENABLED {
                let profile = $crate::Profile::new($name, $type, $is_async, $is_method);
                $crate::ProfileSection { profile }
            } else {
                $crate::ProfileSection::new($name)
            }
        }

        // // For testing, use direct calls to avoid import issues
        // #[cfg(any(test, doctest))]
        // {
        //     if $crate::profiling::is_profiling_enabled() {
        //         let profile = $crate::profiling::Profile::new($name, $type, $is_async, $is_method);
        //         $crate::profiling::ProfileSection { profile }
        //     } else {
        //         $crate::profiling::ProfileSection::new($name)
        //     }
        // }
    }};
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

/// Dumps the contents of the profiled functions registry for debugging purposes
///
/// This function is primarily intended for test and debugging use.
#[cfg(any(test, debug_assertions))]
pub fn dump_profiled_functions() -> Vec<(String, String)> {
    PROFILED_FUNCTIONS.lock().map_or_else(
        |_| vec![],
        |registry| {
            registry
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        },
    )
}

#[cfg(test)]
static TEST_MODE_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
/// Checks if we're in test mode to avoid duplicate profiling
/// This is used by the Profile::new function to avoid creating duplicate profiles
#[inline]
pub fn is_test_mode_active() -> bool {
    TEST_MODE_ACTIVE.load(Ordering::SeqCst)
}

#[cfg(all(test, feature = "time_profiling"))]
/// Force sets the profiling state for testing purposes
/// This is only available in test mode with the profiling feature enabled
pub fn force_set_profiling_state(enabled: bool) {
    // This function is only used in tests to directly manipulate the profiling state
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
}

#[cfg(test)]
/// Sets up profiling for a test in a safe manner by first clearing the stack
pub fn safely_setup_profiling_for_test() -> crate::ProfileResult<()> {
    // Set test mode active to prevent #[profiled] from creating duplicate entries
    TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

    // Then enable profiling
    enable_profiling(true, ProfileType::Time)
}

#[cfg(test)]
/// Safely cleans up profiling after a test
pub fn safely_cleanup_profiling_after_test() -> crate::ProfileResult<()> {
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
    use std::time::Duration;

    // Basic profiling tests

    #[test]
    #[serial]
    fn test_profiling_profile_type_from_str() {
        assert_eq!(ProfileType::from_str("time"), Some(ProfileType::Time));
        assert_eq!(ProfileType::from_str("memory"), Some(ProfileType::Memory));
        assert_eq!(ProfileType::from_str("both"), Some(ProfileType::Both));
        assert_eq!(ProfileType::from_str("invalid"), None);
    }

    // Function registry tests

    #[test]
    #[serial]
    fn test_profiling_function_registry() {
        // Register a function
        register_profiled_function("test_func", "test_desc".to_string());

        // Check if it's registered
        assert!(is_profiled_function("test_func"));

        // Check the descriptive name
        assert_eq!(
            get_reg_desc_name("test_func"),
            Some("test_desc".to_string())
        );

        // Check a non-registered function
        assert!(!is_profiled_function("nonexistent"));
        assert_eq!(get_reg_desc_name("nonexistent"), None);
    }

    // ProfileStats tests

    #[test]
    fn test_profiling_profile_stats() {
        let mut stats = ProfileStats::default();

        // Record some calls
        stats.record("func1", Duration::from_micros(100));
        stats.record("func1", Duration::from_micros(200));
        stats.record("func2", Duration::from_micros(150));

        // Check call counts
        assert_eq!(*stats.calls.get("func1").unwrap(), 2);
        assert_eq!(*stats.calls.get("func2").unwrap(), 1);

        // Check total times
        assert_eq!(*stats.total_time.get("func1").unwrap(), 300);
        assert_eq!(*stats.total_time.get("func2").unwrap(), 150);

        // Mark async boundaries
        stats.mark_async("func1");
        assert!(stats.is_async_boundary("func1"));
        assert!(!stats.is_async_boundary("func2"));
    }

    // Profile type tests

    // Thread-safety tests

    // utils tests

    #[test]
    fn test_profiling_clean_function_name() {
        // Test with hash suffix
        let name = "module::func::h1234abcd";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with closure
        let name = "module::func{{closure}}";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with both
        let name = "module::func{{closure}}::h1234abcd";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with multiple colons
        let name = "module::::func";
        assert_eq!(clean_function_name(name), "module::func");
    }

    #[test]
    fn test_profiling_extract_fn_only() {
        // Test with module path
        let name = "module::submodule::function";
        assert_eq!(extract_fn_only(name), Some("function".to_string()));

        // Test with just function
        let name = "function";
        assert_eq!(extract_fn_only(name), Some("function".to_string()));
    }

    // Memory profiling tests

    // Test enabling/disabling profiling

    #[test]
    #[serial]
    fn test_profiling_enable_disable_profiling() {
        // Only test in our integrated test that doesn't depend on the other tests
        // This test is replaced by feature_tests::test_profiling_feature_flag_behavior in lib.rs
        // Skip in normal internal unit tests
    }

    // Test different file paths

    #[test]
    #[serial]
    fn test_profiling_profile_paths() {
        // Check that paths include the executable name and timestamp
        let paths = ProfilePaths::get();

        assert!(
            paths.time.ends_with(".folded"),
            "Time path should end with .folded"
        );
        assert!(
            paths.memory.ends_with("-memory.folded"),
            "Memory path should end with -memory.folded"
        );

        // Check that timestamp format is correct (YYYYmmdd-HHMMSS)
        let re = regex::Regex::new(r"\d{8}-\d{6}\.folded$").unwrap();
        assert!(
            re.is_match(&paths.time),
            "Time path should contain timestamp in YYYYmmdd-HHMMSS format"
        );
    }
}
