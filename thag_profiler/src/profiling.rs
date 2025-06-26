#![allow(unused_variables)]
use crate::{debug_log, static_lazy, ProfileError, ProfileResult};
use chrono::Local;
use parking_lot::{Mutex, RwLock};
use std::{
    collections::{BTreeSet, HashMap},
    env,
    fmt::{Display, Formatter},
    fs::File,
    io::BufWriter,
    path::PathBuf,
    str::FromStr,
    sync::atomic::{AtomicU8, Ordering},
    time::Instant,
};

use crate::safe_alloc;

#[cfg(feature = "full_profiling")]
use crate::{
    fn_name,
    mem_attribution::{deregister_profile, get_next_profile_id, register_profile},
    mem_tracking::{
        activate_task, create_memory_task, TaskGuard, TaskMemoryContext, TASK_PATH_REGISTRY,
    },
};

#[cfg(feature = "time_profiling")]
use backtrace::{resolve_frame, trace};

#[cfg(feature = "time_profiling")]
use crate::{file_stem_from_path_str, flush_debug_log, get_base_location, warn_once};

#[cfg(feature = "time_profiling")]
use parking_lot::ReentrantMutex;

#[cfg(feature = "time_profiling")]
use std::{
    collections::HashSet,
    convert::Into,
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64},
        OnceLock,
    },
    time::SystemTime,
};

#[cfg(feature = "full_profiling")]
use regex::Regex;

#[cfg(feature = "full_profiling")]
use std::sync::{atomic::AtomicUsize, Arc};

// Single atomic for runtime profiling state
#[cfg(feature = "time_profiling")]
static PROFILING_STATE: AtomicBool = AtomicBool::new(false);

/// Mutex to prevent concurrent access to profiling by different executions.
#[cfg(feature = "time_profiling")]
pub static PROFILING_MUTEX: ReentrantMutex<()> = ReentrantMutex::new(());

/// Profiling capability flags (bitflags pattern) for determining which profiling types are supported
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileCapability(pub u8);

#[allow(dead_code)]
impl ProfileCapability {
    /// No profiling capabilities
    pub const NONE: Self = Self(0);
    /// Time profiling capability
    pub const TIME: Self = Self(1);
    /// Memory profiling capability
    pub const MEMORY: Self = Self(2);
    /// Both time and memory profiling capabilities
    pub const BOTH: Self = Self(3); // TIME | MEMORY

    /// Returns the capabilities available based on enabled features
    #[must_use]
    pub const fn available() -> Self {
        #[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
        {
            Self::TIME
        }

        #[cfg(feature = "full_profiling")]
        {
            Self::BOTH
        }

        // If no profiling features are enabled
        #[cfg(all(not(feature = "time_profiling"), not(feature = "full_profiling")))]
        {
            Self::NONE
        }
    }

    /// Checks if the given profile type is supported by the available capabilities
    #[must_use]
    pub const fn supports(&self, profile_type: ProfileType) -> bool {
        match profile_type {
            ProfileType::Time => (self.0 & Self::TIME.0) == Self::TIME.0,
            ProfileType::Memory => (self.0 & Self::MEMORY.0) == Self::MEMORY.0,
            ProfileType::Both => (self.0 & Self::BOTH.0) == Self::BOTH.0,
            ProfileType::None => true,
        }
    }

    /// Convert from `ProfileType` to capabilities
    #[must_use]
    pub const fn from_profile_type(profile_type: ProfileType) -> Self {
        match profile_type {
            ProfileType::Time => Self::TIME,
            ProfileType::Memory => Self::MEMORY,
            ProfileType::Both => Self::BOTH,
            ProfileType::None => Self::NONE,
        }
    }

    /// Returns the intersection of the requested profile type and available capabilities
    #[must_use]
    pub const fn intersection(self, profile_type: ProfileType) -> Self {
        Self(self.0 & Self::from_profile_type(profile_type).0)
    }
}

/// Checks if a profile type is valid for the current feature set
#[cfg(debug_assertions)]
const fn is_valid_profile_type(profile_type: ProfileType) -> bool {
    ProfileCapability::available().supports(profile_type)
}

// Compile-time feature check - always use the runtime state in tests
#[cfg(all(feature = "time_profiling", not(test)))]
const PROFILING_FEATURE: bool = true;

#[allow(dead_code)]
#[cfg(all(feature = "time_profiling", test))]
const PROFILING_FEATURE: bool = false;

static GLOBAL_PROFILE_TYPE: AtomicU8 = AtomicU8::new(0); // 0 = None, 1 = Time, 2 = Memory, 3 = Both

// Implementation for ProfileCapability is defined above

// In-memory cache for profile configuration
static PROFILE_CONFIG_CACHE: Mutex<Option<ProfileConfiguration>> = Mutex::new(None);

/// Gets the current profile configuration
///
/// This function returns the current profile configuration, reading from
/// the environment if necessary. This ensures that any changes to the
/// environment variables are picked up immediately.
///
/// # Panics
///
/// Panics if it encounters an invalid `THAG_PROFILER` environment variable.
#[must_use]
pub fn get_profile_config() -> ProfileConfiguration {
    // First check if we have a cached configuration
    {
        let cache = PROFILE_CONFIG_CACHE.lock();
        if let Some(config) = &*cache {
            return config.clone();
        }
    }

    // No cached config, create one
    let config = parse_env_profile_config().expect("Expected environment variable `THAG_PROFILER={time|memory|both|none},[dir],{none}quiet|announce}[,true|false]`");
    // eprintln!("No cached config - setting config to {:#?}", config);
    // eprintln!("{:?}", backtrace::Backtrace::new());

    // Cache the config for future use
    let mut cache = PROFILE_CONFIG_CACHE.lock();
    *cache = Some(config.clone());

    config
}

/// Clears the cached profile configuration, forcing a fresh read from environment
/// on the next call to `get_profile_config()`
pub fn clear_profile_config_cache() {
    let mut cache = PROFILE_CONFIG_CACHE.lock();
    *cache = None;
}

/// Sets the profile configuration
///
/// This function updates the profile configuration with a new value.
pub fn set_profile_config(config: ProfileConfiguration) {
    let mut cache = PROFILE_CONFIG_CACHE.lock();
    *cache = Some(config);
}

#[allow(dead_code)]
// Define the DebugLog enum
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// Debug level for profiling output
pub enum DebugLevel {
    #[default]
    /// No debug output
    None,
    /// Quiet debug output (debug without announcing debug log file location)
    Quiet,
    /// Announce debug output (debug, announcing debug log file location to `stderr`)
    Announce,
}

impl Display for DebugLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Quiet => write!(f, "quiet"),
            Self::Announce => write!(f, "announce"),
        }
    }
}

impl FromStr for DebugLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "quiet" => Ok(Self::Quiet),
            "announce" => Ok(Self::Announce),
            _ => Err(format!(
                "Invalid debug log type '{s}'. Expected 'none', 'quiet', or 'announce'"
            )),
        }
    }
}

#[derive(Debug, Clone)]
/// Configuration for profiling operations.
///
/// This struct contains all the settings needed to configure profiling behavior,
/// including the type of profiling to perform, output directory, debug level,
/// and whether to enable detailed memory profiling.
pub struct ProfileConfiguration {
    enabled: bool,
    profile_type: Option<ProfileType>,
    output_dir: Option<PathBuf>,
    debug_level: Option<DebugLevel>,
    detailed_memory: bool,
}

impl TryFrom<&[&str]> for ProfileConfiguration {
    type Error = ProfileError;

    fn try_from(value: &[&str]) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        // Parse profile type (first element)
        let profile_type = {
            let profile_type_str = value.first().map_or("", |s| *s).trim();
            // eprintln!("THAG_PROFILER: Parsing profile type '{profile_type_str}'");
            match profile_type_str.parse::<ProfileType>() {
                Ok(val) => {
                    // eprintln!("THAG_PROFILER: Successfully parsed profile type: {val:?}");
                    Some(val)
                }
                Err(e) => {
                    // eprintln!("THAG_PROFILER: Failed to parse profile type: {e}");
                    errors.push(e);
                    None
                }
            }
        };

        // Parse output directory (second element)
        let output_dir = if value.get(1).map_or("", |s| *s).trim().is_empty() {
            Some(PathBuf::from(".")) // Default to current directory if empty
        } else {
            Some(PathBuf::from(value.get(1).unwrap().trim()))
        };

        // Parse debug log (third element)
        let debug_level = if value.get(2).map_or("none", |s| *s).trim().is_empty() {
            errors.push(
                "Third element (debug log) is empty. Expected 'none', 'quiet', or 'announce'"
                    .to_string(),
            );
            None
        } else {
            match value.get(2).unwrap_or(&"none").parse::<DebugLevel>() {
                Ok(val) => Some(val),
                Err(e) => {
                    errors.push(e);
                    None
                }
            }
        };

        // Parse detailed memory (fourth element)
        let detailed_memory = value.get(3).is_some_and(|val| if val.trim().is_empty() {
            false // Default if empty
        } else if let Ok(val) = val.trim().parse::<bool>() {
            // Validate that detailed memory is only true for Memory or Both profile types
            if val
                && profile_type
                    .as_ref().is_some_and(|pt| *pt == ProfileType::Time)
            {
                errors.push(
                    "Detailed memory profiling can only be enabled with profile_type=memory or profile_type=both"
                        .to_string(),
                );
                false
            } else {
                val
            }
        } else {
            errors.push(format!(
                "Failed to parse '{val}' as boolean for detailed memory flag. Expected 'true' or 'false'"
            ));
            false
        });

        // If there are errors, return them
        if !errors.is_empty() {
            // eprintln!("THAG_PROFILER errors:{errors:#?}");
            return Err(ProfileError::General(errors.join("\n")));
        }

        Ok(Self {
            enabled: true, // Assume enabled if all elements are valid
            profile_type,
            output_dir,
            debug_level,
            detailed_memory,
        })
    }
}

impl ProfileConfiguration {
    /// Returns whether profiling is enabled in this configuration.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the profile type configured for this configuration.
    #[must_use]
    pub const fn profile_type(&self) -> Option<ProfileType> {
        self.profile_type
    }

    /// Sets the profile type for this configuration.
    pub const fn set_profile_type(&mut self, profile_type: Option<ProfileType>) {
        self.profile_type = profile_type;
    }

    /// Returns the debug level configured for this configuration.
    #[must_use]
    pub const fn debug_level(&self) -> Option<DebugLevel> {
        self.debug_level
    }

    /// Returns whether detailed memory profiling is enabled in this configuration.
    #[must_use]
    pub const fn is_detailed_memory(&self) -> bool {
        self.detailed_memory
    }
}

impl Default for ProfileConfiguration {
    #[cfg(feature = "time_profiling")]
    fn default() -> Self {
        use std::env::current_dir;

        #[cfg(feature = "full_profiling")]
        let profile_type = Some(ProfileType::Both);

        #[cfg(not(feature = "full_profiling"))]
        let profile_type = Some(ProfileType::Time);

        Self {
            enabled: true,
            profile_type,
            output_dir: Some(current_dir().expect("Failed to determine current directory")),
            debug_level: Some(DebugLevel::None),
            detailed_memory: false,
        }
    }

    #[cfg(not(feature = "time_profiling"))]
    fn default() -> Self {
        Self {
            enabled: false,
            profile_type: None,
            output_dir: None,
            debug_level: None,
            detailed_memory: false,
        }
    }
}

/// Internal helper function to parse `THAG_PROFILER` environment variable
/// into a `ProfileConfiguration`
///
/// # Errors
///
/// This function will return an error if it encounters an invalid `THAG_PROFILER` environment variable.
pub fn parse_env_profile_config() -> ProfileResult<ProfileConfiguration> {
    let Ok(env_var) = env::var("THAG_PROFILER") else {
        // eprintln!("THAG_PROFILER environment variable not found, returning disabled config");
        let profile_type = if cfg!(feature = "full_profiling") {
            Some(ProfileType::Both)
        } else if cfg!(feature = "time_profiling") {
            Some(ProfileType::Time)
        } else {
            None
        };
        return Ok(ProfileConfiguration {
            enabled: false,
            profile_type,
            output_dir: None,
            debug_level: None,
            detailed_memory: false,
        });
    };

    // eprintln!("THAG_PROFILER environment variable found: {env_var}");
    let parts: Vec<&str> = env_var.split(',').collect();
    // eprintln!("THAG_PROFILER parts: {parts:?}");
    ProfileConfiguration::try_from(parts.as_slice())
}

