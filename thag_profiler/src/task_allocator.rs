#![allow(clippy::uninlined_format_args)]
#![deny(unsafe_op_in_unsafe_fn)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling. It also contains
//! the custom memory allocator implementation that enables memory profiling.

use crate::{
    debug_log, extract_path, flush_debug_log, lazy_static_var,
    profiling::{
        clean_function_name, extract_alloc_callstack, extract_detailed_alloc_callstack,
        get_memory_detail_path, get_memory_path, is_detailed_memory, is_profiling_state_enabled,
        MemoryDetailFile,
    },
    regex, Profile,
};
use backtrace::Backtrace;
use parking_lot::Mutex;
use regex::Regex;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::{BTreeSet, HashMap, HashSet},
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock,
    },
    thread::{self, ThreadId},
    time::Instant,
};

// ========== MEMORY ALLOCATOR DEFINITION ==========

/// Enum defining all available allocators used by the profiler
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Allocator {
    /// Default allocator that performs memory tracking
    TaskAware,
    /// System allocator used internally to prevent recursion
    System,
}

// We'll use a simple atomic instead of TLS to avoid destructors
static CURRENT_ALLOCATOR_ID: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

// Function to get current allocator type
pub fn current_allocator() -> Allocator {
    let allocator_id = CURRENT_ALLOCATOR_ID.load(std::sync::atomic::Ordering::Relaxed);
    if allocator_id == 0 {
        Allocator::TaskAware
    } else {
        Allocator::System
    }
}

// Function to run code with a specific allocator
pub fn with_allocator<T, F: FnOnce() -> T>(req_alloc: Allocator, f: F) -> T {
    // Convert allocator enum to ID
    let allocator_id = match req_alloc {
        Allocator::TaskAware => 0,
        Allocator::System => 1,
    };

    // Save the current allocator
    let prev_id = CURRENT_ALLOCATOR_ID.load(std::sync::atomic::Ordering::Relaxed);

    // Set the new allocator
    CURRENT_ALLOCATOR_ID.store(allocator_id, std::sync::atomic::Ordering::Relaxed);

    // Run the function
    let result = f();

    // Restore the previous allocator
    CURRENT_ALLOCATOR_ID.store(prev_id, std::sync::atomic::Ordering::Relaxed);

    result
}

/// Task-aware allocator that tracks memory allocations
pub struct TaskAwareAllocator;

// Static instance for global access
static TASK_AWARE_ALLOCATOR: TaskAwareAllocator = TaskAwareAllocator;

// Helper to get the allocator instance
pub fn get_allocator() -> &'static TaskAwareAllocator {
    &TASK_AWARE_ALLOCATOR
}

#[allow(clippy::unused_self)]
impl TaskAwareAllocator {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize in profile registry
        activate_task(task_id);

        TaskMemoryContext { task_id }
    }
}

unsafe impl GlobalAlloc for TaskAwareAllocator {
    #[allow(clippy::too_many_lines)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };

