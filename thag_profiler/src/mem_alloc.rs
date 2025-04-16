use crate::{
    debug_log, flush_debug_log,
    profiling::{clean_function_name, Profile},
    regex, strip_hex_suffix,
    task_allocator::{with_allocator, write_detailed_stack_alloc, Allocator},
};
use backtrace::{Backtrace, BacktraceFrame};
use parking_lot::Mutex;
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    string::ToString,
    sync::LazyLock,
};

type RangeSectionMap = BTreeMap<(Option<u32>, Option<u32>), ProfileRef>;
type FunctionRangeMap = HashMap<String, RangeSectionMap>;
type ModuleFunctionMap = HashMap<String, FunctionRangeMap>;

/// Enhanced registry for tracking profiles by module path and line number ranges
/// This allows allocations to be attributed to the correct profile based on where they occur
#[derive(Debug, Default)]
pub struct ProfileRegistry {
    /// Module path -> line ranges -> Profile mapping
    /// For each module path, maintains a sorted map of line ranges to profiles
    /// This allows for efficient lookup of the most specific profile for a given line number
    module_functions: ModuleFunctionMap,
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
    /// Static reference to the original Profile
    /// This is leaked to avoid ownership issues
    profile: Option<&'static Profile>,
}

impl ProfileRegistry {
    /// Register a profile with the registry
    pub fn register_profile(&mut self, profile: &Profile) {
        debug_log!("In register_profile for {profile:?}");
        // Get the next ID (unused but maintained for future compatibility)
        let _id = get_next_profile_id();

        // Create a static reference to the profile using Box::leak to avoid ownership issues
        let static_profile: &'static Profile = Box::leak(Box::new(profile.clone()));

        // Create a reference to this profile
        let profile_ref = ProfileRef {
            name: profile
                .custom_name()
                .unwrap_or_else(|| profile.registered_name().to_string()),
            detailed_memory: profile.detailed_memory(),
            profile: Some(static_profile),
        };

        // Get the line range for this profile
        // If end_line is None, we use start_line for both (single line profile)
        let start_line = profile.start_line();
        let end_line = profile.end_line();

        // First, ensure we have a module entry
        let module_path = profile.module_path().to_string();
        let fn_name = profile.fn_name().to_string();

        debug_log!("About to register module_path={module_path}, fn_name={fn_name}, lines={start_line:?}..{end_line:?}");

        // Get or create the function ranges map for this module
        let function_ranges = self
            .module_functions
            .entry(module_path.clone())
            .or_default();

        // Get or create the range sections map for this function
        let range_sections = function_ranges.entry(fn_name.clone()).or_default();

        // Insert the profile reference at this line range
        range_sections.insert((start_line, end_line), profile_ref);

        debug_log!("Successfully registered profile in module {module_path}, function {fn_name}, lines {start_line:?}..{end_line:?}");

        // Verify it was stored correctly
        if let Some(fr) = self.module_functions.get(&module_path) {
            if let Some(rs) = fr.get(&fn_name) {
                debug_log!(
                    "Verification: Found function_ranges with {} entries",
                    rs.len()
                );
            } else {
                debug_log!(
                    "Verification FAILED: function_ranges exists but has no entry for {fn_name}"
                );
            }
        } else {
            debug_log!("Verification FAILED: No entry found for module {module_path}");
        }

        debug_log!(
            "Registered profile in module {} with line range {:?}-{:?}",
            profile.module_path(),
            start_line,
            end_line
        );
    }

    /// Find the most specific profile for a given module path and line number
    /// Returns the profile reference if found
    pub fn find_profile(&self, module_path: &str, fn_name: &str, line: u32) -> Option<ProfileRef> {
        // Check if we have this module
        let Some(function_ranges) = self.module_functions.get(module_path) else {
            debug_log!("Module not found in registry: {module_path}");
            return None;
        };

        // Check if we have this function
        let Some(range_sections) = function_ranges.get(fn_name) else {
            debug_log!("Function {fn_name} not found in module {module_path}");
            return None;
        };

        debug_log!(
            "Found range_sections for {module_path}::{fn_name} with {} entries",
            range_sections.len()
        );

        // First look for a specific line range match
        // We want a range where start_line <= line <= end_line (or end_line is None)
        for (&(start_line, end_line), profile_ref) in range_sections.iter().rev() {
            if start_line.is_some()
                && start_line.unwrap() <= line
                && (end_line.is_none() || end_line.unwrap() >= line)
            {
                debug_log!(
                    "Found specific line range match {start_line:?}..{end_line:?} for line {line}"
                );
                return Some(profile_ref.clone());
            }
        }

        // If no specific match, try to find a whole-function profile (one with no line numbers)
        if let Some(profile_ref) = range_sections.get(&(None, None)) {
            debug_log!("Found whole-function profile for {module_path}::{fn_name}");
            return Some(profile_ref.clone());
        }

        debug_log!("No profile found for {module_path}::{fn_name} at line {line}");
        None
    }

    pub fn get_module_paths(&self) -> Vec<String> {
        self.module_functions.keys().cloned().collect()
    }

