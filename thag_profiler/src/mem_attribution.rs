use crate::{
    debug_log, flush_debug_log, internal_doc,
    mem_tracking::write_detailed_stack_alloc,
    profiling::{clean_function_name, strip_hex_suffix_slice, Profile},
    regex, safe_alloc, static_lazy, ProfileError, ProfileResult,
};
use backtrace::{resolve_frame, trace};
use dashmap::{DashMap, DashSet};
use regex::Regex;
use std::{
    clone::Clone, collections::HashSet, convert::AsRef, ops::Range, string::ToString, sync::Arc,
};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct ProfileKey {
    pub module: String,
    pub function: String,
    pub line_range: Range<u32>,
}

impl ProfileKey {
    #[must_use]
    pub const fn new(module: String, function: String, start_line: u32, end_line: u32) -> Self {
        Self {
            module,
            function,
            line_range: start_line..end_line,
        }
    }

    #[must_use]
    pub fn contains_line(&self, line: u32) -> bool {
        self.line_range.contains(&line)
    }
}

/// A thread-safe registry for managing profile instances and their metadata.
///
/// The `ProfileRegistry` provides centralized storage and lookup capabilities for profiles,
/// supporting operations like registration, deregistration, and finding profiles by
/// module, function, and line number. It also tracks active tasks and provides
/// efficient cleanup through instance ID mapping.
pub struct ProfileRegistry {
    // Main profile storage - single flat map
    profiles: DashMap<ProfileKey, ProfileRef>,

    // Quick lookup: instance_id -> ProfileKey for cleanup
    instance_to_key: DashMap<u64, ProfileKey>,

    // Optional: Quick lookup by module for get_file_names
    modules: DashMap<String, ()>, // Just track module names

    // // Task tracking (restore the original approach)
    // active_tasks: DashMap<u64, usize>, // task_id -> some identifier
    // // or
    active_tasks: DashSet<usize>, // Just track active task IDs
}

impl Default for ProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileRegistry {
    /// Creates a new `ProfileRegistry` instance with empty collections
    #[must_use]
    pub fn new() -> Self {
        Self {
            profiles: DashMap::new(),
            instance_to_key: DashMap::new(),
            modules: DashMap::new(),
            active_tasks: DashSet::new(),
        }
    }

    /// Activates a task by adding its ID to the active tasks set
    pub fn activate_task(&self, task_id: usize) {
        self.active_tasks.insert(task_id); // or just insert if using DashSet
    }

    /// Deactivates a task by removing its ID from the active tasks set
    pub fn deactivate_task(&self, task_id: usize) {
        self.active_tasks.remove(&task_id);
    }

    // /// Returns a vector of all currently active task IDs
    // #[must_use]
    // pub fn get_active_tasks(&self) -> Vec<usize> {
    //     self.active_tasks.iter().map(|entry| *entry.key()).collect()
    // }

    /// Returns the highest active task ID, if any tasks are active
    #[must_use]
    pub fn get_last_active_task(&self) -> Option<usize> {
        // This might need more sophisticated logic for "max_by_key" equivalent
        self.active_tasks.iter().map(|entry| *entry.key()).max()
    }

    /// Register a profile with the registry
    ///
    /// # Errors
    ///
    /// This function will return an error if the `ProfileRef` does not reference a `Profile`.
    pub fn register_profile(&self, profile_ref: &ProfileRef) -> ProfileResult<()> {
        // Extract information from the ProfileRef and its contained Profile
        let instance_id = profile_ref.instance_id;
        let profile = profile_ref
            .profile()
            .ok_or_else(|| ProfileError::General("No profile found for ProfileRef".to_string()))?;
        // Extract module, function, and line info from the Profile
        // profile.file_name(), profile.detailed_memory(), profile.start_line(), profile.end_line(), profile.instance_id());
        let module_name = profile.file_name();
        // let function_name = profile.fn_name().clone();
        let start_line = profile.start_line();
        let end_line = profile.end_line();

        let key = ProfileKey::new(
            module_name.to_string(),
            profile.fn_name().to_string(),
            start_line.unwrap_or(0),
            end_line.unwrap_or(u32::MAX),
        );

        // Simple insertions - no nested locking
        self.profiles.insert(key.clone(), profile_ref.clone());
        self.instance_to_key.insert(instance_id, key);
        self.modules.insert(module_name.to_string(), ());

        Ok(())
    }

    /// Deregister a profile when it's dropped
    pub fn deregister_profile(
        &self,
        instance_id: u64,
        _file_name: &str,         // No longer needed
        _function_name: &str,     // No longer needed
        _start_line: Option<u32>, // No longer needed
        _end_line: Option<u32>,   // No longer needed
    ) {
        // Super simple cleanup - just two removals
        if let Some((_, key)) = self.instance_to_key.remove(&instance_id) {
            self.profiles.remove(&key);
            // Note: We keep the module in self.modules for get_file_names
            // Could add ref counting if needed
        }
    }