        if !ptr.is_null() && is_profiling_state_enabled() {
            with_allocator(Allocator::System, || {
                // Skip small allocations
                let size = layout.size();
                if size > MINIMUM_TRACKED_SIZE {
                    // Simple recursion prevention without using TLS with destructors
                    static mut IN_TRACKING: bool = false;
                    struct Guard;
                    impl Drop for Guard {
                        fn drop(&mut self) {
                            unsafe {
                                IN_TRACKING = false;
                            }
                        }
                    }

                    // Flag if we're already tracking in case it causes an infinite recursion
                    if unsafe { IN_TRACKING } {
                        debug_log!(
                            "*** Caution: already tracking: proceeding for allocation of {size} B"
                        );
                        // return ptr;
                    }

                    // Set tracking flag and create guard for cleanup
                    unsafe {
                        IN_TRACKING = true;
                    }
                    let _guard = Guard;

                    // Get backtrace without recursion
                    // debug_log!("Attempting backtrace");
                    // Use a different allocator for backtrace operations
                    let start_ident = Instant::now();
                    let mut task_id = 0;
                    // Now we can safely use backtrace without recursion!
                    let start_pattern: &Regex = regex!("thag_profiler::task_allocator.+Dispatcher");

                    // debug_log!("Calling extract_callstack");
                    let mut current_backtrace = Backtrace::new_unresolved();
                    let cleaned_stack =
                        extract_alloc_callstack(start_pattern, &mut current_backtrace);
                    debug_log!("Cleaned_stack for size={size}: {cleaned_stack:?}");
                    let in_profile_code = cleaned_stack.iter().any(|frame| {
                        frame.contains("Backtrace::new") || frame.contains("Profile::new")
                    });

                    if in_profile_code {
                        debug_log!("Ignoring allocation request of size {size} for profiler code");
                        return;
                    }

                    let detailed_memory = lazy_static_var!(bool, deref, is_detailed_memory());
                    if size > 0 && detailed_memory {
                        let detailed_stack =
                            extract_detailed_alloc_callstack(start_pattern, &mut current_backtrace);

                        let entry = if detailed_stack.is_empty() {
                            format!("[out_of_bounds] +{size}")
                        } else {
                            format!("{} +{size}", detailed_stack.join(";"))
                        };

                        let memory_detail_path = get_memory_detail_path().unwrap();
                        let _ = Profile::write_profile_event(
                            memory_detail_path,
                            MemoryDetailFile::get(),
                            &entry,
                        );
                    }

                    current_backtrace.resolve();

                    if cleaned_stack.is_empty() {
                        debug_log!(
                            "...empty cleaned_stack for backtrace: size={size}:\n{:#?}",
                            trim_backtrace(start_pattern, &current_backtrace)
                        );
                        debug_log!("Getting last active task (hmmm :/)");
                        task_id = get_last_active_task().unwrap_or(0);
                    } else {
                        // Make sure the use of a separate allocator is working.
                        assert!(!cleaned_stack
                            .iter()
                            .any(|frame| frame.contains("find_matching_profile")));

                        debug_log!("Calling extract_path");
                        let path = extract_path(&cleaned_stack, None);
                        if path.is_empty() {
                            let trimmed_backtrace =
                                trim_backtrace(start_pattern, &current_backtrace);
                            if trimmed_backtrace
                                .iter()
                                .any(|frame| frame.contains("Backtrace::new"))
                            {
                                debug_log!("Ignoring setup allocation of size {size} containing Backtrace::new:");
                                return;
                            }
                            debug_log!(
                                "...path is empty for thread {:?}, size: {size:?}, not eligible for allocation",
                                thread::current().id(),
                            );
                        } else {
                            task_id = find_matching_profile(&path);
                            debug_log!(
                                "...find_matching_profile found task_id={task_id} for size={size}"
                            );
                        }
                    }
                    debug_log!(
                        "task_id={task_id}, size={size}, time to assign = {}ms",
                        start_ident.elapsed().as_millis()
                    );

                    // Record allocation if task found
                    if task_id == 0 {
                        // TODO record in suspense file and allocate later
                        return;
                    }

                    let start_record_alloc = Instant::now();
                    // Use system allocator to avoid recursive allocations
                    let address = ptr as usize;

                    debug_log!("Recording allocation for task_id={task_id}, address={address:#x}, size={size}");
                    let mut registry = ALLOC_REGISTRY.lock();
                    registry
                        .task_allocations
                        .entry(task_id)
                        .or_default()
                        .push((address, size));

                    registry.address_to_task.insert(address, task_id);
                    let check_map = &registry.task_allocations;
                    let reg_task_id = *registry.address_to_task.get(&address).unwrap();
                    let maybe_vec = check_map.get(&task_id);
                    let (addr, sz) = *maybe_vec
                        .and_then(|v: &Vec<(usize, usize)>| {
                            let last = v.iter().filter(|&(addr, _)| *addr == address).last();
                            last
                        })
                        .unwrap();
                    drop(registry);
                    assert_eq!(sz, size);
                    assert_eq!(addr, address);
                    assert_eq!(reg_task_id, task_id);
                    debug_log!(
                        "Time to record allocation: {}ms",
                        start_record_alloc.elapsed().as_millis()
                    );
                }
            });
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Just forward to system allocator for deallocation
        unsafe { System.dealloc(ptr, layout) };
    }
}

/// Dispatcher allocator that routes allocation requests to the appropriate allocator
pub struct Dispatcher {
    task_aware: TaskAwareAllocator,
    system: System,
}

impl Dispatcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            task_aware: TaskAwareAllocator,
            system: System,
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for Dispatcher {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get current allocator type from thread-local storage
        match current_allocator() {
            Allocator::TaskAware => unsafe { self.task_aware.alloc(layout) },
            Allocator::System => unsafe { self.system.alloc(layout) },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // For deallocation, we don't need to check the allocator type, as both
        // allocators use System for the actual deallocation.
        unsafe { self.system.dealloc(ptr, layout) };
    }
}

// Create a direct static instance
#[global_allocator]
static ALLOCATOR: Dispatcher = Dispatcher::new();

// ========== ALLOCATION TRACKING DEFINITIONS ==========

pub const MINIMUM_TRACKED_SIZE: usize = 0;

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
        self.task_allocations
            .get(&task_id)
            .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
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
}

// Global profile registry
static PROFILE_REGISTRY: LazyLock<Mutex<ProfileRegistry>> =
    LazyLock::new(|| Mutex::new(ProfileRegistry::new()));

// ========== PUBLIC REGISTRY API ==========

/// Add a task to active profiles
pub fn activate_task(task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().activate_task(task_id);
    });
}

/// Remove a task from active profiles
#[allow(dead_code)]
pub fn deactivate_task(task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().deactivate_task(task_id);
    });
}

