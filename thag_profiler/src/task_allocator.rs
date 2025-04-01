#![allow(clippy::uninlined_format_args)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling.

use crate::{
    debug_log,
    flush_debug_log,
    // mem_alloc,
    profiling::{clean_function_name, get_memory_path},
    with_allocator,
    Allocator,
    TaskAwareAllocator,
};
use backtrace::Backtrace;
use parking_lot::Mutex;
use regex::Regex;
use std::{
    // alloc::{Layout, System},
    collections::{BTreeSet, HashMap, HashSet},
    io::{self, Write},
    sync::{atomic::AtomicUsize, LazyLock},
    thread::{self, ThreadId},
    // time::Instant,
};

pub const MINIMUM_TRACKED_SIZE: usize = 64;

/// Registry for tracking memory allocations and deallocations
#[derive(Debug)]
pub struct AllocationRegistry {
    /// Task ID -> Allocations mapping: [(address, size)]
    pub task_allocations: HashMap<usize, Vec<(usize, usize)>>,

    /// Address -> Task ID mapping for deallocations
    pub address_to_task: HashMap<usize, usize>,
}

impl AllocationRegistry {
    fn new() -> Self {
        Self {
            task_allocations: HashMap::new(),
            address_to_task: HashMap::new(),
        }
    }

    /// Get the memory usage for a specific task
    fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        self.task_allocations.get(&task_id).map(|allocations| {
            allocations
                .iter()
                .map(|(_, size)| *size)
                // .inspect(|size| debug_log!("... found alloc {size}"))
                .sum()
        })
    }
}

// Global allocation registry
pub static ALLOC_REGISTRY: LazyLock<Mutex<AllocationRegistry>> =
    LazyLock::new(|| Mutex::new(AllocationRegistry::new()));

/// Registry for tracking active profiles and task stacks
#[derive(Debug)]
struct ProfileRegistry {
    /// Set of active task IDs
    active_profiles: BTreeSet<usize>,

    /// Thread ID -> Stack of active task IDs (most recent on top)
    thread_task_stacks: HashMap<ThreadId, Vec<usize>>,
}

impl ProfileRegistry {
    fn new() -> Self {
        Self {
            active_profiles: BTreeSet::new(),
            thread_task_stacks: HashMap::new(),
        }
    }

    /// Add a task to active profiles
    fn activate_task(&mut self, task_id: usize) {
        self.active_profiles.insert(task_id);
    }

    /// Remove a task from active profiles
    fn deactivate_task(&mut self, task_id: usize) {
        self.active_profiles.remove(&task_id);
    }

    /// Get a copy of the active task IDs
    fn get_active_tasks(&self) -> Vec<usize> {
        self.active_profiles.iter().copied().collect()
    }

    /// Get the last (most recently added) active task
    fn get_last_active_task(&self) -> Option<usize> {
        self.active_profiles.iter().next_back().copied()
    }

    /// Add a task to a thread's stack
    fn push_task_to_stack(&mut self, thread_id: ThreadId, task_id: usize) {
        let stack = self.thread_task_stacks.entry(thread_id).or_default();
        stack.push(task_id);
    }

    /// Remove a task from a thread's stack
    fn pop_task_from_stack(&mut self, thread_id: ThreadId, task_id: usize) {
        if let Some(stack) = self.thread_task_stacks.get_mut(&thread_id) {
            if let Some(pos) = stack.iter().position(|id| *id == task_id) {
                stack.remove(pos);

                // Remove empty stack
                if stack.is_empty() {
                    self.thread_task_stacks.remove(&thread_id);
                }
            }
        }
    }

    // /// Get the top task for a thread
    // fn get_top_task_for_thread(&self, thread_id: ThreadId) -> Option<usize> {
    //     self.thread_task_stacks
    //         .get(&thread_id)
    //         .and_then(|stack| stack.last().copied())
    // }
}

// Global profile registry
static PROFILE_REGISTRY: LazyLock<Mutex<ProfileRegistry>> =
    LazyLock::new(|| Mutex::new(ProfileRegistry::new()));

// ---------- Public Registry API ----------