    /// Add an allocation to a profile based on module path and line number
    /// Returns true if allocation was recorded, false otherwise
    pub fn record_allocation(
        &self,
        module_path: &str,
        fn_name: &str,
        line: u32,
        size: usize,
        address: usize,
        current_backtrace: &mut Backtrace,
    ) -> bool {
        // Check first if we even have this module and function
        if !self.module_functions.contains_key(module_path) {
            debug_log!(
                "No module found for {module_path}. Available modules: {:?}",
                self.module_functions.keys().collect::<Vec<_>>()
            );
            return false;
        }

        let function_ranges = self.module_functions.get(module_path).unwrap();
        if !function_ranges.contains_key(fn_name) {
            debug_log!("No function found for {fn_name} in module {module_path}. Available functions: {:?}",
                    function_ranges.keys().collect::<Vec<_>>());
            return false;
        }

        // Find the profile for this allocation
        let profile_ref_opt = self.find_profile(module_path, fn_name, line);

        // debug_log!("profile_ref_opt={profile_ref_opt:#?}");

        // Process the found profile if any
        if let Some(profile_ref) = profile_ref_opt {
            // debug_log!(
            //     "profile_ref={profile_ref:#?}, profile_ref.detailed_memory={}",
            //     profile_ref.detailed_memory
            // );

            // Check if we have a profile reference
            if let Some(profile) = profile_ref.profile {
                // Record the allocation to the profile
                if profile_ref.detailed_memory {
                    // let detailed_stack =
                    //     extract_detailed_alloc_callstack(&ALLOC_START_PATTERN, current_backtrace);
                    let start_pattern: &Regex = regex!("thag_profiler::task_allocator.+Dispatcher");
                    let end_point = profile.fn_name();
                    current_backtrace.resolve();
                    let mut already_seen = HashSet::new();

                    // First, collect all relevant frames
                    let callstack: Vec<String> = Backtrace::frames(current_backtrace)
                        // let iter = Backtrace::frames(current_backtrace)
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
                        // .rev();
                        // .chain(profile)
                        .collect();

                    let detailed_stack: Vec<String> = profile
                        .path()
                        .iter()
                        .cloned()
                        .chain(profile.custom_name())
                        .chain(callstack.iter().rev().cloned())
                        .collect();

                    // TODO De-scaffold detailed_stack below this profile's entry, or cut off and append this profile's stack.
                    write_detailed_stack_alloc(size, false, &detailed_stack);
                } else {
                    // Not detailed memory
                    // Call the profile's record_allocation method directly
                    debug_log!("Calling record_allocation on Profile for {size} bytes in {module_path}::{fn_name} at line {line}");
                    let _ = profile.record_allocation(size, address);
                }
                return true;
            }
            debug_log!("Profile reference is missing the actual Profile pointer for {module_path}::{fn_name}");
        } else {
            debug_log!("No matching profile found for {module_path}::{fn_name} at line {line}");
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
fn get_next_profile_id() -> u64 {
    NEXT_PROFILE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Register a profile with the global registry
pub fn register_profile(profile: &Profile) {
    with_allocator(Allocator::System, || {
        // First log the information (acquires debug log mutex)
        debug_log!("Registering profile in registry: module={}, detailed_memory={}, start_line={:?}, end_line={:?}",
            profile.module_path(), profile.detailed_memory(), profile.start_line(), profile.end_line());

        // Then flush to ensure the debug log mutex is released before acquiring the PROFILE_REGISTRY mutex
        flush_debug_log();

        // Now acquire the PROFILE_REGISTRY mutex
        let mut registry = PROFILE_REGISTRY.lock();
        registry.register_profile(profile);
    });
}

/// Record an allocation with the global registry based on module path and line number
pub fn record_allocation(
    module_path: &str,
    fn_name: &str,
    line: u32,
    size: usize,
    address: usize,
    current_backtrace: &mut Backtrace,
) -> bool {
    with_allocator(Allocator::System, || {
        // First log (acquires debug log mutex)
        debug_log!(
            "Looking for profile to record allocation: module={module_path}, fn={fn_name}, line={line}, size={size}"
        );

        // Flush to release the debug log mutex
        flush_debug_log();

        // Print list of registered modules to help diagnose issues
        {
            let modules = PROFILE_REGISTRY.lock().get_module_paths();
            debug_log!("Available modules in registry: {modules:?}");
            flush_debug_log();
        }

        // Now acquire the PROFILE_REGISTRY mutex
        let result;
        {
            debug_log!("About to call record_allocation on registry");
            result = PROFILE_REGISTRY.lock().record_allocation(
                module_path,
                fn_name,
                line,
                size,
                address,
                current_backtrace,
            );
            debug_log!("record_allocation on registry returned {result}");
        }

        // Log after releasing the mutex
        if result {
            debug_log!(
                "Successfully recorded allocation of {size} bytes in module {module_path}::{fn_name} at line {line}"
            );
        } else {
            debug_log!("No matching profile found to record allocation of {size} bytes in module {module_path}::{fn_name} at line {line}");
        }
        flush_debug_log();

        result
    })
}

/// Find a profile for a specific module path and line number
#[must_use]
pub fn find_profile(module_path: &str, fn_name: &str, line: u32) -> Option<ProfileRef> {
    with_allocator(Allocator::System, || {
        // Acquire the registry lock
        let registry = PROFILE_REGISTRY.lock();
        // Return the result
        registry.find_profile(module_path, fn_name, line)
    })
}
