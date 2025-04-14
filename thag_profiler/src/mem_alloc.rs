use crate::{
    debug_log, flush_debug_log,
    profiling::Profile,
    task_allocator::{with_allocator, Allocator},
};
use parking_lot::Mutex;
use std::{
    collections::{BTreeMap, HashMap},
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
pub struct ProfileRef {
    /// Function name or custom name if provided
    name: String,
    /// Whether this profile does detailed memory tracking
    detailed_memory: bool,
}

impl ProfileRegistry {
    /// Register a profile with the registry
    pub fn register_profile(&mut self, profile: &Profile) {
        // Get the next ID (unused but maintained for future compatibility)
        let _id = get_next_profile_id();

        // Create a reference to this profile
        let profile_ref = ProfileRef {
            name: profile
                .custom_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| profile.registered_name().to_string()),
            detailed_memory: profile.detailed_memory(),
        };

        // Get the line range for this profile
        // If end_line is None, we use start_line for both (single line profile)
        let start_line = profile.start_line();
        let end_line = profile.end_line();

        // Insert into the module_functions map
        let function_ranges = self
            .module_functions
            .entry(profile.module_path().to_string())
            .or_default();

        let fn_name = profile.fn_name().to_string();

        // Fixed: Avoid multiple mutable borrows by directly working with the map
        let range_sections = function_ranges.entry(fn_name).or_default();
        range_sections.insert((start_line, end_line), profile_ref);

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
        // First, collect all the information we need to avoid calling debug_log while holding locks
        let function_ranges = self.module_functions.get(module_path)?;
        let range_sections = function_ranges.get(fn_name)?;

        // module_map?;

        // let module_map = module_map.unwrap();
        // let entry_count = module_map.len();

        // Find the most specific profile for this line
        // We want a range where start_line <= line <= end_line (or end_line is None)
        for (&(start_line, end_line), profile_ref) in range_sections.iter().rev() {
            if start_line.is_some()
                && start_line.unwrap() <= line
                && (end_line.is_none() || end_line.unwrap() >= line)
            {
                // Found a match
                return Some(profile_ref.clone());
            }
        }

        range_sections.get(&(None, None)).cloned()
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
    ) -> bool {
        // Find the profile for this allocation
        let profile_ref_opt = self.find_profile(module_path, fn_name, line);

        debug_log!("profile_ref_opt={profile_ref_opt:#?}");

        // Release the implicit lock before logging or doing other operations
        if let Some(profile_ref) = profile_ref_opt {
            debug_log!(
                "profile_ref={profile_ref:#?}, profile_ref.detailed_memory={}",
                profile_ref.detailed_memory
            );

            // Only record if this profile does detailed memory tracking
            if profile_ref.detailed_memory {
                // Log after all mutex operations are done
                return true;
            }
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
pub fn record_allocation(module_path: &str, fn_name: &str, line: u32, size: usize) -> bool {
    with_allocator(Allocator::System, || {
        // First log (acquires debug log mutex)
        debug_log!(
            "Looking for profile to record allocation: module={}, line={}, size={}",
            module_path,
            line,
            size
        );

        // Flush to release the debug log mutex
        flush_debug_log();

        // Now acquire the PROFILE_REGISTRY mutex
        let result;
        {
            let registry = PROFILE_REGISTRY.lock();
            result = registry.record_allocation(module_path, fn_name, line, size);
        }

        // Log after releasing the mutex
        if result {
            debug_log!(
                "Successfully recorded allocation of {size} bytes in module {module_path} at line {line}"
            );
        } else {
            debug_log!("No matching profile found to record allocation of {size} bytes in module {module_path} fn {fn_name} at line {line}");
        }
        flush_debug_log();

        result
    })
}

/// Find a profile for a specific module path and line number
pub fn find_profile(module_path: &str, fn_name: &str, line: u32) -> Option<ProfileRef> {
    with_allocator(Allocator::System, || {
        // Acquire the registry lock
        let registry = PROFILE_REGISTRY.lock();
        let result = registry.find_profile(module_path, fn_name, line);

        // Return the result
        result
    })
}