/// Add a task to active profiles
pub fn activate_task(task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().activate_task(task_id);
    });
}

/// Remove a task from active profiles
pub fn deactivate_task(task_id: usize) {
    with_allocator(Allocator::System, || {
        // Process any pending allocations before deactivating
        // process_pending_allocations();

        PROFILE_REGISTRY.lock().deactivate_task(task_id);
    });
}

/// Get the memory usage for a specific task
pub fn get_task_memory_usage(task_id: usize) -> Option<usize> {
    // Process any pending allocations first
    // process_pending_allocations();

    ALLOC_REGISTRY.lock().get_task_memory_usage(task_id)
}

/// Add a task to a thread's stack
pub fn push_task_to_stack(thread_id: ThreadId, task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY
            .lock()
            .push_task_to_stack(thread_id, task_id);
    });
}

/// Remove a task from a thread's stack
pub fn pop_task_from_stack(thread_id: ThreadId, task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY
            .lock()
            .pop_task_from_stack(thread_id, task_id);
    });
}

/// Get active tasks
pub fn get_active_tasks() -> Vec<usize> {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().get_active_tasks()
    })
}

/// Get the last active task
#[must_use]
pub fn get_last_active_task() -> Option<usize> {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().get_last_active_task()
    })
}

/// Task context for tracking allocations
#[derive(Debug, Clone)]
pub struct TaskMemoryContext {
    pub task_id: usize,
}

impl TaskMemoryContext {
    /// Gets the unique ID for this task
    pub const fn id(&self) -> usize {
        self.task_id
    }

    /// Gets current memory usage for this task
    pub fn memory_usage(&self) -> Option<usize> {
        get_task_memory_usage(self.task_id)
    }
}

// Provide a dummy TaskMemoryContext type for when full_profiling is disabled
#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskMemoryContext;

/// Creates a new task context for memory tracking.
pub fn create_memory_task() -> TaskMemoryContext {
    let allocator = get_allocator();
    allocator.create_task_context()
}

pub fn trim_backtrace(start_pattern: &Regex, current_backtrace: &Backtrace) -> Vec<String> {
    Backtrace::frames(current_backtrace)
        .iter()
        .flat_map(backtrace::BacktraceFrame::symbols)
        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
        .skip_while(|element| !start_pattern.is_match(element))
        .take_while(|name| !name.contains("__rust_begin_short_backtrace"))
        .map(|mut name| clean_function_name(&mut name))
        .collect::<Vec<String>>()
}

// Task tracking state
pub struct TaskState {
    // Counter for generating task IDs
    pub next_task_id: AtomicUsize,
}

// Global task state
pub static TASK_STATE: LazyLock<TaskState> = LazyLock::new(|| {
    // debug_log!("Initializing TASK_STATE with next_task_id = 1");
    TaskState {
        next_task_id: AtomicUsize::new(1),
    }
});

// To handle active task tracking, instead of thread-locals, we'll use task-specific techniques
#[derive(Debug)]
pub struct TaskGuard {
    task_id: usize,
}

impl TaskGuard {
    pub const fn new(task_id: usize) -> Self {
        Self { task_id }
    }
}

#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskGuard;

impl Drop for TaskGuard {
    fn drop(&mut self) {
        with_allocator(Allocator::System, || {
            // Process pending allocations before removing the task
            // process_pending_allocations();

            // Remove from active profiles
            deactivate_task(self.task_id);

            // Remove from thread stack
            pop_task_from_stack(thread::current().id(), self.task_id);

            flush_debug_log();

            // IMPORTANT: We no longer remove from task path registry
            // so that paths remain available for the memory profile output
            // remove_task_path(self.task_id);
        });
    }
}

static TASK_AWARE_ALLOCATOR: TaskAwareAllocator = TaskAwareAllocator;

// Helper to get the allocator instance
pub fn get_allocator() -> &'static TaskAwareAllocator {
    &TASK_AWARE_ALLOCATOR
}