    /// Find the most specific legitimate profile for a given module path and line number, given that
    /// nested and overlapping sections are not supported. Regardless of whether they are implemented
    /// anyway, we will find the section (if any) with the lowest starting line number that encompasses
    /// the line number `line` of the allocation request. This means that in the case of nested sections
    /// we pick up only the outermost matching one and ignore the inner ones, while in the case of
    /// overlapping sections, we pick up only the one that starts first/highest in the function and
    ///
    /// Returns the profile reference if found
    #[must_use]
    #[allow(clippy::missing_panics_doc, reason = "checked start_line.is_some()")]
    pub fn find_profile(&self, module: &str, function: &str, line: u32) -> Option<ProfileRef> {
        // Search through profiles for matching module/function/line
        let mut best_match: Option<(u32, ProfileRef)> = None;

        for entry in &self.profiles {
            let key = entry.key();
            let profile_ref = entry.value();

            if key.module == module && key.function == function && key.contains_line(line) {
                let start_line = key.line_range.start;

                match &best_match {
                    None => {
                        best_match = Some((start_line, profile_ref.clone()));
                    }
                    Some((best_start, _)) if start_line < *best_start => {
                        best_match = Some((start_line, profile_ref.clone()));
                    }
                    _ => {} // Current match has higher start line, keep existing best
                }
            }
        }

        best_match.map(|(_, profile_ref)| profile_ref)
    }

