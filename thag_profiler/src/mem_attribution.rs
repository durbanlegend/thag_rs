use crate::{
    debug_log, flush_debug_log,
    mem_tracking::{with_allocator, write_detailed_stack_alloc, Allocator},
    profiling::{clean_function_name, Profile},
    regex, strip_hex_suffix,
};
use backtrace::{Backtrace, BacktraceFrame};
use parking_lot::Mutex;
use regex::Regex;
use std::clone::Clone;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    string::ToString,
    sync::{Arc, LazyLock},
};

// Map of instance IDs to ProfileRefs
type InstanceMap = HashMap<u64, ProfileRef>;
// Mapping of line ranges to instance maps
type RangeSectionMap = BTreeMap<(Option<u32>, Option<u32>), InstanceMap>;
type FunctionRangeMap = HashMap<String, RangeSectionMap>;
type ModuleFunctionMap = HashMap<String, FunctionRangeMap>;

/// Enhanced registry for tracking profiles by module path and line number ranges
/// This allows allocations to be attributed to the correct profile based on where they occur
#[derive(Debug, Default)]
pub struct ProfileRegistry {
    /// Module path -> function name -> line ranges -> Profile instances mapping
    /// For each module path and function, maintains a map of line ranges to active Profile instances
    module_functions: ModuleFunctionMap,
    /// Set of instance IDs that are currently active
    /// This helps with quick validation without accessing the nested maps
    active_instances: HashSet<u64>,
}

/// Reference to a Profile for the registry
/// We use a simple wrapper to avoid ownership issues
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
    /// Flag to track if this ProfileRef is being dropped
    /// This helps prevent recursive drops
    dropping: bool,
}

impl ProfileRef {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn detailed_memory(&self) -> bool {
        self.detailed_memory
    }

    #[must_use]
    pub const fn instance_id(&self) -> u64 {
        self.instance_id
    }

    #[must_use]
    pub fn profile(&self) -> Option<&Profile> {
        self.profile.as_ref().map(|arc| arc.as_ref())
    }
}

impl ProfileRegistry {
    /// Register a profile with the registry
    pub fn register_profile(&mut self, profile: Arc<Profile>) {
        debug_log!("In register_profile for {profile:?}");

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
            profile: Some(Arc::clone(&profile)),
            dropping: false,
        };

        // Get the line range for this profile
        let start_line = profile.start_line();
        let end_line = profile.end_line();

        // First, ensure we have a module entry
        let file_name = profile.file_name().to_string();
        let fn_name = profile.fn_name().to_string();

        debug_log!("About to register file_name={file_name}, fn_name={fn_name}, lines={start_line:?}..{end_line:?}");

        // Get or create the function ranges map for this module
        let function_ranges = self.module_functions.entry(file_name.clone()).or_default();

        // Get or create the range sections map for this function
        let range_sections = function_ranges.entry(fn_name.clone()).or_default();

        // Get or create the instance map for this line range
        let instance_map = range_sections.entry((start_line, end_line)).or_default();

        // Insert the profile reference for this instance
        instance_map.insert(instance_id, profile_ref);

        // Add to the active instances set
        self.active_instances.insert(instance_id);