// Task Path Registry for debugging
// 1. Declare the TASK_PATH_REGISTRY
pub static TASK_PATH_REGISTRY: LazyLock<Mutex<HashMap<usize, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// 2. Function to look up a task's path by ID
pub fn lookup_task_path(task_id: usize) -> Option<Vec<String>> {
    // debug_log!("About to try_lock TASK_PATH_REGISTRY for lookup_task_path");
    let registry = TASK_PATH_REGISTRY.lock();
    registry.get(&task_id).cloned()
}

// 3. Function to dump the entire registry
#[allow(dead_code)]
pub fn dump_task_path_registry() {
    // debug_log!("About to try_lock TASK_PATH_REGISTRY for dump_task_path_registry");
    debug_log!("==== TASK PATH REGISTRY DUMP ====");
    let task_paths = TASK_PATH_REGISTRY.lock().clone();
    debug_log!("Total registered tasks: {}", task_paths.len());

    let mut v = task_paths
        .iter()
        .map(|(&task_id, path)| (task_id, path.join("::")))
        // .cloned()
        .collect::<Vec<(usize, String)>>();

    v.sort();

    for (task_id, path) in &v {
        debug_log!("Task {}: {}", task_id, path);
    }
    drop(task_paths);
    debug_log!("=================================");
    flush_debug_log();
}

// 4. Utility function to look up and print a specific task's path
#[allow(dead_code)]
pub fn print_task_path(task_id: usize) {
    match lookup_task_path(task_id) {
        Some(path) => debug_log!("Task {} path: {}", task_id, path.join("::")),
        None => debug_log!("No path registered for task {}", task_id),
    }
    flush_debug_log();
}

// 5. Function to remove an entry from the TASK_PATH_REGISTRY
#[allow(dead_code)]
pub fn remove_task_path(task_id: usize) {
    let mut registry = TASK_PATH_REGISTRY.lock();
    registry.remove(&task_id);
}

// Helper function to find the best matching profile
pub fn find_matching_profile(path: &[String]) -> usize {
    let path_registry = TASK_PATH_REGISTRY.lock();
    // debug_log!("...success!");
    // For each active profile, compute a similarity score
    let mut best_match = 0;
    let mut best_score = 0;
    let path_len = path.len();

    #[allow(unused_assignments)]
    let mut score = 0;
    for task_id in get_active_tasks().iter().rev() {
        if let Some(reg_path) = path_registry.get(task_id) {
            score = compute_similarity(path, reg_path);
            // debug_log!(
            //     "...scored {score} checking task {} with path {:?}",
            //     task_id,
            //     reg_path.join(" -> ")
            // );
            if score > best_score || score == path_len {
                best_score = score;
                best_match = *task_id;
            }
            if score == path_len {
                break;
            }
        }
    }

    // Return the best match if found, otherwise fall back to last active task
    if best_match > 0 {
        return best_match;
    }

    // Fallback: Return the most recently activated profile
    debug_log!("...returning fallback: most recently activated profile - for path: {path:?}");
    get_last_active_task().unwrap_or(0)
}

// Compute similarity between a task path and backtrace frames
fn compute_similarity(task_path: &[String], reg_path: &[String]) -> usize {
    if task_path.is_empty() || reg_path.is_empty() {
        debug_log!("task_path.is_empty() || reg_path.is_empty()");
        return 0;
    }

    let score = task_path
        .iter()
        .zip(reg_path.iter())
        // .inspect(|(path_func, frame)| {
        //     debug_log!("Comparing [{}]\n          [{}]", path_func, frame);
        // })
        .filter(|(path_func, frame)| frame == path_func)
        // .inspect(|(path_func, frame)| {
        //     let matched = frame == path_func;
        //     debug_log!("frame == path_func? {}", matched);
        //     if matched {
        //         score += 1;
        //     }
        // })
        .count();

    // debug_log!("score={score}");
    if score == 0 {
        debug_log!("score = {score} for path of length {}", task_path.len(),);
        // let diff = create_side_by_side_diff(&task_path.join("->"), &reg_path.join("->"), 80);
        // debug_log!("{diff}");
        debug_log!("{}\n{}", task_path.join("->"), reg_path.join("->"));
    }

    score
}

/// Initialize memory profiling.
/// This is called by the main `init_profiling` function.
pub fn initialize_memory_profiling() {
    // This is called at application startup to set up memory profiling
    with_allocator(Allocator::System, || {
        debug_log!("Memory profiling initialized");
        flush_debug_log();
    });
}