impl Display for ProfileConfiguration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Profile Config:")?;
        writeln!(f, "  Enabled: {}", self.enabled)?;

        match &self.profile_type {
            Some(pt) => writeln!(f, "  Profile Type: {pt:?}")?,
            None => writeln!(f, "  Profile Type: none")?,
        }

        match &self.output_dir {
            Some(dir) => writeln!(f, "  Output Directory: {}", dir.display())?,
            None => writeln!(f, "  Output Directory: none")?,
        }

        match &self.debug_level {
            Some(log) => writeln!(f, "  Debug Log: {log:?}")?,
            None => writeln!(f, "  Debug Log: none")?,
        }

        write!(f, "  Detailed Memory: {:?}", self.detailed_memory)
    }
}

/// Gets the current debug level from the profile configuration.
///
/// This function retrieves the debug level setting from the current profile
/// configuration, returning `DebugLevel::None` as the default if no debug
/// level is explicitly configured.
///
/// # Returns
///
/// The configured `DebugLevel`, or `DebugLevel::None` if not set.
#[must_use]
pub fn get_debug_level() -> DebugLevel {
    get_profile_config().debug_level.unwrap_or_default()
}

#[must_use]
/// Checks if detailed memory profiling is enabled in the current configuration.
///
/// This function retrieves the detailed memory setting from the current profile
/// configuration. Detailed memory profiling provides more granular memory tracking
/// information when enabled.
///
/// # Returns
///
/// `true` if detailed memory profiling is enabled, `false` otherwise.
pub fn is_detailed_memory() -> bool {
    get_profile_config().detailed_memory
}

/// Gets the profile type from the current configuration.
///
/// This function retrieves the profile type setting from the current profile
/// configuration, returning `ProfileType::default()` if no profile type is
/// explicitly configured.
///
/// # Returns
///
/// The configured `ProfileType`, or the default profile type if not set.
#[must_use]
pub fn get_config_profile_type() -> ProfileType {
    // parse_env_profile_config().profile_type.unwrap_or_default()
    get_profile_config().profile_type.unwrap_or_default()
}

// Global registry of profiled functions
static PROFILED_FUNCTIONS: std::sync::LazyLock<RwLock<HashMap<String, String>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

static_lazy! {
    ProfilePaths: ProfileFilePaths = {
        let script_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("unknown"));
        let script_stem = script_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let base = format!("{script_stem}-{timestamp}");

        // Create debug log path in temp directory
        let mut debug_log_path = std::env::temp_dir();
        debug_log_path.push("thag_profiler");
        std::fs::create_dir_all(&debug_log_path).ok();
        debug_log_path.push(format!("{base}-debug.log"));

        ProfileFilePaths {
            time: format!("{base}.folded"),
            inclusive_time: format!("{base}-inclusive.folded"),
            memory: format!("{base}-memory.folded"),
            debug_log: debug_log_path.to_string_lossy().to_string(),
            executable_stem: script_stem.to_string(),
            timestamp,
            memory_detail: format!("{base}-memory_detail.folded"),
            memory_detail_dealloc: format!("{base}-memory_detail_dealloc.folded")
        }
    }
}