/// Get the memory usage for a specific task
pub fn get_task_memory_usage(task_id: usize) -> Option<usize> {
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
#[allow(dead_code)]
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

// ========== TASK CONTEXT DEFINITIONS ==========

/// Task context for tracking allocations
#[derive(Debug, Clone)]
pub struct TaskMemoryContext {
    pub task_id: usize,
}

impl TaskMemoryContext {
    /// Gets the unique ID for this task
    #[must_use]
    pub const fn id(&self) -> usize {
        self.task_id
    }

    /// Gets current memory usage for this task
    #[must_use]
    pub fn memory_usage(&self) -> Option<usize> {
        get_task_memory_usage(self.task_id)
    }

    /// Enter this task context for memory tracking
    ///
    /// # Errors
    ///
    /// This function will bubble up any errors encountered (TODO: do we need a Result wrapper?)
    pub fn enter(&self) -> crate::ProfileResult<TaskGuard> {
        // Push to thread stack
        push_task_to_stack(thread::current().id(), self.task_id);
        Ok(TaskGuard::new(self.task_id))
    }
}

// Provide a dummy TaskMemoryContext type for when full_profiling is disabled
#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskMemoryContext;

/// Creates a new task context for memory tracking.
#[must_use]
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

// ========== TASK STATE MANAGEMENT ==========

// Task tracking state
pub struct TaskState {
    // Counter for generating task IDs
    pub next_task_id: AtomicUsize,
}

// Global task state
pub static TASK_STATE: LazyLock<TaskState> = LazyLock::new(|| TaskState {
    next_task_id: AtomicUsize::new(1),
});

// To handle active task tracking, instead of thread-locals, we'll use task-specific techniques
#[derive(Debug)]
pub struct TaskGuard {
    task_id: usize,
}

impl TaskGuard {
    #[must_use]
    pub const fn new(task_id: usize) -> Self {
        Self { task_id }
    }
}

#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskGuard;

impl Drop for TaskGuard {
    fn drop(&mut self) {
        // Use simpler approach with direct method calls
        // This avoids complex TLS interactions during thread shutdown

        // Run these operations with System allocator
        with_allocator(Allocator::System, || {
            // Remove from active profiles
            PROFILE_REGISTRY.lock().deactivate_task(self.task_id);

            // Remove from thread stack
            PROFILE_REGISTRY
                .lock()
                .pop_task_from_stack(thread::current().id(), self.task_id);

            // Flush logs directly
            if let Some(logger) = crate::DebugLogger::get() {
                let _ = logger.lock().flush();
            }
        });
    }
}

// ========== TASK PATH MANAGEMENT ==========

// Task Path Registry for debugging
// 1. Declare the TASK_PATH_REGISTRY
pub static TASK_PATH_REGISTRY: LazyLock<Mutex<HashMap<usize, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// 2. Function to look up a task's path by ID
pub fn lookup_task_path(task_id: usize) -> Option<Vec<String>> {
    let registry = TASK_PATH_REGISTRY.lock();
    registry.get(&task_id).cloned()
}

// 3. Function to dump the entire registry
#[allow(dead_code)]
pub fn dump_task_path_registry() {
    debug_log!("==== TASK PATH REGISTRY DUMP ====");
    let task_paths = TASK_PATH_REGISTRY.lock().clone();
    debug_log!("Total registered tasks: {}", task_paths.len());

    let mut v = task_paths
        .iter()
        .map(|(&task_id, path)| (task_id, path.join("::")))
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
    // For each active profile, compute a similarity score
    let mut best_match = 0;
    let mut best_score = 0;
    let path_len = path.len();

    #[allow(unused_assignments)]
    let mut score = 0;
    for task_id in get_active_tasks().iter().rev() {
        if let Some(reg_path) = path_registry.get(task_id) {
            score = compute_similarity(path, reg_path);
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
        .filter(|(path_func, frame)| frame == path_func)
        .count();

    if score == 0 {
        debug_log!("score = {score} for path of length {}", task_path.len());
        debug_log!("{}\n{}", task_path.join("->"), reg_path.join("->"));
    }

    score
}

// ========== MEMORY PROFILING LIFECYCLE ==========

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
    write_memory_profile_data();
    flush_debug_log();
}

/// Write memory profile data to a file
#[allow(clippy::too_many_lines)]
fn write_memory_profile_data() {
    use chrono::Local;
    use std::{collections::HashMap, fs::File, path::Path};

    with_allocator(Allocator::System, || {
        // Retrieve registries to get task allocations and names
        let memory_path = get_memory_path().unwrap_or("memory.folded");

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
            let mut writer = io::BufWriter::new(file);

            // Get all task allocations
            let task_allocs = { ALLOC_REGISTRY.lock().task_allocations.clone() };

            // Get the task path registry mapping for easier lookup
            let task_paths_map: HashMap<usize, Vec<String>> = {
                let binding = TASK_PATH_REGISTRY.lock();

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
                        already_written.insert(path_str.clone());
                    }
                    Err(e) => debug_log!("Error writing line for task {task_id}: {e}"),
                }
            }

            // Make sure to flush the writer
            if let Err(e) = writer.flush() {
                debug_log!("Error flushing writer: {e}");
            }
        }
    });
}