/// Finalize memory profiling and write out data.
/// This is called by the main `finalize_profiling` function.
pub fn finalize_memory_profiling() {
    with_allocator(Allocator::System, || {
        write_memory_profile_data();
    });
    flush_debug_log();
}

/// Write memory profile data to a file
#[allow(clippy::too_many_lines)]
fn write_memory_profile_data() {
    use chrono::Local;
    use std::{collections::HashMap, fs::File, path::Path};

    // use crate::profiling::get_memory_path;

    with_allocator(Allocator::System, || {
        // Retrieve registries to get task allocations and names
        let memory_path = get_memory_path().unwrap_or("memory.folded");
        // debug_log!("Memory path: {memory_path}");

        // Check if the file exists first
        let file_exists = Path::new(memory_path).exists();

        // If the file already exists, write the summary information to the existing file
        // Otherwise, create a new file with the appropriate headers
        let file_result = if file_exists {
            debug_log!("Opening existing file in append mode");
            File::options().append(true).open(memory_path)
        } else {
            debug_log!("Creating new file");
            match File::create(memory_path) {
                Ok(mut file) => {
                    // Write headers similar to time profile file
                    if let Err(e) = writeln!(file, "# Memory Profile") {
                        debug_log!("Error writing header: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(
                        file,
                        "# Script: {}",
                        std::env::current_exe().unwrap_or_default().display()
                    ) {
                        debug_log!("Error writing script path: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(file, "# Version: {}", env!("CARGO_PKG_VERSION")) {
                        debug_log!("Error writing version: {e}");
                        return;
                    }

                    if let Err(e) =
                        writeln!(file, "# Date: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))
                    {
                        debug_log!("Error writing date: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(file) {
                        debug_log!("Error writing newline: {e}");
                        return;
                    }

                    Ok(file)
                }
                Err(e) => {
                    debug_log!("Error creating file: {e}");
                    Err(e)
                }
            }
        };

        if let Ok(file) = file_result {
            // debug_log!("Successfully opened file");
            let mut writer = io::BufWriter::new(file);

            // Get all task allocations
            let task_allocs = { ALLOC_REGISTRY.lock().task_allocations.clone() };
            // let task_ids = { task_allocs.keys().copied().collect::<Vec<_>>() };
            // debug_log!("Task IDs: {:?}", task_ids);

            // Get the task path registry mapping for easier lookup
            let task_paths_map: HashMap<usize, Vec<String>> = {
                let binding = TASK_PATH_REGISTRY.lock();
                // debug_log!("TASK_PATH_REGISTRY has {} entries", binding.len());

                // Dump all entries for debugging
                for (id, path) in binding.iter() {
                    debug_log!("Registry entry: task {id}: path: {:?}", path);
                }

                // Get all entries from the registry
                binding
                    .iter()
                    .map(|(task_id, pat)| (*task_id, pat.clone()))
                    .collect()
            };
            // debug_log!("Task paths map has {} entries", task_paths_map.len());

            let mut already_written = HashSet::new();

            // Now write all tasks from registry that might not have allocations
            // This helps with keeping the full call hierarchy in the output
            for (task_id, path) in &task_paths_map {
                // Skip tasks we've already written
                if task_allocs.contains_key(task_id) {
                    continue;
                }

                let path_str = path.join(";");
                if already_written.contains(&path_str) {
                    continue;
                }

                debug_log!("Writing for task {task_id} from registry: '{path_str}' with 0 bytes");

                // Write line with zero bytes to maintain call hierarchy
                match writeln!(writer, "{} {}", path_str, 0) {
                    Ok(()) => {
                        // lines_written += 1;
                        already_written.insert(path_str.clone());
                    }
                    Err(e) => debug_log!("Error writing line for task {task_id}: {e}"),
                }
            }

            // Make sure to flush the writer
            if let Err(e) = writer.flush() {
                debug_log!("Error flushing writer: {e}");
            } else {
                // debug_log!("Successfully wrote {lines_written} lines and flushed buffer");
            }
        }
    });
}