// File handles
static_lazy! {
    TimeProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

static_lazy! {
    InclusiveTimeProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

// #[cfg(feature = "full_profiling")]
static_lazy! {
    MemoryProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

static_lazy! {
    MemoryDeallocFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

#[cfg(feature = "full_profiling")]
static_lazy! {
    MemoryDetailFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

#[cfg(feature = "full_profiling")]
static_lazy! {
    MemoryDetailDeallocFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

#[cfg(feature = "time_profiling")]
static START_TIME: AtomicU64 = AtomicU64::new(0);

/// Stores file paths for different types of profiling output files.
///
/// This struct contains the paths to various output files generated during profiling,
/// including time profiles, memory profiles, debug logs, and other metadata.
#[derive(Clone)]
#[allow(dead_code)]
pub struct ProfileFilePaths {
    time: String,
    inclusive_time: String, // Stores inclusive times before conversion to exclusive
    memory: String,
    memory_detail: String,
    memory_detail_dealloc: String,
    /// The full path to the debug log file
    pub debug_log: String,
    /// Store the executable stem for reuse
    pub executable_stem: String,
    /// Store the timestamp for reuse
    pub timestamp: String,
}

/// Get the path to the plain `.folded` output file.
///
/// # Errors
///
/// This function will bubble up any file system errors that occur trying to create the directory.
#[cfg(feature = "time_profiling")]
pub fn get_time_path() -> ProfileResult<&'static str> {
    struct TimePathHolder;
    impl TimePathHolder {
        fn get() -> ProfileResult<&'static str> {
            static PATH_RESULT: OnceLock<Result<String, ProfileError>> = OnceLock::new();

            let result = PATH_RESULT.get_or_init(|| {
                let paths = ProfilePaths::get();
                let config = get_profile_config();

                let path = if let Some(dir) = &config.output_dir {
                    let dir_path = PathBuf::from(dir);
                    if !dir_path.exists() {
                        match std::fs::create_dir_all(&dir_path) {
                            Ok(()) => {}
                            Err(e) => return Err(ProfileError::from(e)),
                        }
                    }

                    let time_file =
                        dir_path.join(paths.time.split('/').next_back().unwrap_or(&paths.time));
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

/// Get the path to the `memory_detail.folded` output file.
///
/// # Errors
///
/// This function will bubble up any file system errors that occur trying to create the directory.
#[cfg(feature = "full_profiling")]
pub fn get_memory_detail_path() -> ProfileResult<&'static str> {
    struct MemoryDetailPathHolder;
    impl MemoryDetailPathHolder {
        fn get() -> ProfileResult<&'static str> {
            static PATH_RESULT: OnceLock<Result<String, ProfileError>> = OnceLock::new();

            let result = PATH_RESULT.get_or_init(|| {
                let paths = ProfilePaths::get();
                let config = get_profile_config();

                let path = if let Some(dir) = &config.output_dir {
                    let dir_path = PathBuf::from(dir);
                    if !dir_path.exists() {
                        match std::fs::create_dir_all(&dir_path) {
                            Ok(()) => {}
                            Err(e) => return Err(ProfileError::from(e)),
                        }
                    }

                    let memory_detail_file = dir_path.join(
                        paths
                            .memory_detail
                            .split('/')
                            .next_back()
                            .unwrap_or(&paths.memory_detail),
                    );

                    memory_detail_file.to_string_lossy().to_string()
                } else {
                    paths.memory_detail.clone()
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

    MemoryDetailPathHolder::get()
}

/// Get the path to the `memory_detail_dealloc.folded` output file.
///
/// # Errors
///
/// This function will bubble up any filesystem errors that occur trying to create the directory.
#[cfg(feature = "full_profiling")]
pub fn get_memory_detail_dealloc_path() -> ProfileResult<&'static str> {
    struct MemoryDetailDeallocPathHolder;
    impl MemoryDetailDeallocPathHolder {
        fn get() -> ProfileResult<&'static str> {
            static PATH_RESULT: OnceLock<Result<String, ProfileError>> = OnceLock::new();

            let result = PATH_RESULT.get_or_init(|| {
                let paths = ProfilePaths::get();
                let config = get_profile_config();

                let path = if let Some(dir) = &config.output_dir {
                    let dir_path = PathBuf::from(dir);
                    if !dir_path.exists() {
                        match std::fs::create_dir_all(&dir_path) {
                            Ok(()) => {}
                            Err(e) => return Err(ProfileError::from(e)),
                        }
                    }

                    let memory_detail_dealloc_file = dir_path.join(
                        paths
                            .memory_detail_dealloc
                            .split('/')
                            .next_back()
                            .unwrap_or(&paths.memory_detail_dealloc),
                    );

                    memory_detail_dealloc_file.to_string_lossy().to_string()
                } else {
                    paths.memory_detail_dealloc.clone()
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

    MemoryDetailDeallocPathHolder::get()
}

/// Get the path to the memory.folded output file.
///
/// # Errors
///
/// This function will bubble up any filesystem errors that occur trying to create the directory.
#[cfg(feature = "full_profiling")]
pub fn get_memory_path() -> ProfileResult<&'static str> {
    struct MemoryPathHolder;
    impl MemoryPathHolder {
        fn get() -> ProfileResult<&'static str> {
            static PATH_RESULT: OnceLock<Result<String, ProfileError>> = OnceLock::new();

            let result = PATH_RESULT.get_or_init(|| {
                let paths = ProfilePaths::get();
                let config = get_profile_config();

                let path = if let Some(dir) = &config.output_dir {
                    let dir_path = PathBuf::from(dir);
                    if !dir_path.exists() {
                        match std::fs::create_dir_all(&dir_path) {
                            Ok(()) => {}
                            Err(e) => return Err(ProfileError::from(e)),
                        }
                    }

                    let memory_file =
                        dir_path.join(paths.memory.split('/').next_back().unwrap_or(&paths.memory));

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
/// This function handles the initialization sequence for profiling files
/// based on enabled features and requested profile type.
///
/// # Arguments
/// * `profile_type` - The type of profiling to initialize files for
///
/// # Errors
/// Returns a `ProfileError` if any file operations fail
#[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
fn initialize_profile_files(profile_type: ProfileType) -> ProfileResult<()> {
    // Get available capabilities based on enabled features
    let available_caps = ProfileCapability::available();

    // Check if the requested profile type is supported
    if !available_caps.supports(profile_type) {
        // Handle unsupported profile types
        if matches!(profile_type, ProfileType::Memory | ProfileType::Both) {
            panic!(
                "Profile type `{profile_type:?}` requested but feature `full_profiling` is not enabled",
            );
        }

        if profile_type == ProfileType::None {
            debug_log!("ProfileType::None selected: no profiling will be done");
            return Ok(());
        }
    }

    // Initialize time profiling
    if matches!(profile_type, ProfileType::Time | ProfileType::Both) {
        // Initialize the inclusive time file first
        let paths = ProfilePaths::get();
        let inclusive_time_path = &paths.inclusive_time;
        InclusiveTimeProfileFile::init();
        initialize_file(inclusive_time_path, InclusiveTimeProfileFile::get())?;
        debug_log!("Inclusive time profile will be written to {inclusive_time_path}");

        // Initialize the final time file (will be used for exclusive time)
        let time_path = get_time_path()?;
        TimeProfileFile::init();
        initialize_file(time_path, TimeProfileFile::get())?;
        debug_log!("Time profile will be written to {time_path}");
    }

    flush_debug_log();
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
/// Returns a `ProfileError` if any file operations fail
#[cfg(feature = "full_profiling")]
fn initialize_profile_files(profile_type: ProfileType) -> ProfileResult<bool> {
    // Get available capabilities based on enabled features
    let available_caps = ProfileCapability::available();

    debug_log!("In initialize_profile_files for profile_type={profile_type:?}");

    // Early return for ProfileType::None
    if profile_type == ProfileType::None {
        debug_log!("ProfileType::None selected: no profiling will be done");
        flush_debug_log();
        return Ok(true);
    }

    // Check the intersection of requested and available capabilities
    let actual_caps = available_caps.intersection(profile_type);

    // Initialize files based on the actual capabilities
    // Initialize time profiling if needed
    if (actual_caps.0 & ProfileCapability::TIME.0) != 0 {
        // Initialize the inclusive time file first
        let paths = ProfilePaths::get();
        let inclusive_time_path = &paths.inclusive_time;
        InclusiveTimeProfileFile::init();
        initialize_file(inclusive_time_path, InclusiveTimeProfileFile::get())?;
        debug_log!("Inclusive time profile will be written to {inclusive_time_path}");

        // Initialize the final time file (will be used for exclusive time)
        let time_path = get_time_path()?;
        TimeProfileFile::init();
        initialize_file(time_path, TimeProfileFile::get())?;
        debug_log!("Time profile will be written to {time_path}");
    }

    // Initialize memory profiling if needed
    if (actual_caps.0 & ProfileCapability::MEMORY.0) != 0 {
        let memory_path = get_memory_path()?;
        let memory_detail_path = get_memory_detail_path()?;
        let memory_detail_dealloc_path = get_memory_detail_dealloc_path()?;

        MemoryProfileFile::init();
        initialize_file(memory_path, MemoryProfileFile::get())?;
        debug_log!("Memory profile will be written to {memory_path}");

        if is_detailed_memory() {
            MemoryDetailFile::init();
            initialize_file(memory_detail_path, MemoryDetailFile::get())?;
            debug_log!("Memory detail will be written to {memory_detail_path}");
            MemoryDetailDeallocFile::init();
            initialize_file(memory_detail_dealloc_path, MemoryDetailDeallocFile::get())?;
            debug_log!("Memory detail dealloc will be written to {memory_detail_dealloc_path}");
        }
    }

    flush_debug_log();
    Ok(true)
}

#[cfg(feature = "time_profiling")]
fn initialize_file(
    file_path: &str,
    file: &parking_lot::lock_api::Mutex<parking_lot::RawMutex, Option<BufWriter<File>>>,
) -> Result<(), ProfileError> {
    *file.lock() = None;
    initialize_profile_file(file_path)?;
    Ok(())
}

/// Returns the global profile type.
///
/// This function maps from the atomic global stored value to the corresponding
/// `ProfileType`. If no global type is set, it first sets the atomic value from
/// the profile configuration.
pub fn get_global_profile_type() -> ProfileType {
    // Removed lazy_static_var! to enable thorough unit testing.
    // Instead ensure frequent callers of this function implement it themselves if needed.
    // lazy_static_var!(ProfileType, deref, {
    // Map the stored value to a ProfileType using the bitflags pattern
    let global_value = GLOBAL_PROFILE_TYPE.load(Ordering::SeqCst);

    // eprintln!(
    //     "get_global_profile_type: global_value={global_value}",
    //     // backtrace::Backtrace::new()
    // );

    // if global_value == 0 {
    //     eprintln!(
    //         "...setting to {:?}",
    //         get_profile_config()
    //             .profile_type
    //             .unwrap_or(ProfileType::None)
    //     );
    // }

    match global_value {
        0 => {
            // eprintln!("*** get_global_profile_type found value 0 ***");
            let profile_type = get_profile_config()
                .profile_type
                .unwrap_or(ProfileType::None);
            set_global_profile_type(profile_type);
            profile_type
        }
        1 => ProfileType::Time,
        2 => ProfileType::Memory,
        3 => ProfileType::Both,
        _ => {
            // Should never happen, but handle gracefully if it does
            debug_log!("Unexpected GLOBAL_PROFILE_TYPE value: {}", global_value);
            get_profile_config()
                .profile_type
                .unwrap_or(ProfileType::None)
        }
    }
    // })
}

#[allow(clippy::missing_panics_doc)]
/// Sets the global profile type.
///
/// This function updates the global profile type atomically. It validates that the
/// profile type is valid for the current feature set in debug builds.
///
/// # Arguments
/// * `profile_type` - The profile type to set globally
///
/// # Panics
/// In debug builds, panics if the profile type is not valid for the current feature set.
pub fn set_global_profile_type(profile_type: ProfileType) {
    #[cfg(all(debug_assertions, feature = "full_profiling"))]
    assert!(
        is_valid_profile_type(profile_type),
        "Invalid profile type {profile_type:?} for feature set"
    );

    #[cfg(all(debug_assertions, not(feature = "full_profiling")))]
    if profile_type == ProfileType::Memory {
        assert!(
            !is_valid_profile_type(profile_type),
            "Profile type {profile_type:?} should not be valid for feature set"
        );
    } else {
        assert!(
            is_valid_profile_type(profile_type),
            "Invalid profile type {profile_type:?} for feature set"
        );
    }

    // Map profile type directly to storage value using the bitflags pattern
    let value = ProfileCapability::from_profile_type(profile_type).0;
    GLOBAL_PROFILE_TYPE.store(value, Ordering::SeqCst);

    // debug_log causes this to hang if called from get_global_profile_type during initialisation.
    eprintln!("set_global_profile_type: profile_type={profile_type:?}, stored value={value}");
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
// #[deprecated(
//     since = "0.1.0",
//     note = "Use the #[enable_profiling] attribute macro instead"
// )]
pub(crate) fn enable_profiling(
    enabled: bool,
    maybe_profile_type: Option<ProfileType>,
) -> ProfileResult<()> {
    let _guard = PROFILING_MUTEX.lock();
    // When running tests (either unit or integration), we want to ensure our configuration
    // is up-to-date with the latest environment variables
    #[cfg(test)]
    {
        debug_log!("Unit test: Resetting profile config to ensure latest env vars are used");
        clear_profile_config_cache();
    }

    // Check if the operation is a no-op due to environment settings
    let config = get_profile_config();
    if enabled != config.enabled {
        debug_log!(
            "Caution: `enable_profiling` attribute or function `enabled={enabled}` call overriding configured value"
        );
    }

    if enabled {
        // Programmatic call may override the config defaults.
        debug_log!(
            "maybe_profile_type={maybe_profile_type:?}, get_config_profile_type={:?}",
            get_config_profile_type()
        );

        if PROFILING_STATE.load(Ordering::SeqCst) {
            return Err(ProfileError::General(
                "Can't enable profiling: already enabled".to_string(),
            ));
        }

        let final_profile_type = if let Some(profile_type) = maybe_profile_type {
            debug_log!(
                "enable_profiling: Using provided profile_type={:?}",
                profile_type
            );
            profile_type
        } else {
            let config_profile_type = get_config_profile_type();
            debug_log!(
                "enable_profiling: Using config_profile_type={:?}",
                config_profile_type
            );

            if !cfg!(feature = "full_profiling") && config_profile_type != ProfileType::Time {
                debug_log!(
                    "enable_profiling: Memory profiling not allowed without full_profiling feature"
                );
                return Err(ProfileError::General(
                    "Memory profiling not allowed since feature `full_profiling` is not specified"
                        .to_string(),
                ));
            }
            config_profile_type
        };

        set_global_profile_type(final_profile_type);
        debug_log!("Set global profile type to {:?}", get_global_profile_type());

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
    debug_log!("Profiling state set to {}", enabled);

    Ok(())
}

// /// No-op version when profiling feature is disabled.
// ///
// /// # Errors
// /// None
// #[cfg(not(feature = "time_profiling"))]
// pub(crate) const fn enable_profiling(
//     _enabled: bool,
//     _maybe_profile_type: Option<ProfileType>,
// ) -> Result<(), ProfileError> {
//     // No-op implementation
//     Ok(())
// }

/// Disables profiling.
///
/// This function disables profiling and resets the profiling state.
/// Use this to explicitly stop profiling that was enabled via the
/// `#[enable_profiling]` attribute macro.
#[allow(clippy::missing_const_for_fn)]
pub fn disable_profiling() {
    #[cfg(feature = "time_profiling")]
    {
        // Call the internal enable_profiling function with false
        let _ = crate::profiling::enable_profiling(false, None);
    }

    #[cfg(not(feature = "time_profiling"))]
    {}
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
fn initialize_profile_file(path: &str) -> ProfileResult<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    // writeln!(file, "# {profile_type}")?;
    // writeln!(
    //     file,
    //     "# Script: {}",
    //     std::env::current_exe().unwrap_or_default().display()
    // )?;
    // writeln!(file, "# Started: {}", START_TIME.load(Ordering::SeqCst))?;
    // writeln!(file, "# Version: {}", env!("CARGO_PKG_VERSION"))?;
    // writeln!(file)?;

    Ok(())
}

/// Checks if profiling is currently enabled.
///
/// This is used throughout the profiling system to determine whether
/// profiling operations should be performed. It's atomic and thread-safe.
///
/// # Returns
/// `true` if profiling is enabled, `false` otherwise
// #[inline(always)]
// #[allow(clippy::inline_always)]
#[allow(clippy::missing_const_for_fn)]
#[must_use]
pub fn is_profiling_enabled() -> bool {
    #[cfg(feature = "time_profiling")]
    {
        // debug_log!(
        //     r#"cfg!(test)={}, cfg(feature = "time_profiling")={}"#,
        //     cfg!(test),
        //     cfg!(feature = "time_profiling")
        // );
        // In test mode, only use the runtime state to allow enable/disable testing.
        // Note that cfg(test) only applies to internal tests, not to external dirs like test/.
        // eprintln!("cfg!(test)={}", cfg!(test));

        #[cfg(test)]
        let enabled = PROFILING_STATE.load(Ordering::SeqCst);

        // In normal operation, use both feature flag and runtime state
        #[cfg(not(test))]
        let enabled = PROFILING_FEATURE && PROFILING_STATE.load(Ordering::SeqCst);

        enabled
    }

    #[cfg(not(feature = "time_profiling"))]
    {
        false
    }
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
// #[inline(always)]
// #[allow(clippy::inline_always)]
#[allow(clippy::missing_const_for_fn)]
#[must_use]
pub fn is_profiling_state_enabled() -> bool {
    #[cfg(feature = "time_profiling")]
    {
        // debug_log!(
        //     r#"cfg!(test)={}, cfg(feature = "time_profiling")={}"#,
        //     cfg!(test),
        //     cfg!(feature = "time_profiling")
        // );
        PROFILING_STATE.load(Ordering::SeqCst)
    }

    #[cfg(not(feature = "time_profiling"))]
    {
        false
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// Types of profiling that can be performed.
///
/// This enum defines the different profiling modes available in the system.
/// The profiling type determines what metrics are collected and reported.
pub enum ProfileType {
    /// Time profiling only - measures wall clock/elapsed time
    Time, // Wall clock/elapsed time
    /// Memory profiling only - tracks memory allocations and deallocations
    Memory,
    /// Both time and memory profiling - combines both measurement types
    #[default]
    Both,
    /// No profiling - disables all profiling operations
    None,
}

impl Display for ProfileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Time => write!(f, "time"),
            Self::Memory => write!(f, "memory"),
            Self::Both => write!(f, "both"),
            Self::None => write!(f, "none"),
        }
    }
}

impl FromStr for ProfileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();
        // eprintln!("ProfileType::from_str: parsing '{trimmed}'");

        match trimmed.as_str() {
            "time" => {
                // eprintln!("ProfileType::from_str: matched 'time'");
                Ok(Self::Time)
            }
            "memory" => {
                // eprintln!("ProfileType::from_str: matched 'memory'");
                Ok(Self::Memory)
            }
            "both" => {
                // eprintln!("ProfileType::from_str: matched 'both'");
                Ok(Self::Both)
            }
            "none" | "" => {
                // eprintln!("ProfileType::from_str: matched 'none' or ''");
                Ok(Self::None)
            }
            _ => {
                let err = format!(
                    "Invalid profile type '{s}'. Expected 'time', 'memory', 'both', or 'none'"
                );
                // eprintln!("ProfileType::from_str: error: {err}");
                Err(err)
            }
        }
    }
}

/// The Profile struct represents one profiled execution of a function or section of code.
///
/// Its logical key is the same call hierarchy that will be reflected in the flamegraph, namely
/// the callstack from the (main) function to the current function, but with all unprofiled
/// functions removed.
#[allow(clippy::struct_field_names, dead_code)]
#[derive(Clone, Debug)]
pub struct Profile {
    start: Option<Instant>,
    profile_type: ProfileType,
    path: Vec<String>,
    section_name: Option<String>, // Custom section name when provided via profile!(name) macro
    registered_name: String,
    fn_name: String,
    start_line: Option<u32>, // Source line where profile was created (for sections)
    end_line: Option<u32>,   // Source line where profile was ended (if section explicitly ended)
    detailed_memory: bool,   // Whether to do detailed memory profiling for this profile
    file_name: String,       // Filename where this profile was created
    instance_id: u64,        // Unique identifier for this Profile instance
    #[cfg(feature = "full_profiling")]
    allocation_total: Arc<AtomicUsize>, // All clones share the same underlying value
    #[cfg(feature = "full_profiling")]
    memory_reported: Arc<AtomicBool>, // All clones share the same underlying value
    #[cfg(feature = "full_profiling")]
    memory_task: Option<TaskMemoryContext>,
    #[cfg(feature = "full_profiling")]
    memory_guard: Option<TaskGuard>,
}

impl Profile {
    /// Get the module path of this profile
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Get the `fn_name` of this profile
    #[must_use]
    pub const fn fn_name(&self) -> &str {
        self.fn_name.as_str()
    }

    /// Get the start line of this profile
    #[must_use]
    pub const fn start_line(&self) -> Option<u32> {
        self.start_line
    }

    /// Get the end line of this profile, if available
    #[must_use]
    pub const fn end_line(&self) -> Option<u32> {
        self.end_line
    }

    /// Check if this profile uses detailed memory tracking
    #[must_use]
    pub const fn detailed_memory(&self) -> bool {
        self.detailed_memory
    }

    /// Get the registered name of this profile
    #[must_use]
    pub fn registered_name(&self) -> &str {
        &self.registered_name
    }

    /// Get the custom name of this profile, if provided
    #[must_use]
    pub fn section_name(&self) -> Option<String> {
        self.section_name.clone()
    }

    /// Get the unique identifier for this profile
    #[must_use]
    pub const fn instance_id(&self) -> u64 {
        self.instance_id
    }

    /// Records a memory allocation in this profile
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the allocation in bytes
    ///
    /// # Returns
    ///
    /// `true` if the allocation was recorded, `false` otherwise
    #[cfg(feature = "full_profiling")]
    #[must_use]
    pub fn record_allocation(&self, size: usize) -> bool {
        debug_log!(
            "In Profile::record_allocation for size={size} for profile {} of type {:?}, detailed_memory={}, task_id={}",
            self.registered_name,
            self.profile_type,
            self.detailed_memory,
            self.memory_task.as_ref().map_or("N/A".to_string(), |context| format!("{}", context.task_id))
        );

        if size == 0 {
            return false;
        }

        // Just add to our local counter
        self.allocation_total.fetch_add(size, Ordering::Relaxed);

        debug_log!(
            "Profile {} recorded allocation of {size} bytes, total now: {}",
            self.registered_name,
            self.allocation_total.load(Ordering::Relaxed)
        );

        true
    }

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
    ///     let _p = Profile::new(Some("time_only_function"), None, ProfileType::Time, false, false, file!(), None, None);
    ///     // Code to profile...
    /// }
    ///
    /// // With memory profiling (requires `full_profiling` feature)
    /// #[cfg(feature = "full_profiling")]
    /// {
    ///     let _p = Profile::new(Some("memory_tracking_function"), None, ProfileType::Memory, false, false, file!(), None, None);
    ///     // Code to profile with memory tracking...
    /// }
    /// ```
    /// # Panics
    ///
    /// Panics if stack validation fails.
    #[allow(
        clippy::inline_always,
        clippy::too_many_arguments,
        clippy::too_many_lines,
        unused_variables
    )]
    #[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
    pub fn new(
        section_name: Option<&str>,
        _maybe_fn_name: Option<&str>,
        requested_type: ProfileType,
        is_async: bool,
        detailed_memory: bool,
        file_name: &'static str,
        start_line: Option<u32>,
        end_line: Option<u32>,
    ) -> Option<Self> {
        warn_once!(
            !is_profiling_enabled(),
            || {
                debug_log!("Profiling is not enabled, returning None");
            },
            return None
        );

        // In test mode with our test wrapper active, skip creating profile for #[profiled] attribute
        #[cfg(test)]
        if is_test_mode_active() {
            // If this is from an attribute in a test, don't create a profile
            // Our safe wrapper will handle profiling instead
            eprintln!("Test mode is active, returning None");
            return None;
        }

        // Try allowing overrides
        let profile_type = if matches!(requested_type, ProfileType::Memory | ProfileType::Both) {
            debug_log!("Memory profiling requested but the 'full_profiling' feature is not enabled. Only time will be profiled.");
            ProfileType::Time
        } else {
            requested_type
        };

        let detailed_memory = detailed_memory
            && (profile_type == ProfileType::Memory || profile_type == ProfileType::Both);

        // debug_log!("Current function/section: {section_name:?}, requested_type: {requested_type:?}, full_profiling?: {}", cfg!(feature = "full_profiling"));
        let start_pattern = "Profile::new";

        // let mut current_backtrace = Backtrace::new_unresolved();
        let cleaned_stack = extract_profile_callstack(start_pattern);

        // debug_log!("cleaned_stack={cleaned_stack:#?}");

        let fn_name = &cleaned_stack[0];

        #[cfg(not(target_os = "windows"))]
        let desc_fn_name = if is_async {
            format!("async::{fn_name}")
        } else {
            fn_name.to_string()
        };

        #[cfg(target_os = "windows")]
        let desc_fn_name = fn_name.to_string(); // Windows already highlights async functions

        let path = extract_path(&cleaned_stack, Some(fn_name));

        let stack = path.join(";");

        debug_log!("Calling register_profiled_function({stack}, {desc_fn_name})");
        register_profiled_function(&stack, &desc_fn_name);

        // Determine if we should keep the section name
        let section_name = section_name.map(str::to_string);

        // Debug output can be turned back on if needed for troubleshooting
        // debug_log!(
        //     "DEBUG: Profile::new with fn_name='{fn_name}', section_name={section_name:?}, requested_type={requested_type:?}, profile_type={profile_type:?}, initial_memory={initial_memory:?}"
        // );

        // Create a basic profile structure that works for all configurations
        if profile_type == ProfileType::Memory {
            debug_log!("Memory profiling requested but the 'full_profiling' feature is not enabled. Only time will be profiled.");
        }

        debug_log!(
            "NEW PROFILE: (Time) created for {}\ndesc_stack = {}",
            path.join(" -> "),
            build_stack(&path, section_name.as_ref(), " -> ")
        );

        let file_name_stem = file_stem_from_path_str(file_name);

        // Get a unique ID for this profile instance
        #[cfg(feature = "full_profiling")]
        let instance_id = get_next_profile_id();
        #[cfg(not(feature = "full_profiling"))]
        let instance_id = 0; // Dummy value when full_profiling is disabled

        Some(Self {
            profile_type,
            start: Some(Instant::now()),
            path,
            section_name,
            registered_name: fn_name.to_string(),
            fn_name: fn_name.to_string(),
            start_line,
            end_line,
            detailed_memory,
            file_name: file_name_stem,
            instance_id,
        })
    }

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
    ///     let _p = Profile::new(Some("time_only_function"), None, ProfileType::Time, false, false, file!(), None, None);
    ///     // Code to profile...
    /// }
    ///
    /// // With memory profiling (requires `full_profiling` feature)
    /// #[cfg(feature = "full_profiling")]
    /// {
    ///     let _p = Profile::new(Some("memory_tracking_function"), None, ProfileType::Memory, false, false, file!(), None, None);
    ///     // Code to profile with memory tracking...
    /// }
    /// ```
    /// # Panics
    ///
    /// Panics if stack validation fails.
    #[allow(
        clippy::inline_always,
        clippy::too_many_arguments,
        clippy::too_many_lines,
        unused_variables
    )]
    #[cfg(feature = "full_profiling")]
    pub fn new(
        section_name: Option<&str>,
        _maybe_fn_name: Option<&str>,
        requested_type: ProfileType,
        is_async: bool,
        detailed_memory: bool,
        file_name: &'static str,
        start_line: Option<u32>,
        end_line: Option<u32>,
    ) -> Option<Self> {
        warn_once!(
            !is_profiling_enabled(),
            || {
                debug_log!("Profiling is not enabled, returning None");
            },
            return None
        );

        // In test mode with our test wrapper active, skip creating profile for #[profiled] attribute
        #[cfg(test)]
        if is_test_mode_active() {
            // If this is from an attribute in a test, don't create a profile
            // Our safe wrapper will handle profiling instead
            eprintln!("Test mode is active, returning None");
            return None;
        }

        // For full profiling (specifically memory), run this method using the system allocator
        // so as not to clog the allocation tracking in mod mem_tracking.
        safe_alloc! {
            let start = Instant::now();
            // Try allowing overrides
            let profile_type = requested_type;
            // eprintln!("requested_type={requested_type:?}");

            let file_name_stem = file_stem_from_path_str(file_name);

            // debug_log!("file_name={file_name_stem}");

            // debug_log!("Current function/section: {section_name:?}, requested_type: {requested_type:?}, full_profiling?: {}", cfg!(feature = "full_profiling"));
            let start_pattern = "Profile::new";

            // let fn_name = maybe_fn_name.unwrap();

            // let mut current_backtrace = Backtrace::new_unresolved();
            // current_backtrace.resolve();
            // debug_log!("************\n{current_backtrace:?}\n************");

            let cleaned_stack = extract_profile_callstack(
                // fn_name,
                start_pattern,
                // &mut current_backtrace,
            );
            // debug_log!("cleaned_stack={cleaned_stack:#?}");

            if cleaned_stack.is_empty() {
                debug_log!("Empty cleaned stack found");
                return None;
            }

            // Register this function
            let fn_name = &cleaned_stack[0];

            #[cfg(not(target_os = "windows"))]
            let desc_fn_name = if section_name.is_some() && is_profiled_function(fn_name) {
                safe_alloc!(get_reg_desc_name(fn_name).unwrap_or_else(|| fn_name.to_string()))
            } else if is_async {
                safe_alloc!(format!("async::{fn_name}"))
            } else {
                safe_alloc!(fn_name.to_string())
            };

            #[cfg(target_os = "windows")]
            let desc_fn_name = safe_alloc!(fn_name.to_string()); // Windows already highlights async functions

            let path = extract_path(&cleaned_stack, Some(fn_name));
            // debug_log!("cleaned_stack={cleaned_stack:#?}, path={path:#?}");

            let stack = path.join(";");

            // debug_log!("fn_name={fn_name}, is_method={is_method}, maybe_method_name={maybe_method_name:?}, maybe_function_name={maybe_function_name:?}, desc_fn_name={desc_fn_name}");
            // debug_log!("Calling register_profiled_function({stack}, {desc_fn_name})");
            register_profiled_function(&stack, &desc_fn_name);

            // Determine if we should keep the section name
            let section_name = section_name.map(str::to_string);

            // Debug output can be turned back on if needed for troubleshooting
            // debug_log!(
            //     "DEBUG: Profile::new with , fn_name='{fn_name}', section_name={section_name:?}, requested_type={requested_type:?}, profile_type={profile_type:?}, initial_memory={initial_memory:?}"
            // );

            let instance_id = get_next_profile_id();

            // For full profiling, we need to handle memory task and guard creation ASAP and try to let the allocator track the
            // memory allocations in the profile setup itself in this method.
            if profile_type == ProfileType::Time {
                //     debug_log!(
                //     "Memory profiling enabled but only time profiling will be profiled as requested."
                // );

                debug_log!(
                    "NEW PROFILE: (Time) created for {}\ndesc_stack = {}",
                    path.join(" -> "),
                    build_stack(&path, section_name.as_ref(), " -> ")
                );

                let mut profile = Self {
                    profile_type,
                    start: None,
                    path,
                    section_name,
                    registered_name: stack,
                    fn_name: fn_name.to_string(),
                    start_line,
                    end_line,
                    detailed_memory,
                    file_name: file_name_stem,
                    instance_id,
                    allocation_total: Arc::new(AtomicUsize::new(0)),
                    memory_reported: Arc::new(AtomicBool::new(false)),
                    memory_task: None,
                    memory_guard: None,
                };

                // Register this profile with the new ProfileRegistry
                // First log the details to avoid potential deadlock
                // debug_log!(
                //         "About to register time_only profile in module {file_name} for fn {fn_name} with line range {start_line:?}..None",
                //     );

                // Flush logs before calling register_profile
                // flush_debug_log();

                // Now register the profile
                register_profile(&profile);

                profile.start = Some(Instant::now());

                return Some(profile);
            }

            // Create a memory task and activate it
            let memory_task = create_memory_task();
            let task_id = memory_task.id();

            // Register task path
            // debug_log!("Registering task path for task {task_id}: {path:?}");
            let mut registry = TASK_PATH_REGISTRY.lock();
            registry.insert(task_id, path.clone());
            let reg_len = registry.len();
            drop(registry);
            // debug_log!("TASK_PATH_REGISTRY now has {reg_len} entries",);

            // Activate the task
            activate_task(task_id);

            // Add to thread stack
            // push_task_to_stack(thread::current().id(), task_id);

            debug_log!(
                "NEW PROFILE: Task {task_id} created for {}\ndesc_stack = {}",
                path.join(" -> "),
                build_stack(&path, section_name.as_ref(), " -> ")
            );

            // Create memory guard
            let memory_guard = TaskGuard::new(task_id);

            let mut profile = {
                // Create the profile with necessary components
                debug_log!(
                    "Creating profile for {} in file {} with memory profiling enabled={}",
                    fn_name,
                    file_name_stem,
                    matches!(profile_type, ProfileType::Memory | ProfileType::Both)
                );

                Self {
                    profile_type,
                    start: None,
                    path,
                    section_name,
                    registered_name: stack,
                    fn_name: fn_name.to_string(),
                    start_line,
                    end_line,
                    detailed_memory,
                    #[cfg(feature = "debug_logging")]
                    file_name: file_name_stem.clone(),
                    #[cfg(not(feature = "debug_logging"))]
                    file_name: file_name_stem,
                    instance_id,
                    allocation_total: Arc::new(AtomicUsize::new(0)),
                    memory_reported: Arc::new(AtomicBool::new(false)),
                    memory_task: Some(memory_task),
                    memory_guard: Some(memory_guard),
                }
            };
            // debug_log!("Time to create profile: {}ms", start.elapsed().as_millis());

            // Register this profile with the new ProfileRegistry
            // First log the details to avoid potential deadlock
            debug_log!(
                "About to register profile in module {} for fn {} with line range {:?}..None",
                file_name_stem,
                fn_name,
                start_line
            );

            // Flush logs before calling register_profile
            // flush_debug_log();

            // Now register the profile if full_profiling is enabled
            #[cfg(feature = "full_profiling")]
            register_profile(&profile);

            // Log again after registration completes
            debug_log!(
                "Successfully registered profile in module {}",
                &profile.file_name
            );

            profile.start = Some(Instant::now());

            Some(profile)
        }
    }

    /// The defining path for this profile, as a `Vec` of `String`s
    #[must_use]
    pub const fn path(&self) -> &Vec<String> {
        &self.path
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
    pub fn write_profile_event(
        path: &str,
        file: &Mutex<Option<BufWriter<File>>>,
        entry: &str,
    ) -> ProfileResult<()> {
        let mut guard = file.lock();
        if guard.is_none() {
            *guard = Some(BufWriter::new(
                OpenOptions::new().create(true).append(true).open(path)?,
            ));
        }

        if let Some(writer) = guard.as_mut() {
            writeln!(writer, "{entry}")?;
            writer.flush()?;
        }
        // debug_log!("Wrote entry {entry} to {path} for {guard:?}");
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
            debug_log!(
                "DEBUG: Not writing time event for stack: {:?} due to zero duration",
                self.path
            );
            return Ok(());
        }

        let path = &self.path;

        if path.is_empty() {
            debug_log!("DEBUG: Stack is empty for {:?}", self.section_name);
            return Err(ProfileError::General("Stack is empty".into()));
        }

        // debug_log!("DEBUG: write_time_event for stack: {:?}", path);

        // eprintln!(
        //     "Backtrace for section::print_docs:\n{:#?}",
        //     Backtrace::new()
        // );
        let stack = self.build_stack(path);

        // Add our custom section name to the end of the stack path if present
        // let stack = self.append_section_to_stack(path.clone());

        let entry = format!("{stack} {micros}");

        // First write to the inclusive time file (always)
        let paths = ProfilePaths::get();
        let inclusive_time_path = &paths.inclusive_time;
        Self::write_profile_event(inclusive_time_path, InclusiveTimeProfileFile::get(), &entry)?;

        // Also write to the main time file (will be overwritten if converting to exclusive time)
        let time_path = get_time_path()?;
        Self::write_profile_event(time_path, TimeProfileFile::get(), &entry)
    }

    #[cfg(feature = "full_profiling")]
    fn write_memory_event(&self, delta: usize, op: char) -> ProfileResult<()> {
        if delta == 0 {
            // Keep this as it's a business logic check
            debug_log!(
                "DEBUG: Not writing memory event for stack: {:?} due to zero delta",
                self.path
            );
            return Ok(());
        }

        let path = &self.path;

        if path.is_empty() {
            return Err(ProfileError::General("Stack is empty".into()));
        }

        let stack = self.build_stack(path);

        let entry = format!("{stack} {}{delta}", if op == '-' { "-" } else { "" });

        debug_log!(
            "DEBUG: task_id: {} section_name: {:?} write_memory_event: {entry}",
            self.memory_task.as_ref().unwrap().id(),
            self.section_name
        );

        // let paths = ProfilePaths::get();
        let memory_path = get_memory_path()?;
        Self::write_profile_event(memory_path, MemoryProfileFile::get(), &entry)
    }

    /// Converts function names in the stack into their descriptive names.
    #[cfg(feature = "time_profiling")]
    #[must_use]
    pub fn build_stack(&self, path: &[String]) -> std::string::String {
        // let mut vanilla_stack = String::new();

        // path.iter()
        //     .map(|fn_name_str| {
        //         let stack_str = if vanilla_stack.is_empty() {
        //             fn_name_str.to_string()
        //         } else {
        //             format!("{vanilla_stack};{fn_name_str}")
        //         };
        //         vanilla_stack.clone_from(&stack_str);
        //         (stack_str, fn_name_str)
        //     })
        //     .map(|(stack_str, fn_name_str)| {
        //         get_reg_desc_name(&stack_str).unwrap_or_else(|| fn_name_str.to_string())
        //     })
        //     .chain(self.section_name.clone())
        //     .collect::<Vec<String>>()
        //     .join(";")
        build_stack(path, self.section_name.as_ref(), ";")
    }

    #[cfg(feature = "full_profiling")]
    #[allow(clippy::branches_sharing_code)]
    fn record_memory_change(&self, delta: usize) {
        if delta == 0 {
            return;
        }

        debug_log!(
            "Recording memory change: delta={}, profile={}, detailed_memory={}",
            delta,
            self.registered_name(),
            self.detailed_memory()
        );

        // Record allocation
        let result = self.write_memory_event(delta, '+');

        if let Err(ref e) = result {
            debug_log!("Error writing memory event: {:?}", e);
        } else {
            debug_log!("Successfully wrote memory event for delta={}", delta);
        }
    }

    /// Returns the profile type for this profile
    #[must_use]
    pub const fn get_profile_type(&self) -> ProfileType {
        self.profile_type
    }

    /// Returns whether this profile uses detailed memory tracking
    #[must_use]
    pub const fn is_detailed_memory(&self) -> bool {
        self.detailed_memory
    }
}

/// Converts function names in the stack into their descriptive names.
#[cfg(feature = "time_profiling")]
#[must_use]
pub fn build_stack(
    path: &[String],
    maybe_section_name: Option<&String>,
    sep: &str,
) -> std::string::String {
    safe_alloc! {
        let mut vanilla_stack = String::new();

        path.iter()
            .map(|fn_name_str| {
                let stack_str = if vanilla_stack.is_empty() {
                    fn_name_str.to_string()
                } else {
                    format!("{vanilla_stack};{fn_name_str}")
                };
                vanilla_stack.clone_from(&stack_str);
                (stack_str, fn_name_str)
            })
            .map(|(stack_str, fn_name_str)| {
                get_reg_desc_name(&stack_str).unwrap_or_else(|| fn_name_str.to_string())
            })
            .chain(maybe_section_name.cloned())
            .collect::<Vec<String>>()
            .join(sep)
    }
}

/// Extracts the call path from a cleaned stack trace.
///
/// This function processes a cleaned stack trace and builds a path of profiled functions,
/// filtering out non-profiled functions and handling optional append names.
///
/// # Arguments
/// * `cleaned_stack` - A slice of function names from the stack trace
/// * `maybe_append` - An optional function name to append to the path
///
/// # Returns
/// A vector of strings representing the call path of profiled functions
#[cfg(feature = "time_profiling")]
#[must_use]
pub fn extract_path(cleaned_stack: &[String], maybe_append: Option<&String>) -> Vec<String> {
    #[cfg(all(not(feature = "full_profiling"), feature = "time_profiling"))]
    {
        let dup = maybe_append
            .and_then(|append| cleaned_stack.first().map(|first| first == append))
            == Some(true);
        let start = usize::from(dup);
        cleaned_stack[start..]
            .iter()
            // Reverse the path so it goes from root caller to current function
            .rev()
            // Add path elements that make for a registered function name stack
            .fold(vec![], |stack: Vec<String>, fn_name_str| {
                let new_vec: Vec<String> = stack.iter().chain(Some(fn_name_str)).cloned().collect();
                let stack_str = new_vec.join(";");
                if is_profiled_function(&stack_str) {
                    new_vec
                } else {
                    stack
                }
            })
            .iter()
            .chain(maybe_append)
            .cloned()
            .collect()
    }

    #[cfg(feature = "full_profiling")]
    safe_alloc! {
        let dup = maybe_append.and_then(|append| cleaned_stack.first().map(|first| first == append))
            == Some(true);
        let start = usize::from(dup);

        let mut stack = Vec::new();
        let mut stack_str = String::new();
        // Manual loop instead of iterator chain
        for frame in cleaned_stack[start..].iter().rev() {
            let mut temp_stack_str = stack_str.clone();
            if !temp_stack_str.is_empty() {
                temp_stack_str.push(';');
            }
            temp_stack_str.push_str(frame);
            if is_profiled_function(&temp_stack_str) {
                stack.push(frame.to_string());
                if !stack_str.is_empty() {
                    stack_str.push(';');
                }
                stack_str.push_str(frame);
                // assert_eq!(stack.join(";"), stack_str);
                debug_log!("frame={frame}, stack_str={stack_str}");
            }
        }
        // Handle the optional append (replacing maybe_append.into_iter())
        if let Some(append_name) = maybe_append {
            stack.push(append_name.to_string());
        }
        stack
    }
}

/// Filter out backtrace lines identified as scaffolding.
#[cfg(feature = "time_profiling")]
#[must_use]
pub fn filter_scaffolding(name: &str) -> bool {
    !name.starts_with("tokio::") && !SCAFFOLDING_PATTERNS.iter().any(|s| name.contains(s))
}

// #[cfg(all(not(feature = "full_profiling"), feature = "time_profiling"))]
// pub fn extract_profile_callstack(
//     start_pattern: &str,
//     current_backtrace: &mut Backtrace,
// ) -> Vec<String> {
//     // First, collect all relevant frames
//     let callstack: Vec<String> = Backtrace::frames(current_backtrace)
//         .iter()
//         .flat_map(BacktraceFrame::symbols)
//         .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
//         .skip_while(|name| !name.contains(start_pattern) || name.contains("{{closure}}"))
//         // Be careful, this is very sensitive to changes in the function signatures of this module.
//         .skip(1)
//         .take_while(|name| !name.contains(end_point))
//         .filter(|name| filter_scaffolding(name))
//         .map(strip_hex_suffix)
//         .map(|mut name| {
//             // Remove hash suffixes and closure markers to collapse tracking of closures into their calling function
//             clean_function_name(&mut name)
//         })
//         // TODO May be problematic? - this will collapse legitimate nesting, but protects against recursion
//         .filter(|name| {
//             // Skip duplicate function calls (helps with the {{closure}} pattern)
//             if already_seen.contains(name.as_str()) {
//                 false
//             } else {
//                 already_seen.insert(name.clone());
//                 true
//             }
//         })
//         // .map(|(_, name)| name.clone())
//         .collect();
//     // debug_log!("Callstack: {:#?}", callstack);
//     // debug_log!("already_seen: {:#?}", already_seen);
//     callstack
// }

/// Extracts the callstack for a `Profile`.
///
/// # Panics
///
/// Panics if the arbitrary limit of 20 frames is exceeded.
#[cfg(feature = "time_profiling")]
#[must_use]
pub fn extract_profile_callstack(
    start_pattern: &str,
    // current_backtrace: &mut Backtrace,
) -> Vec<String> {
    // safe_alloc!(current_backtrace.resolve());
    /*
    let mut already_seen = safe_alloc!(HashSet::new());

    // let end_point = "__rust_begin_short_backtrace";
    let end_point = safe_alloc!(get_base_location().unwrap_or("__rust_begin_short_backtrace"));
    // debug_log!("end_point={end_point}");
    let mut start = safe_alloc!(false);
    let mut callstack = safe_alloc!(vec![]);

    for frame in Backtrace::frames(current_backtrace) {
        'inner: for symbol in frame.symbols() {
            let maybe_symbol_name = safe_alloc!(symbol.name());
            let Some(symbol_name) = maybe_symbol_name else {
                continue;
            };
            let name = safe_alloc!(symbol_name.to_string());
            if !start && (!name.contains(start_pattern) || name.contains("{{closure}}")) {
                continue;
            }
            if !start && name.contains(start_pattern) && !name.contains("{{closure}}") {
                start = true;
                continue;
            }
            if name.contains(end_point) {
                break;
            }
            if name.starts_with("tokio::") {
                continue;
            }
            for &s in SCAFFOLDING_PATTERNS {
                if name.contains(s) {
                    continue 'inner;
                }
            }
            let mut name = safe_alloc!(strip_hex_suffix(name));
            let name = safe_alloc!(clean_function_name(&mut name));
            if already_seen.contains(name.as_str()) {
                continue;
            }
            let name_clone = safe_alloc!(name.clone());
            safe_alloc!(already_seen.insert(name_clone));
            safe_alloc!(callstack.push(name));
        }
    }
    */

    let end_point = safe_alloc!(get_base_location().unwrap_or("__rust_begin_short_backtrace"));
    let mut already_seen = safe_alloc!(HashSet::new());
    safe_alloc! {
        // Pre-allocate with fixed capacity to avoid reallocations
        let capacity = 20;
        let mut callstack: Vec<String> = Vec::with_capacity(capacity); // Fixed size, no growing
        // let mut found_recursion = false;
        let mut start = false;
        let mut fin = false;
        let mut i = 0;

        trace(|frame| {
            let mut suppress = false;

            resolve_frame(frame, |symbol| {

                'process_symbol: {
                    let Some(name) = symbol.name() else {
                        suppress = true;
                        break 'process_symbol;
                    };
                    let name = name.to_string();
                    if name.contains("tokio") {
                        suppress = true;
                        break 'process_symbol;
                    }
                    if !start {
                        if name.contains(start_pattern) && !name.contains("{{closure}}") {
                            start = true;
                        }
                        suppress = true;
                        break 'process_symbol;
                    }
                    if name.contains(end_point) {
                        fin = true;
                        suppress = true;
                        break 'process_symbol;
                    }

                    for &s in SCAFFOLDING_PATTERNS {
                        if name.contains(s) {
                            suppress = true;
                            break 'process_symbol;
                        }
                    }

                    let mut name = strip_hex_suffix_slice(&name);
                    let name = clean_function_name(&mut name);
                    if already_seen.contains(&name) {
                        suppress = true;
                        break 'process_symbol;
                    }
                    already_seen.insert(name.clone());
                    if suppress { break 'process_symbol; }

                    // // Check for our own functions (recursion detection)
                    // if i > 0 && name.contains("extract_profile_callstack") {
                    //     found_recursion = true;
                    //     break 'process_symbol;
                    // }

                    // Safe to unwrap now
                    callstack.push(name);
                    i += 1;
                    if i >= capacity {
                        safe_alloc! {
                             println!("frames={callstack:#?}");
                         };
                         panic!("Max limit of {capacity} frames exceeded");
                    }
                }
            });
            !fin
        });
        callstack
    }
}

/// Extracts the callstack for memory deallocation tracking.
///
/// This function captures the current call stack starting from a pattern match
/// and builds a vector of function names with line numbers for detailed memory
/// deallocation tracking.
///
/// # Arguments
/// * `start_pattern` - A regex pattern to identify where to start capturing the stack
///
/// # Returns
/// A vector of strings in the format "function_name:line_number" representing
/// the call stack, or an empty vector if recursion is detected
///
/// # Panics
/// Panics if the arbitrary limit of 20 frames is exceeded during stack traversal
#[cfg(feature = "full_profiling")]
#[must_use]
#[allow(clippy::missing_panics_doc)]
#[fn_name]
pub fn extract_dealloc_callstack(start_pattern: &Regex) -> Vec<String> {
    let mut already_seen = HashSet::new();

    // let end_point = "__rust_begin_short_backtrace";
    let end_point = get_base_location().unwrap_or("__rust_begin_short_backtrace");

    // First, collect all relevant frames
    /*
    let callstack: Vec<String> = Backtrace::frames(current_backtrace)
        .iter()
        .flat_map(BacktraceFrame::symbols)
        .map(|symbol| {
            (
                symbol.name().map(|name| name.to_string()),
                symbol.lineno().unwrap_or(0),
            )
        })
        .filter(|(maybe_frame, _)| maybe_frame.is_some())
        .map(|(maybe_frame, lineno)| (maybe_frame.unwrap(), lineno))
        .skip_while(|(frame, _)| !start_pattern.is_match(frame))
        .take_while(|(frame, _)| !frame.contains(end_point))
        .filter(|(frame, _)| filter_scaffolding(frame))
        // .inspect(|frame| {
        //     debug_log!("frame: {frame}");
        // })
        .map(|(frame, lineno)| (strip_hex_suffix(frame), lineno))
        .map(|(mut name, lineno)| {
            // Remove hash suffixes and closure markers to collapse tracking of closures into their calling function
            (clean_function_name(&mut name), lineno)
        })
        .filter(|(name, _)| {
            // Skip duplicate function calls (helps with the {{closure}} pattern)
            if already_seen.contains(name.as_str()) {
                false
            } else {
                already_seen.insert(name.clone());
                true
            }
        })
        .map(|(frame, lineno)| format!("{frame}:{lineno}"))
        .collect();
    */

    safe_alloc! {
        // Pre-allocate with fixed capacity to avoid reallocations
        let capacity = 20;
        let mut callstack: Vec<String> = Vec::with_capacity(capacity); // Fixed size, no growing
        let mut found_recursion = false;
        let mut start = false;
        let mut fin = false;
        let mut i = 0;

        trace(|frame| {
            let mut suppress = false;

            resolve_frame(frame, |symbol| {

                'process_symbol: {
                    let Some(name) = symbol.name() else {
                        suppress = true;
                        break 'process_symbol;
                    };
                    let name = name.to_string();
                    if name.contains("tokio") {
                        suppress = true;
                        break 'process_symbol;
                    }
                    if !start {
                        if start_pattern.is_match(&name) {
                            start = true;
                        }
                        suppress = true;
                        break 'process_symbol;
                    }
                    if name.contains(end_point) {
                        fin = true;
                        suppress = true;
                        break 'process_symbol;
                    }

                    for &s in SCAFFOLDING_PATTERNS {
                        if name.contains(s) {
                            suppress = true;
                            break 'process_symbol;
                        }
                    }

                    let mut name = strip_hex_suffix_slice(&name);
                    let name = clean_function_name(&mut name);
                    if already_seen.contains(&name) {
                        suppress = true;
                        break 'process_symbol;
                    }
                    already_seen.insert(name.clone());
                    if suppress { break 'process_symbol; }

                    // Check for our own functions (recursion detection)
                    if i > 0 && name.contains(fn_name) {
                        found_recursion = true;
                        break 'process_symbol;
                    }

                    // Safe to unwrap now
                    let entry = format!("{name}:{}", symbol.lineno().unwrap_or(0));
                    callstack.push(entry);
                    i += 1;
                    if i >= capacity {
                        safe_alloc! {
                             println!("frames={callstack:#?}");
                         };
                         panic!("Max limit of {capacity} frames exceeded");
                    }
                }
            });
            !found_recursion && !fin
        });
        if found_recursion {
            vec![]
        } else {
            callstack
        }
    }
}

/// .
///
/// # Panics
///
/// Panics if arbitrary preset limit of 100 frames exceeded.
#[cfg(feature = "full_profiling")]
#[must_use]
#[fn_name]
pub fn extract_detailed_alloc_callstack(start_pattern: &Regex) -> Vec<String> {
    let mut already_seen = HashSet::new();

    let end_point = "__rust_begin_short_backtrace";

    // First, collect all relevant frames
    /*
    let callstack: Vec<String> = Backtrace::frames(current_backtrace)
        .iter()
        .flat_map(BacktraceFrame::symbols)
        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
        .skip_while(|frame| !start_pattern.is_match(frame))
        .skip(1)
        .take_while(|frame| !frame.contains(end_point))
        // .inspect(|frame| {
        //     debug_log!("frame: {frame}");
        // })
        .map(strip_hex_suffix)
        .map(|mut name| {
            // Remove hash suffixes and closure markers to collapse tracking of closures into their calling function
            clean_function_name(&mut name)
        })
        .filter(|name| {
            // Skip duplicate function calls (helps with the {{closure}} pattern)
            if already_seen.contains(name.as_str()) {
                false
            } else {
                already_seen.insert(name.clone());
                true
            }
        })
        .collect();
     */

    let maybe_callstack: Option<Vec<String>> = safe_alloc! {
        // Pre-allocate with fixed capacity to avoid reallocations
        let capacity = 100;
        let mut callstack: Vec<String> = Vec::with_capacity(capacity); // Fixed size, no growing
        let mut found_recursion = false;
        let mut start = false;
        let mut fin = false;
        let mut i = 0;

        trace(|frame| {
            let mut suppress = false;

            resolve_frame(frame, |symbol| {

                'process_symbol: {
                    let Some(name) = symbol.name() else {
                        suppress = true;
                        break 'process_symbol;
                    };
                    let name = name.to_string();
                    if !start {
                        if start_pattern.is_match(&name) {
                            start = true;
                        }
                        suppress = true;
                        break 'process_symbol;
                    }
                    if name.contains(end_point) {
                        fin = true;
                        suppress = true;
                        break 'process_symbol;
                    }

                    let mut name = strip_hex_suffix_slice(&name);
                    let name = clean_function_name(&mut name);
                    if already_seen.contains(&name) {
                        suppress = true;
                        break 'process_symbol;
                    }
                    already_seen.insert(name.clone());
                    if suppress { break 'process_symbol; }

                    // Check for our own functions (recursion detection)
                    if i > 0 && name.contains(fn_name) {
                        found_recursion = true;
                        break 'process_symbol;
                    }

                    // Safe to unwrap now
                    callstack.push(name);
                    i += 1;
                    if i >= capacity {
                        safe_alloc! {
                             println!("frames={callstack:#?}");
                         };
                         panic!("Max limit of {capacity} frames exceeded");
                    }
                }
            });
            !found_recursion && !fin
        });
        if found_recursion {
            None // Signal to skip tracking
        } else {
            Some(callstack)
        }
    };

    // debug_log!("Callstack: {callstack:#?}");
    // debug_log!("already_seen: {:#?}", already_seen);

    // Redefine end-point as inclusive
    // let end_point = get_root_module().unwrap_or("__rust_begin_short_backtrace");

    let Some(callstack) = maybe_callstack else {
        return vec![];
    };

    callstack
        .iter()
        .rev()
        // .skip_while(|frame| !frame.contains(end_point))
        .cloned()
        .collect()
}

// Global thread-safe BTreeSet
static GLOBAL_CALL_STACK_ENTRIES: std::sync::LazyLock<Mutex<BTreeSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(BTreeSet::new()));

/// Prints all entries in the global `BTreeSet`.
/// Entries are printed in sorted order (alphabetically).
pub fn print_all_call_stack_entries() {
    let parts = { GLOBAL_CALL_STACK_ENTRIES.lock().clone() };
    debug_log!("All entries in the global set (sorted):");
    if parts.is_empty() {
        debug_log!("  (empty set)");
    } else {
        for part in &parts {
            debug_log!("  {part}");
        }
    }
    debug_log!("Total entries: {}", parts.len());
}

#[allow(dead_code)]
fn get_fn_desc_name(fn_name_str: &String) -> String {
    // extract_fn_only(fn_name_str).map_or_else(|| fn_name_str.to_string(), |fn_only| fn_only)
    extract_fn_only(fn_name_str).unwrap_or_else(|| fn_name_str.to_string())
}

#[cfg(all(not(feature = "full_profiling"), feature = "time_profiling"))]
impl Drop for Profile {
    fn drop(&mut self) {
        // debug_log!("In drop for Profile {:?}", self);
        let drop_start = Instant::now();

        if let Some(start) = self.start.take() {
            // Handle time profiling as before
            match self.profile_type {
                ProfileType::Time | ProfileType::Both => {
                    let elapsed = start.elapsed();
                    let _ = self.write_time_event(elapsed);
                }
                ProfileType::Memory | ProfileType::None => todo!(),
            }
        }
        debug_log!(
            "Time to drop profile: {}ms",
            drop_start.elapsed().as_millis()
        );
        flush_debug_log();
    }
}

#[cfg(feature = "full_profiling")]
impl Drop for Profile {
    #[allow(clippy::branches_sharing_code)]
    fn drop(&mut self) {
        safe_alloc! {
            // Capture the information needed for deregistration but use it only at the end
            #[cfg(feature = "full_profiling")]
            let instance_id = self.instance_id();

            // debug_log!("In drop for Profile {:?}", self);
            let drop_start = Instant::now();
            if let Some(start) = self.start.take() {
                // Handle time profiling as before
                match self.profile_type {
                    ProfileType::Time | ProfileType::Both => {
                        // debug_log!("In drop for Profile {:?}", self);
                        if matches!(
                            get_global_profile_type(),
                            ProfileType::Time | ProfileType::Both
                        ) {
                            let elapsed = start.elapsed();
                            let _ = self.write_time_event(elapsed);
                        }
                    }
                    ProfileType::Memory | ProfileType::None => (),
                }
            }
            debug_log!(
                "Time to write event: {}ms",
                drop_start.elapsed().as_millis()
            );

            // For memory profiling, use our direct counter
            if matches!(self.profile_type, ProfileType::Memory | ProfileType::Both) {
                if self
                    .memory_reported
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    let total_allocated = self.allocation_total.load(Ordering::Relaxed);
                    if total_allocated > 0 {
                        debug_log!(
                            "Writing memory allocation of {total_allocated} bytes for profile {}",
                            self.registered_name
                        );
                        self.record_memory_change(total_allocated);
                    } else {
                        debug_log!(
                            "0-byte memory allocation not recorded for profile {}",
                            self.registered_name
                        );
                    }
                } else {
                    debug_log!(
                        "Skipping memory write for profile {} - already reported",
                        self.registered_name
                    );
                }
            }
            debug_log!(
                "Time to drop profile: {}ms",
                drop_start.elapsed().as_millis()
            );
            // flush_debug_log();

            // After all processing is done, signal that the profile should be deregistered
            // instead of trying to do it ourselves
            #[cfg(feature = "full_profiling")]
            {
                debug_log!("Requesting deregistration of profile instance {instance_id}");
                // flush_debug_log();

                // Use deregister_profile which is now safe due to our changes
                deregister_profile(self);
            }
        };
    }
}

/// Converts an inclusive time profile to exclusive time
///
/// This function reads a folded file with inclusive times (total time spent in each function)
/// and converts it to exclusive times (time spent only in the function itself, excluding child calls).
/// # Errors
///
/// This function will bubble up any i/o errors that occur trying to convert the file.
#[cfg(feature = "time_profiling")]
#[allow(clippy::branches_sharing_code, unused_assignments)]
pub fn convert_to_exclusive_time(input_path: &str, output_path: &str) -> ProfileResult<()> {
    debug_log!("Converting inclusive time profile to exclusive time");

    // Read input file
    let file = File::open(input_path)
        .map_err(|e| ProfileError::General(format!("Failed to open input file: {e}")))?;
    let reader = BufReader::new(file);

    // // Store header lines to preserve them
    // let mut header_lines = Vec::new();

    // Store stack lines as (stack_str, time) pairs
    let mut stack_lines: Vec<(String, u64)> = Vec::new();
    let mut input_lines = 0;

    // First pass: Parse the file and separate headers from stack lines
    for (line_count, line) in reader.lines().enumerate() {
        input_lines = line_count;
        let line = line.map_err(|e| ProfileError::General(format!("Failed to read line: {e}")))?;
        // // Preserve comment/header lines
        // if line.starts_with('#') || line.trim().is_empty() {
        //     header_lines.push(line);
        //     continue;
        // }

        // Parse line: "stack time"
        let parts: Vec<&str> = line.rsplitn(2, ' ').collect();
        if parts.len() != 2 {
            debug_log!("Warning: Invalid line format at line {line_count}: {line}");
            continue;
        }

        let stack_str = parts[1].trim();
        let time = match parts[0].parse::<u64>() {
            Ok(t) => t,
            Err(e) => {
                debug_log!("Warning: Invalid time value at line {line_count}: {e}");
                continue;
            }
        };

        // Store the stack line
        stack_lines.push((stack_str.to_string(), time));
    }

    // Process in reverse order
    let mut stack_lines: Vec<(String, u64)> = stack_lines.into_iter().rev().collect();

    // Calculate exclusive times using a sequential approach
    let mut exclusive_times: Vec<(String, u64)> = vec![];
    let len = stack_lines.len();

    for _i in 1..=len {
        // Process each stack, moving it from the stack_lines to exclusive_times
        let mut parent = stack_lines.remove(0);

        // For each stack, find its direct descendants and subtract their inclusive time from the parent
        for (candidate, time_ref) in &mut stack_lines {
            if !candidate.starts_with(&parent.0) {
                break;
            }
            let parts: Vec<&str> = candidate.split(';').collect();
            let parent_stack = parts[..parts.len() - 1].join(";");
            if parent_stack == parent.0 {
                parent.1 = parent.1.saturating_sub(*time_ref);
            }
        }
        exclusive_times.push(parent);
    }

    // Restore original order
    let exclusive_times: Vec<(String, u64)> = exclusive_times.into_iter().rev().collect();

    // Write output file
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)
        .map_err(|e| ProfileError::General(format!("Failed to create output file: {e}")))?;
    let mut writer = BufWriter::new(output_file);

    // // Write original headers
    // for header in &header_lines {
    //     writeln!(writer, "{header}")
    //         .map_err(|e| ProfileError::General(format!("Failed to write header: {e}")))?;
    // }

    // // Add a note about this being exclusive time
    // writeln!(writer, "# Converted to exclusive time by thag_profiler")
    //     .map_err(|e| ProfileError::General(format!("Failed to write header: {e}")))?;
    // writeln!(writer).map_err(|e| ProfileError::General(format!("Failed to write newline: {e}")))?;

    for (stack, exclusive) in &exclusive_times {
        writeln!(writer, "{stack} {exclusive}")
            .map_err(|e| ProfileError::General(format!("Failed to write stack line: {e}")))?;
    }

    writer
        .flush()
        .map_err(|e| ProfileError::General(format!("Failed to flush writer: {e}")))?;

    debug_log!("Successfully processed {input_lines} lines");
    debug_log!("Found {len} stacks");

    // Sum up exclusive times to validate
    let total_exclusive: u64 = exclusive_times.iter().map(|(_, time)| *time).sum();
    debug_log!("Total exclusive time: {total_exclusive} µs");
    debug_log!("Successfully converted time profile from inclusive to exclusive time");

    Ok(())
}

/// Enable or disable exclusive time conversion
#[cfg(feature = "time_profiling")]
pub fn set_convert_to_exclusive_time(enable: bool) {
    CONVERT_TO_EXCLUSIVE_TIME.store(enable, Ordering::SeqCst);
}

/// Check if exclusive time conversion is enabled
#[cfg(feature = "time_profiling")]
pub fn is_convert_to_exclusive_time_enabled() -> bool {
    CONVERT_TO_EXCLUSIVE_TIME.load(Ordering::SeqCst)
}

/// Perform the conversion from inclusive to exclusive time if enabled
/// # Errors
///
/// This function will bubble up any i/o errors that occur trying to convert the file.
#[cfg(feature = "time_profiling")]
pub fn process_time_profile() -> ProfileResult<()> {
    if is_convert_to_exclusive_time_enabled() {
        let paths = ProfilePaths::get();
        let inclusive_path = &paths.inclusive_time;
        let exclusive_path = &paths.time;

        // Check if the inclusive time file exists and has content
        if !inclusive_path.is_empty() {
            let metadata = std::fs::metadata(inclusive_path).map_err(|e| {
                ProfileError::General(format!("Failed to check inclusive time file: {e}"))
            })?;

            if metadata.len() > 0 {
                debug_log!("Converting inclusive time profile to exclusive time");
                convert_to_exclusive_time(inclusive_path, exclusive_path)?;
            }
        }
    }
    Ok(())
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
                // debug_log!("Backtrace contains pattern: {pattern}, line: {line}");
                return true;
            }
        }
    }

    false
}

/// Register a function name with the profiling registry
///
/// # Panics
///
/// Panics if it finds the name "new", which shows that the inclusion of the
/// type in the method name is not working.
pub fn register_profiled_function(name: &str, desc_name: &str) {
    // let start = safe_alloc!(Instant::now());
    #[cfg(all(debug_assertions, not(test)))]
    assert!(
        name != "new",
        "Logic error: `new` is not an accepted function name on its own. It must be qualified with the type name: `<Type>::new`. desc_name={desc_name}"
    );
    let name = safe_alloc!(name.to_string());
    let desc_name = safe_alloc!(desc_name.to_string());
    {
        if let Some(mut lock) = PROFILED_FUNCTIONS.try_write() {
            safe_alloc!(lock.insert(name, desc_name));
        } else {
            safe_alloc!(debug_log!(
                "register_profiled_function failed to acquire write lock on PROFILED_FUNCTIONS"
            ););
        }
    }
    // debug_log!(
    //     "register_profiled_function took {}ms",
    //     start.elapsed().as_millis()
    // );
}

/// Check if a function is registered for profiling
pub fn is_profiled_function(name: &str) -> bool {
    // debug_log!("Checking if function is profiled: {}", name);
    let contains_key = PROFILED_FUNCTIONS.try_read().map_or_else(
        || {
            debug_log!("is_profiled_function failed to acquire read lock on PROFILED_FUNCTIONS");
            false
        },
        |lock| lock.contains_key(name),
    );
    // debug_log!("...done");
    contains_key
}

/// Get the descriptive name of a profiled function
pub fn get_reg_desc_name(name: &str) -> Option<String> {
    safe_alloc! {
        let maybe_reg_desc_name = PROFILED_FUNCTIONS.try_read().map_or_else(
            || {
                debug_log!("get_reg_desc_name failed to acquire read lock on PROFILED_FUNCTIONS");
                None
            },
            |lock| lock.get(name).cloned(),
        );
        // debug_log!("...done");
        maybe_reg_desc_name
    }
}

// Extract just the base function name from a fully qualified function name
// #[cfg(feature = "time_profiling")]
fn extract_fn_only(qualified_name: &str) -> Option<String> {
    // Split by :: and get the last component
    qualified_name.rfind("::").map_or_else(
        || Some(qualified_name.to_string()),
        |pos| Some(qualified_name[(pos + 2)..].to_string()),
    )
}

#[cfg(feature = "time_profiling")]
const SCAFFOLDING_PATTERNS: &[&str] = &[
    // "::main::",
    "::poll::",
    "::poll_next_unpin",
    "<F as core::future::future::Future>::poll",
    "FuturesOrdered<Fut>",
    "FuturesUnordered<Fut>",
    "ProfiledFuture",
    "ProfileSection",
    "__rust_alloc",
    "__rust_realloc",
    "__rust_try",
    "alloc::",
    "core::",
    "core::ops::function::FnOnce::call_once",
    "hashbrown",
    "mem_tracking::with_sys_alloc",
    "mio::",
    "std::panic::catch_unwind",
    "std::panicking",
    "std::rt::lang_start",
    "std::sync::poison::",
    "std::sys::backtrace::__rust_begin_short_backtrace",
    "std::sys::sync::",
    "std::sys::thread_local",
    "std::thread",
    "mem_tracking::MultiAllocator::with",
    // "Profile::new",
];

/// Normalises function names by removing closure references and hash suffixes.
pub fn clean_function_name(clean_name: &mut str) -> String {
    // Remove any closure markers
    let mut clean_name: &mut str = if let Some(closure_pos) = clean_name.find("::{{closure}}") {
        // index = closure_pos;
        &mut clean_name[..closure_pos]
    } else if let Some(hash_pos) = clean_name.rfind("::h") {
        // Find and remove hash suffixes (::h followed by hex digits)
        // from the last path segment
        if clean_name[hash_pos + 3..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        {
            &mut clean_name[..hash_pos]
        } else {
            clean_name
        }
    } else {
        clean_name
    };
    // .to_string();

    while clean_name.ends_with("::") {
        let len = clean_name.len();
        clean_name = &mut clean_name[..len - 2];
    }

    let mut clean_name = (*clean_name).to_string();

    // Clean up any double colons that might be left
    while clean_name.contains("::::") {
        clean_name = clean_name.replace("::::", "::");
    }

    clean_name
}

// Optional: add memory info to error handling
#[derive(Debug)]
/// Errors related to memory profiling operations
pub enum MemoryError {
    /// Memory statistics are not available
    StatsUnavailable,
    /// Failed to calculate memory delta
    DeltaCalculationFailed,
}

#[derive(Default)]
/// Statistics for profiled functions, tracking performance metrics.
///
/// This struct maintains both per-function statistics (`calls` and `total_time`)
/// and legacy aggregate statistics for backwards compatibility.
pub struct ProfileStats {
    /// Number of calls made to each profiled function
    pub calls: HashMap<String, u64>,
    /// Total execution time in microseconds for each profiled function
    pub total_time: HashMap<String, u128>,
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

// #[cfg(feature = "full_profiling")]
// pub fn force_gc() {
//     // Allocate and immediately drop a large object to encourage GC
//     let pressure = vec![0u8; 1024 * 1024];
//     drop(pressure);

//     // Sleep briefly to give the system time to process deallocations
//     std::thread::sleep(std::time::Duration::from_millis(10));
// }

// #[cfg(not(feature = "full_profiling"))]
// pub const fn force_gc() {}

/// Dumps the contents of the profiled functions registry for debugging purposes
///
/// This function is primarily intended for test and debugging use.
#[cfg(any(test, debug_assertions))]
pub fn dump_profiled_functions() -> Vec<(String, String)> {
    let hash_map = { PROFILED_FUNCTIONS.read().clone() };
    hash_map
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

#[cfg(test)]
static TEST_MODE_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Test utilities module
///
/// This provides internal functions for testing only.
///
/// Note: All test initialization code should now use the #[enable_profiling] attribute
/// instead of programmatic enable_profiling function calls

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

// Test state manipulation utilities removed.
// All testing code should now use the #[enable_profiling] attribute or disable_profiling()

// Helper functions for tests that can't use attribute macros

/// Force enables time profiling for tests without using attribute macros
///
/// This is a simplified helper for test code that can't easily use attribute macros
#[cfg(test)]
#[cfg(feature = "time_profiling")]
pub fn force_enable_profiling_time_for_tests() {
    use std::sync::atomic::Ordering;

    // Set test mode active to prevent #[profiled] from creating duplicate entries
    TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

    // Enable profiling by directly setting state variables
    PROFILING_STATE.store(true, Ordering::SeqCst);

    // Set the profile type
    set_global_profile_type(ProfileType::Time);

    // Initialize files if needed
    let _ = initialize_profile_files(ProfileType::Time);
}

/// Force enables memory profiling for tests without using attribute macros
///
/// This is a simplified helper for test code that can't easily use attribute macros
#[cfg(all(test, feature = "full_profiling"))]
pub fn force_enable_profiling_memory_for_tests() {
    use std::sync::atomic::Ordering;

    // Set test mode active to prevent #[profiled] from creating duplicate entries
    TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

    // Enable profiling by directly setting state variables
    PROFILING_STATE.store(true, Ordering::SeqCst);

    // Set the profile type
    set_global_profile_type(ProfileType::Memory);

    // Initialize files if needed
    let _ = initialize_profile_files(ProfileType::Memory);
}

/// Force enables both time and memory profiling for tests without using attribute macros
///
/// This is a simplified helper for test code that can't easily use attribute macros
#[cfg(all(test, feature = "full_profiling"))]
pub fn force_enable_profiling_both_for_tests() {
    use std::sync::atomic::Ordering;

    // Set test mode active to prevent #[profiled] from creating duplicate entries
    TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

    // Enable profiling by directly setting state variables
    PROFILING_STATE.store(true, Ordering::SeqCst);

    // Set the profile type
    set_global_profile_type(ProfileType::Both);

    // Initialize files if needed
    let _ = initialize_profile_files(ProfileType::Both);
}

#[cfg(test)]
/// Safely cleans up profiling after a test
pub fn safely_cleanup_profiling_after_test() {
    // First disable profiling (use the public API here)
    disable_profiling();

    // Finally reset test mode flag
    use std::sync::atomic::Ordering;
    TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);
}

// Flag to determine whether to convert inclusive time profiles to exclusive time
#[cfg(feature = "time_profiling")]
static CONVERT_TO_EXCLUSIVE_TIME: AtomicBool = AtomicBool::new(true);

/// Strips hexadecimal suffixes from Rust function names.
///
/// This function removes hash suffixes (like `::h1234abcd`) that are added
/// by the Rust compiler for symbol disambiguation.
///
/// # Arguments
/// * `name` - The function name that may contain a hex suffix
///
/// # Returns
/// A `String` with the hex suffix removed, or the original name if no suffix was found
///
/// # Examples
/// ```
/// # use thag_profiler::strip_hex_suffix;
/// let name = "my_function::h1234abcd".to_string();
/// assert_eq!(strip_hex_suffix(name), "my_function");
///
/// let name = "no_suffix".to_string();
/// assert_eq!(strip_hex_suffix(name), "no_suffix");
/// ```
#[must_use]
pub fn strip_hex_suffix(name: String) -> String {
    if let Some(hash_pos) = name.rfind("::h") {
        if name[hash_pos + 3..].chars().all(|c| c.is_ascii_hexdigit()) {
            name[..hash_pos].to_string()
        } else {
            name
        }
    } else {
        name
    }
}

/// Strips hexadecimal suffixes from Rust function names.
///
/// This function removes hash suffixes (like `::h1234abcd`) that are added
/// by the Rust compiler for symbol disambiguation.
///
/// # Arguments
/// * `name` - The function name that may contain a hex suffix
///
/// # Returns
/// A `String` with the hex suffix removed, or the original name if no suffix was found
///
/// # Examples
/// ```
/// # use thag_profiler::strip_hex_suffix_slice;
/// let name = "my_function::h1234abcd";
/// assert_eq!(strip_hex_suffix_slice(name), "my_function");
///
/// let name = "no_suffix";
/// assert_eq!(strip_hex_suffix_slice(name), "no_suffix");
/// ```
#[must_use]
pub fn strip_hex_suffix_slice(name: &str) -> String {
    name.rfind("::h").map_or_else(
        || name.to_string(),
        |hash_pos| {
            if name[hash_pos + 3..].chars().all(|c| c.is_ascii_hexdigit()) {
                name[..hash_pos].to_string()
            } else {
                name.to_string()
            }
        },
    )
}

#[cfg(test)]
#[cfg(feature = "time_profiling")]
pub(crate) mod test_utils {
    //! This module contains utilities for testing profiling functionality.
    //! These are not part of the public API and are only used for internal tests.

    #[cfg(feature = "full_profiling")]
    use crate::ProfileType;

    /// Initializes profiling for tests
    ///
    /// This is an internal function provided for tests to initialize profiling
    /// without exposing the implementation details of how profiling is enabled.
    ///
    /// # Arguments
    /// * `profile_type` - The type of profiling to enable
    #[cfg(feature = "full_profiling")]
    pub fn initialize_profiling_for_test(profile_type: ProfileType) -> crate::ProfileResult<()> {
        use crate::profiling::{enable_profiling, TEST_MODE_ACTIVE};
        use std::sync::atomic::Ordering;

        // Set test mode active to prevent #[profiled] from creating duplicate entries
        TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

        // Then enable profiling using the internal function
        enable_profiling(true, Some(profile_type))
    }

    // /// Safely cleans up profiling after a test
    // pub fn cleanup_profiling_after_test() -> crate::ProfileResult<()> {
    //     // First disable profiling
    //     let result = enable_profiling(false, None);

    //     // Reset test mode flag
    //     TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);

    //     result
    // }

    // /// Force sets the profiling state for testing purposes
    // /// This is only used in tests to directly manipulate the profiling state
    // pub fn force_set_profiling_state(enabled: bool) {
    //     // This function is only used in tests to directly manipulate the profiling state
    //     PROFILING_STATE.store(enabled, Ordering::SeqCst);
    // }
}

#[cfg(test)]
mod tests_internal {
    use super::*;
    use regex::Regex;
    use serial_test::serial;
    // use std::env;
    use std::time::Duration;

    // Basic profiling tests

    #[test]
    #[serial]
    fn test_profiling_profile_type_from_str() {
        assert_eq!(ProfileType::from_str("time"), Ok(ProfileType::Time));
        assert_eq!(ProfileType::from_str("memory"), Ok(ProfileType::Memory));
        assert_eq!(ProfileType::from_str("both"), Ok(ProfileType::Both));
        assert_eq!(ProfileType::from_str("none"), Ok(ProfileType::None));
        assert_eq!(ProfileType::from_str(""), Ok(ProfileType::None));
        assert_eq!(
            ProfileType::from_str("invalid"),
            Err(
                "Invalid profile type 'invalid'. Expected 'time', 'memory', 'both', or 'none'"
                    .to_string()
            )
        );
    }

    // Function registry tests

    #[test]
    #[serial]
    fn test_profiling_function_registry() {
        // Register a function
        register_profiled_function("test_func", "test_desc");

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
    }

    // Profile type tests

    // Thread-safety tests

    // utils tests

    #[test]
    fn test_profiling_clean_function_name() {
        // Test with hash suffix
        let mut name = "module::func::h1234abcd".to_string();
        assert_eq!(clean_function_name(&mut name), "module::func");

        // Test with closure
        let mut name = "module::func::{{closure}}".to_string();
        assert_eq!(clean_function_name(&mut name), "module::func");

        // Test with both
        let mut name = "module::func::{{closure}}::h1234abcd".to_string();
        assert_eq!(clean_function_name(&mut name), "module::func");

        // Test with multiple colons
        let mut name = "module::::func".to_string();
        assert_eq!(clean_function_name(&mut name), "module::func");
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
        let re = Regex::new(r"\d{8}-\d{6}\.folded$").unwrap();
        assert!(
            re.is_match(&paths.time),
            "Time path should contain timestamp in YYYYmmdd-HHMMSS format"
        );
    }
}