    /// Get all registered file names
    ///
    /// Returns a vector containing the names of all modules that have been registered
    #[must_use]
    pub fn get_file_names(&self) -> Vec<String> {
        self.modules
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Add an allocation to a profile based on module path and line number
    /// Returns true if allocation was recorded, false otherwise
    ///
    /// # Panics
    ///
    /// Panics if it can't unwrap after get on a filename that is supposed to have been pre-checked.
    #[internal_doc]
    #[allow(clippy::branches_sharing_code)]
    pub fn record_allocation(
        &self,
        file_name: &str,
        fn_name: &str,
        line: u32,
        size: usize,
    ) -> bool {
        // Find the profile for this allocation
        let profile_ref_opt = self.find_profile(file_name, fn_name, line);

        // Process the found profile if any
        if let Some(profile_ref) = profile_ref_opt {
            // Check if we have a valid profile reference
            if let Some(profile) = profile_ref.profile() {
                // Record the allocation to the profile
                if profile_ref.detailed_memory() {
                    let start_pattern: &Regex = re!("thag_profiler::mem_tracking.+Dispatcher");
                    let end_point = profile.fn_name();
                    let mut already_seen = HashSet::new();

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
                                    if i > 0 && name.contains("record_allocation") {
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

                    /*
                    let current_backtrace = maybe_current_backtrace.unwrap();
                    // ... [existing detailed memory recording code]
                    current_backtrace.resolve();

                    let callstack: Vec<String> = Backtrace::frames(current_backtrace)
                        .iter()
                        .flat_map(BacktraceFrame::symbols)
                        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                        .skip_while(|frame| !start_pattern.is_match(frame))
                        .skip(1)
                        .take_while(|frame| !frame.contains(end_point))
                        .map(strip_hex_suffix)
                        .map(|mut name| clean_function_name(&mut name))
                        .filter(|name| {
                            if already_seen.contains(name.as_str()) {
                                false
                            } else {
                                already_seen.insert(name.clone());
                                true
                            }
                        })
                        .collect();
                     */

                    let Some(callstack) = maybe_callstack else {
                        return false;
                    };

                    let detailed_stack = profile
                        .path()
                        .iter()
                        .cloned()
                        .chain(profile.section_name())
                        .chain(callstack.iter().rev().cloned())
                        .collect();

                    write_detailed_stack_alloc(size, false, &detailed_stack);
                } else {
                    // Not detailed memory - regular allocation tracking
                    // debug_log!("Calling record_allocation on Profile for {size} bytes in {file_name}::{fn_name} at line {line}");
                    let _ = profile.record_allocation(size);
                }
                return true;
            }
            debug_log!(
                "Profile reference contains an invalid profile pointer for {file_name}::{fn_name}"
            );
        } else {
            debug_log!("No matching profile found for {file_name}::{fn_name} at line {line}");
        }

        false
    }
}

/// Reference to a Profile for the registry
/// We use a simple wrapper to avoid ownership issues
#[internal_doc]
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProfileRef {
    /// Function name or custom name if provided
    name: String,
    /// Whether this profile does detailed memory tracking
    detailed_memory: bool,
    /// Unique identifier for the Profile instance
    instance_id: u64,
    /// Reference to the Profile using Arc for thread safety
    profile: Option<Arc<Profile>>,
    /// Flag to track if this `ProfileRef` is being dropped
    /// This helps prevent recursive drops
    dropping: bool,
}

impl ProfileRef {
    /// Returns the name of the profile
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether this profile does detailed memory tracking
    #[must_use]
    pub const fn detailed_memory(&self) -> bool {
        self.detailed_memory
    }

    /// Returns the unique identifier for this profile instance
    #[must_use]
    pub const fn instance_id(&self) -> u64 {
        self.instance_id
    }

    /// Returns a reference to the contained Profile, if any
    #[must_use]
    pub fn profile(&self) -> Option<&Profile> {
        self.profile.as_ref().map(AsRef::as_ref)
    }
}

type AllocationInfo = (Vec<String>, usize);
type AddressAllocMap = DashMap<usize, AllocationInfo>;

// Global profile registry instance
static_lazy! {
    ProfileReg: ProfileRegistry = ProfileRegistry::new()
}

static_lazy! {
DetailedAddressRegistry: AddressAllocMap = DashMap::new()
}

/// Thread-safe counter for generating unique profile IDs
static NEXT_PROFILE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Get the next unique profile ID
#[must_use]
pub fn get_next_profile_id() -> u64 {
    NEXT_PROFILE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Register a profile with the global registry///
/// # Panics
///
/// Panics if the profile registration fails.
#[internal_doc]
pub fn register_profile(profile: &Profile) {
    safe_alloc! {
        // First log the information (acquires debug log mutex)
        // debug_log!("Registering profile in registry: module={}, detailed_memory={}, start_line={:?}, end_line={:?}, instance_id={}",
        //     profile.file_name(), profile.detailed_memory(), profile.start_line(), profile.end_line(), profile.instance_id());

        // Then flush to ensure the debug log mutex is released before acquiring the PROFILE_REGISTRY mutex
        flush_debug_log();

        // Create an Arc to the Profile - we'll clone it instead of trying to construct from scratch
        let profile_arc = Arc::new(profile.clone());

        // Get the profile's instance ID
        let instance_id = profile.instance_id();

        // Create a reference to this profile
        let profile_ref = ProfileRef {
            name: profile
                .section_name()
                .unwrap_or_else(|| profile.registered_name().to_string()),
            detailed_memory: profile.detailed_memory(),
            instance_id,
            // Store an Arc to the Profile
            profile: Some(profile_arc),
            dropping: false,
        };

        // let mut registry = PROFILE_REGISTRY;
        ProfileReg::get()
            .register_profile(&profile_ref)
            .expect("Error registering profile");
    };
}

/// Safely deregister a profile from the `ProfileRegistry`
///
/// This is a safer wrapper that captures all needed information before calling
/// the registry's `deregister_profile method`, to avoid any recursive drop issues.
pub fn deregister_profile(profile: &Profile) {
    // Only deregister if the profile wasn't already deregistered
    static DEREGISTERING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    // Attempt to set the deregistering flag - only proceed if we weren't already deregistering
    if DEREGISTERING
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_ok()
    {
        // First, capture all the information we need before interacting with the registry
        let instance_id = profile.instance_id();
        let file_name = safe_alloc!(profile.file_name().to_string());
        let fn_name = safe_alloc!(profile.fn_name().to_string());
        let start_line = profile.start_line();
        let end_line = profile.end_line();

        // Log the deregistration
        // debug_log!("Calling deregister_profile for instance={instance_id}, module={file_name}");
        // flush_debug_log();

        // Now deregister with the captured information
        safe_alloc! {
            // Use a scope to ensure the registry lock is released promptly
            {
                // if let Some(mut registry) = PROFILE_REGISTRY.try_lock() {
                ProfileReg::get().deregister_profile(
                    instance_id,
                    &file_name,
                    &fn_name,
                    start_line,
                    end_line,
                );
                // }
            }
            // flush_debug_log();
        };

        // Reset the flag when done
        DEREGISTERING.store(false, std::sync::atomic::Ordering::SeqCst);
    } else {
        debug_log!("Already deregistering a profile, skipping to avoid recursion");
        // flush_debug_log();
    }
}

/// Find a profile for a specific module path and line number
#[internal_doc]
#[must_use]
pub fn find_profile(file_name: &str, fn_name: &str, line: u32) -> Option<ProfileRef> {
    safe_alloc! {
        // Acquire the registry lock
        // Return the result
        ProfileReg::get().find_profile(file_name, fn_name, line)
    }
}