        debug_log!("Successfully registered profile in module {file_name}, function {fn_name}, lines {start_line:?}..{end_line:?}");
    }

    /// Deregister a profile when it's dropped
    pub fn deregister_profile(
        &mut self,
        instance_id: u64,
        file_name: &str,
        fn_name: &str,
        start_line: Option<u32>,
        end_line: Option<u32>,
    ) {
        debug_log!("Deregistering profile instance {instance_id} from module {file_name}, function {fn_name}, lines {start_line:?}..{end_line:?}");
        // Immediately flush logs to ensure we capture as much as possible
        flush_debug_log();

        debug_log!("Checking if instance {instance_id} exists in active_instances...");
        flush_debug_log();

        // First check if this instance is in our active instances set
        if !self.active_instances.contains(&instance_id) {
            debug_log!("Instance {instance_id} is not in active_instances, skipping deregistration to avoid recursion.");
            flush_debug_log();
            return;
        }

        // Remove from active instances set first, before any operations that might cause drops
        self.active_instances.remove(&instance_id);
        debug_log!("Removed {instance_id} from active_instances, now checking maps...");
        flush_debug_log();

        // Remove from the nested maps, with careful debug logging at each step
        if let Some(function_ranges) = self.module_functions.get_mut(file_name) {
            debug_log!("Found function_ranges for {file_name}");
            flush_debug_log();

            if let Some(range_sections) = function_ranges.get_mut(fn_name) {
                debug_log!("Found range_sections for {fn_name}");
                flush_debug_log();

                if let Some(instance_map) = range_sections.get_mut(&(start_line, end_line)) {
                    debug_log!("Found instance_map for lines {start_line:?}..{end_line:?}");
                    flush_debug_log();

                    // Check if the instance exists before removing it
                    if instance_map.contains_key(&instance_id) {
                        debug_log!("Found instance {instance_id} in map, removing...");
                        flush_debug_log();

                        // Mark the ProfileRef as dropping before we remove it
                        if let Some(profile_ref) = instance_map.get_mut(&instance_id) {
                            profile_ref.dropping = true;
                        }

                        // Take the ProfileRef out of the map before dropping it
                        let _removed = instance_map.remove(&instance_id);
                        debug_log!("Successfully removed instance {instance_id} from map.");
                        flush_debug_log();

                        // Clean up empty maps
                        if instance_map.is_empty() {
                            debug_log!("Instance map is now empty, removing range {start_line:?}..{end_line:?}");
                            flush_debug_log();
                            range_sections.remove(&(start_line, end_line));
                        }
                    } else {
                        debug_log!("Instance {instance_id} not found in map, skipping remove.");
                        flush_debug_log();
                    }
                } else {
                    debug_log!("No instance_map found for lines {start_line:?}..{end_line:?}");
                    flush_debug_log();
                }

                // Clean up empty range sections
                if range_sections.is_empty() {
                    debug_log!("Range sections is now empty, removing function {fn_name}");
                    flush_debug_log();
                    function_ranges.remove(fn_name);
                }
            } else {
                debug_log!("No range_sections found for {fn_name}");
                flush_debug_log();
            }

            // Clean up empty function ranges
            if function_ranges.is_empty() {
                debug_log!("Function ranges is now empty, removing module {file_name}");
                flush_debug_log();
                self.module_functions.remove(file_name);
            }
        } else {
            debug_log!("No function_ranges found for {file_name}");
            flush_debug_log();
        }

        debug_log!("Successfully deregistered profile instance {instance_id}");
        flush_debug_log();
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
    pub fn find_profile(&self, file_name: &str, fn_name: &str, line: u32) -> Option<ProfileRef> {
        // Check if we have this module
        let Some(function_ranges) = self.module_functions.get(file_name) else {
            debug_log!("Module not found in registry: {file_name}");
            return None;
        };

        // Check if we have this function
        let Some(range_sections) = function_ranges.get(fn_name) else {
            debug_log!("Function {fn_name} not found in module {file_name}");
            // debug_log!("self.module_functions={:#?}", self.module_functions);

            return None;
        };

        debug_log!(
            "Found range_sections for {file_name}::{fn_name} with {} entries",
            range_sections.len()
        );

        // First look for a specific line range match
        // We want a range where start_line <= line <= end_line (or end_line is None)
        for (&(start_line, end_line), instance_map) in range_sections.iter().rev() {
            if start_line.is_some()
                && start_line.unwrap() <= line
                && (end_line.is_none() || end_line.unwrap() >= line)
            {
                debug_log!(
                    "Found specific line range match {start_line:?}..{end_line:?} for line {line}"
                );

                // Find the most recently registered profile that's still active
                // This is an approximation based on the assumption that instance IDs
                // are assigned in increasing order, so higher IDs are more recent
                if let Some((&_latest_id, profile_ref)) = instance_map
                    .iter()
                    .filter(|(&id, _)| self.active_instances.contains(&id))
                    .max_by_key(|(&id, _)| id)
                {
                    return Some(profile_ref.clone());
                }
            }
        }

        // If no specific match, try to find a whole-function profile (one with no line numbers)
        if let Some(instance_map) = range_sections.get(&(None, None)) {
            // Find the most recently registered profile that's still active
            if let Some((&_latest_id, profile_ref)) = instance_map
                .iter()
                .filter(|(&id, _)| self.active_instances.contains(&id))
                .max_by_key(|(&id, _)| id)
            {
                debug_log!("Found whole-function profile for {file_name}::{fn_name}");
                return Some(profile_ref.clone());
            }
        }

        debug_log!("No profile found for {file_name}::{fn_name} at line {line}");
        None
    }

    #[must_use]
    pub fn get_file_names(&self) -> Vec<String> {
        self.module_functions.keys().cloned().collect()
    }

    /// Add an allocation to a profile based on module path and line number
    /// Returns true if allocation was recorded, false otherwise
    ///
    /// # Panics
    ///
    /// Panics if it can't unwrap after get on a filename that is supposed to have been pre-checked.
    pub fn record_allocation(
        &self,
        file_name: &str,
        fn_name: &str,
        line: u32,
        size: usize,
        address: usize,
        current_backtrace: &mut Backtrace,
    ) -> bool {
        // Check first if we even have this module and function
        if !self.module_functions.contains_key(file_name) {
            debug_log!(
                "No module found for {file_name}. Available modules: {:?}",
                self.module_functions.keys().collect::<Vec<_>>()
            );
            return false;
        }

        let function_ranges = self.module_functions.get(file_name).unwrap();
        if !function_ranges.contains_key(fn_name) {
            debug_log!(
                "No function found for {fn_name} in module {file_name}. Available functions: {:?}",
                function_ranges.keys().collect::<Vec<_>>()
            );
            return false;
        }

        // Find the profile for this allocation
        let profile_ref_opt = self.find_profile(file_name, fn_name, line);

        // Process the found profile if any
        if let Some(profile_ref) = profile_ref_opt {
            // Check if we have a valid profile reference
            if let Some(profile) = profile_ref.profile() {
                // Record the allocation to the profile
                if profile_ref.detailed_memory() {
                    // ... [existing detailed memory recording code]
                    let start_pattern: &Regex = regex!("thag_profiler::mem_tracking.+Dispatcher");
                    let end_point = profile.fn_name();
                    current_backtrace.resolve();
                    let mut already_seen = HashSet::new();

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

                    let detailed_stack: Vec<String> = profile
                        .path()
                        .iter()
                        .cloned()
                        .chain(profile.section_name())
                        .chain(callstack.iter().rev().cloned())
                        .collect();

                    write_detailed_stack_alloc(size, false, &detailed_stack);
                } else {
                    // Not detailed memory - regular allocation tracking
                    debug_log!("Calling record_allocation on Profile for {size} bytes in {file_name}::{fn_name} at line {line}");
                    let _ = profile.record_allocation(size, address);
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

/// Global profile registry instance
pub static PROFILE_REGISTRY: LazyLock<Mutex<ProfileRegistry>> =
    LazyLock::new(|| Mutex::new(ProfileRegistry::default()));

/// Thread-safe counter for generating unique profile IDs
static NEXT_PROFILE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Get the next unique profile ID
#[must_use]
pub fn get_next_profile_id() -> u64 {
    NEXT_PROFILE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Register a profile with the global registry
pub fn register_profile(profile: &Profile) {
    with_allocator(Allocator::System, || {
        // First log the information (acquires debug log mutex)
        debug_log!("Registering profile in registry: module={}, detailed_memory={}, start_line={:?}, end_line={:?}, instance_id={}",
            profile.file_name(), profile.detailed_memory(), profile.start_line(), profile.end_line(), profile.instance_id());

        // Then flush to ensure the debug log mutex is released before acquiring the PROFILE_REGISTRY mutex
        flush_debug_log();

        // Create an Arc to the Profile - we'll clone it instead of trying to construct from scratch
        let profile_arc = Arc::new(profile.clone());

        // Now acquire the PROFILE_REGISTRY mutex
        let mut registry = PROFILE_REGISTRY.lock();
        registry.register_profile(profile_arc);
    });
}

/// Safely deregister a profile from the ProfileRegistry
///
/// This is a safer wrapper that captures all needed information before calling
/// the registry's deregister_profile method, to avoid any recursive drop issues.
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
        let file_name = profile.file_name().to_string();
        let fn_name = profile.fn_name().to_string();
        let start_line = profile.start_line();
        let end_line = profile.end_line();

        // Log the deregistration
        debug_log!("Calling deregister_profile for instance={instance_id}, module={file_name}");
        flush_debug_log();

        // Now deregister with the captured information
        with_allocator(Allocator::System, || {
            // Use a scope to ensure the registry lock is released promptly
            {
                let mut registry = PROFILE_REGISTRY.lock();
                registry.deregister_profile(
                    instance_id,
                    &file_name,
                    &fn_name,
                    start_line,
                    end_line,
                );
            }
            flush_debug_log();
        });

        // Reset the flag when done
        DEREGISTERING.store(false, std::sync::atomic::Ordering::SeqCst);
    } else {
        debug_log!("Already deregistering a profile, skipping to avoid recursion");
        flush_debug_log();
    }
}

/// Find a profile for a specific module path and line number
#[must_use]
pub fn find_profile(file_name: &str, fn_name: &str, line: u32) -> Option<ProfileRef> {
    with_allocator(Allocator::System, || {
        // Acquire the registry lock
        let registry = PROFILE_REGISTRY.lock();
        // Return the result
        registry.find_profile(file_name, fn_name, line)
    })
}
