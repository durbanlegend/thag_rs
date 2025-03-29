#![allow(clippy::uninlined_format_args)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling.

use std::alloc::{GlobalAlloc, Layout};
use std::time::Instant;

use crate::profiling::clean_function_name;

use crate::set_multi_global_allocator;

use crate::{okaoka, regex};

use crate::profiling::{extract_callstack_from_alloc_backtrace, extract_path, get_memory_path};

use backtrace::Backtrace;
use parking_lot::Mutex;
use regex::Regex;
use std::{
    alloc::System,
    // cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock,
    },
    thread::{self, ThreadId},
};

const MINIMUM_TRACKED_SIZE: usize = 64;

/// Registry for tracking memory allocations and deallocations
#[derive(Debug)]
struct AllocationRegistry {
    /// Task ID -> Allocations mapping: [(address, size)]
    task_allocations: HashMap<usize, Vec<(usize, usize)>>,

    /// Address -> Task ID mapping for deallocations
    address_to_task: HashMap<usize, usize>,
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
                // .inspect(|size| eprintln!("... found alloc {size}"))
                .sum()
        })
    }
}

// Global allocation registry
static ALLOC_REGISTRY: LazyLock<Mutex<AllocationRegistry>> =
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
    MultiAllocator::with(AllocatorTag::System, || {
        PROFILE_REGISTRY.lock().activate_task(task_id);
    });
}

/// Remove a task from active profiles
pub fn deactivate_task(task_id: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
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
    MultiAllocator::with(AllocatorTag::System, || {
        PROFILE_REGISTRY
            .lock()
            .push_task_to_stack(thread_id, task_id);
    });
}

/// Remove a task from a thread's stack
pub fn pop_task_from_stack(thread_id: ThreadId, task_id: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
        PROFILE_REGISTRY
            .lock()
            .pop_task_from_stack(thread_id, task_id);
    });
}

/// Get active tasks
pub fn get_active_tasks() -> Vec<usize> {
    let mut active_tasks = Box::new(vec![]);
    MultiAllocator::with(AllocatorTag::System, || {
        let active = PROFILE_REGISTRY.lock().get_active_tasks();
        active_tasks = Box::new(active);
    });
    *active_tasks
}

/// Get the last active task
pub fn get_last_active_task() -> Option<usize> {
    let mut last_active_task: Box<Option<usize>> = Box::new(None);
    MultiAllocator::with(AllocatorTag::System, || {
        let last_active = PROFILE_REGISTRY.lock().get_last_active_task();
        last_active_task = Box::new(last_active);
    });
    *last_active_task
}

/// Task-aware allocator that tracks memory usage per task ID
#[derive(Debug)]
pub struct TaskAwareAllocator<A: GlobalAlloc> {
    /// The inner allocator that actually performs allocation
    inner: A,
}

/// Task context for tracking allocations
#[derive(Debug, Clone)]
pub struct TaskMemoryContext {
    pub task_id: usize,
    // allocator: &'static TaskAwareAllocator<System>,
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

// Define registry-specific methods for System allocator
#[allow(clippy::unused_self)]
impl TaskAwareAllocator<System> {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize in profile registry
        activate_task(task_id);

        TaskMemoryContext { task_id }
    }
}

/// Creates a new task context for memory tracking.
pub fn create_memory_task() -> TaskMemoryContext {
    let allocator = get_allocator();
    allocator.create_task_context()
}

pub fn run_mut_with_system_alloc(closure: impl FnMut()) {
    MultiAllocator::with(AllocatorTag::System, closure);
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
    #[allow(clippy::too_many_lines)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        if !ptr.is_null() {
            // Skip small allocations
            let size = layout.size();
            if size >= MINIMUM_TRACKED_SIZE {
                // Simple recursion prevention
                thread_local! {
                    static IN_TRACKING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
                }

                // Create guard for cleanup
                struct Guard;
                impl Drop for Guard {
                    fn drop(&mut self) {
                        IN_TRACKING.with(|flag| flag.set(false));
                    }
                }
                let _guard = Guard;

                // Get backtrace without recursion
                // eprintln!("Attempting backtrace");
                // Use a different allocator for backtrace operations
                let start_ident = Instant::now();
                let mut task_id = 0;
                MultiAllocator::with(AllocatorTag::System, || {
                    // Now we can safely use backtrace without recursion!
                    let start_pattern: &Regex = regex!("thag_profiler::okaoka.+MultiAllocator");

                    // eprintln!("Calling extract_callstack");
                    let mut current_backtrace = Backtrace::new_unresolved();
                    let cleaned_stack = extract_callstack_from_alloc_backtrace(
                        start_pattern,
                        &mut current_backtrace,
                    );
                    eprintln!("cleaned_stack for size={size}: {cleaned_stack:?}");
                    let in_profile_code = cleaned_stack.iter().any(|frame| {
                        frame.contains("Backtrace::new") || frame.contains("Profile::new")
                    });

                    if in_profile_code {
                        eprintln!("Ignoring allocation request of size {size} for profiler code");
                        return;
                    }

                    current_backtrace.resolve();

                    if cleaned_stack.is_empty() {
                        eprintln!(
                            "Empty cleaned_stack for backtrace: size={size}:\n{:#?}",
                            trim_backtrace(start_pattern, &current_backtrace)
                        );
                        eprintln!("Getting last active task (hmmm :/)");
                        task_id = get_last_active_task().unwrap_or(0);
                    } else {
                        // Make sure the use of a separate allocator is working.
                        assert!(!cleaned_stack
                            .iter()
                            .any(|frame| frame.contains("find_matching_profile")));

                        eprintln!("Calling extract_path");
                        let path = extract_path(&cleaned_stack);
                        if path.is_empty() {
                            let trimmed_backtrace =
                                trim_backtrace(start_pattern, &current_backtrace);
                            if trimmed_backtrace
                                .iter()
                                .any(|frame| frame.contains("Backtrace::new"))
                            {
                                eprintln!("Ignoring setup allocation of size {size} containing Backtrace::new:");
                                // Don't record the allocation because it's profiling setup
                                // Backtrace::frames(&current_backtrace)
                                //     .iter()
                                //     .flat_map(backtrace::BacktraceFrame::symbols)
                                //     .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                                //     .for_each(|frame| {
                                //         eprintln!("frame: {}", frame);
                                //     });
                                return;
                            }
                            // eprintln!(
                            //     "...path is empty for thread {:?}: assigning to lastest active task.\nCleaned_stack: {:?}",
                            //     thread::current().id(),
                            //     cleaned_stack
                            // );
                            eprintln!(
                                "...path is empty for thread {:?}, size: {size:?}, not eligible for allocation",
                                thread::current().id(),
                            );
                            // eprintln!("...backtrace:\n{trimmed_backtrace:?}");
                            // let last_active_task = get_last_active_task();
                            // task_id = last_active_task.unwrap_or(0);
                            // if task_id == 0 {
                            //     eprintln!(
                            //         "...no active task found, calling get_active_tasks to confirm"
                            //     );
                            //     eprintln!("...active tasks: {:?}", get_active_tasks());
                            // }
                        } else {
                            // eprintln!("path={path:#?}");

                            task_id = find_matching_profile(&path);
                            eprintln!(
                                "...find_matching_profile found task_id={task_id} for size={size}"
                            );
                        }
                    }
                });
                run_with_system_alloc(|| {
                    println!(
                        "task_id={task_id}, size={size}, time to assign = {}ms",
                        start_ident.elapsed().as_millis()
                    );
                });

                // Record allocation if task found
                if task_id == 0 {
                    return ptr;
                }

                let start_record_alloc = Instant::now();
                // Use okaoka to avoid recursive allocations
                MultiAllocator::with(AllocatorTag::System, || {
                    let address = ptr as usize;
                    // let size = layout.size();

                    // Record in thread-local buffer
                    // record_allocation(task_id, address, size);
                    eprintln!("Recording allocation for task_id={task_id}, address={address:#x}, size={size}");
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
                    // eprintln!("Check registry.task_allocations for task_id {task_id}: ({addr:#x}, {sz})");
                    assert_eq!(sz, size);
                    assert_eq!(addr, address);
                    // eprintln! (
                    //     "Check registry.address_to_task for task_id {task_id}: {:?}",
                    //     registry.address_to_task.get(&address)
                    // );
                    assert_eq!(reg_task_id, task_id);
                    // eprintln!(
                    //     "task {task_id} memory usage: {:?}",
                    //     ALLOC_REGISTRY.lock().get_task_memory_usage(task_id)
                    // );
                    eprintln!(
                        "Time to record allocation: {}ms",
                        start_record_alloc.elapsed().as_millis()
                    );
                });
            } else {
                // eprintln!("ignoring allocation of {} bytes", layout.size());
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Don't record deallocation
        self.inner.dealloc(ptr, layout);
    }
}

fn trim_backtrace(start_pattern: &Regex, current_backtrace: &Backtrace) -> Vec<String> {
    let x = Backtrace::frames(current_backtrace)
        .iter()
        .flat_map(backtrace::BacktraceFrame::symbols)
        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
        .skip_while(|element| !start_pattern.is_match(element))
        .take_while(|name| !name.contains("__rust_begin_short_backtrace"))
        .map(|mut name| clean_function_name(&mut name))
        .collect::<Vec<String>>();
    x
}

// Task tracking state
struct TaskState {
    // Counter for generating task IDs
    next_task_id: AtomicUsize,
}

// Global task state
static TASK_STATE: LazyLock<TaskState> = LazyLock::new(|| {
    // println!("Initializing TASK_STATE with next_task_id = 1");
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
        MultiAllocator::with(AllocatorTag::System, || {
            // Process pending allocations before removing the task
            // process_pending_allocations();

            // Remove from active profiles
            deactivate_task(self.task_id);

            // Remove from thread stack
            pop_task_from_stack(thread::current().id(), self.task_id);

            // IMPORTANT: We no longer remove from task path registry
            // so that paths remain available for the memory profile output
            // remove_task_path(self.task_id);
        });
    }
}

pub fn run_with_system_alloc(closure: impl Fn()) {
    MultiAllocator::with(AllocatorTag::System, closure);
}

static TASK_AWARE_ALLOCATOR: TaskAwareAllocator<System> = TaskAwareAllocator { inner: System };

// Helper to get the allocator instance
pub fn get_allocator() -> &'static TaskAwareAllocator<System> {
    &TASK_AWARE_ALLOCATOR
}

// Task Path Registry for debugging
// 1. Declare the TASK_PATH_REGISTRY
pub static TASK_PATH_REGISTRY: LazyLock<Mutex<HashMap<usize, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// 2. Function to look up a task's path by ID
pub fn lookup_task_path(task_id: usize) -> Option<Vec<String>> {
    // eprintln!("About to try_lock TASK_PATH_REGISTRY for lookup_task_path");
    let registry = TASK_PATH_REGISTRY.lock();
    registry.get(&task_id).cloned()
}

// 3. Function to dump the entire registry
#[allow(dead_code)]
pub fn dump_task_path_registry() {
    // eprintln!("About to try_lock TASK_PATH_REGISTRY for dump_task_path_registry");
    println!("==== TASK PATH REGISTRY DUMP ====");
    let task_paths = TASK_PATH_REGISTRY.lock().clone();
    println!("Total registered tasks: {}", task_paths.len());

    let mut v = task_paths
        .iter()
        .map(|(&task_id, path)| (task_id, path.join("::")))
        // .cloned()
        .collect::<Vec<(usize, String)>>();

    v.sort();

    for (task_id, path) in &v {
        println!("Task {}: {}", task_id, path);
    }
    drop(task_paths);
    println!("=================================");
}

// 4. Utility function to look up and print a specific task's path
#[allow(dead_code)]
pub fn print_task_path(task_id: usize) {
    match lookup_task_path(task_id) {
        Some(path) => println!("Task {} path: {}", task_id, path.join("::")),
        None => println!("No path registered for task {}", task_id),
    }
}

// 5. Function to remove an entry from the TASK_PATH_REGISTRY
#[allow(dead_code)]
pub fn remove_task_path(task_id: usize) {
    let mut registry = TASK_PATH_REGISTRY.lock();
    registry.remove(&task_id);
}

// Helper function to find the best matching profile
fn find_matching_profile(path: &[String]) -> usize {
    let path_registry = TASK_PATH_REGISTRY.lock();
    // eprintln!("...success!");
    // For each active profile, compute a similarity score
    let mut best_match = 0;
    let mut best_score = 0;
    let path_len = path.len();

    #[allow(unused_assignments)]
    let mut score = 0;
    for task_id in get_active_tasks().iter().rev() {
        if let Some(reg_path) = path_registry.get(task_id) {
            score = compute_similarity(path, reg_path);
            // eprintln!(
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
    eprintln!("...returning fallback: most recently activated profile - for path: {path:?}");
    get_last_active_task().unwrap_or(0)
}

// Compute similarity between a task path and backtrace frames
fn compute_similarity(task_path: &[String], reg_path: &[String]) -> usize {
    if task_path.is_empty() || reg_path.is_empty() {
        eprintln!("task_path.is_empty() || reg_path.is_empty()");
        return 0;
    }

    let score = task_path
        .iter()
        .zip(reg_path.iter())
        // .inspect(|(path_func, frame)| {
        //     eprintln!("Comparing [{}]\n          [{}]", path_func, frame);
        // })
        .filter(|(path_func, frame)| frame == path_func)
        // .inspect(|(path_func, frame)| {
        //     let matched = frame == path_func;
        //     eprintln!("frame == path_func? {}", matched);
        //     if matched {
        //         score += 1;
        //     }
        // })
        .count();

    // eprintln!("score={score}");
    if score == 0 {
        eprintln!("score = {score} for path of length {}", task_path.len(),);
        // let diff = create_side_by_side_diff(&task_path.join("->"), &reg_path.join("->"), 80);
        // println!("{diff}");
        println!("{}\n{}", task_path.join("->"), reg_path.join("->"));
    }

    score
}

// Setup for okaoka
set_multi_global_allocator! {
    MultiAllocator, // Name of our allocator facade
    AllocatorTag,   // Name of our allocator tag enum
    _Default => TaskAwareAllocatorWrapper,  // Our profiling allocator, first = default
    System => System,          // Standard system allocator for backtraces
}

// Wrapper to expose our TaskAwareAllocator to okaoka
struct TaskAwareAllocatorWrapper;

unsafe impl std::alloc::GlobalAlloc for TaskAwareAllocatorWrapper {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        // Use the static allocator instance
        TASK_AWARE_ALLOCATOR.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        TASK_AWARE_ALLOCATOR.dealloc(ptr, layout);
    }
}

/// Initialize memory profiling.
/// This is called by the main `init_profiling` function.
pub fn initialize_memory_profiling() {
    // This is called at application startup to set up memory profiling
    MultiAllocator::with(AllocatorTag::System, || {
        println!("Memory profiling initialized");
    });
}

/// Finalize memory profiling and write out data.
/// This is called by the main `finalize_profiling` function.
pub fn finalize_memory_profiling() {
    MultiAllocator::with(AllocatorTag::System, || {
        write_memory_profile_data();
    });
}

/// Write memory profile data to a file
#[allow(clippy::too_many_lines)]
fn write_memory_profile_data() {
    use chrono::Local;
    use std::{collections::HashMap, fs::File, path::Path};

    // use crate::profiling::get_memory_path;

    MultiAllocator::with(AllocatorTag::System, || {
        // println!("Starting write_memory_profile_data...");

        // println!("Profiled functions:\n{:#?}", dump_profiled_functions());

        // Retrieve registries to get task allocations and names
        let memory_path = get_memory_path().unwrap_or("memory.folded");
        // println!("Memory path: {memory_path}");

        // Check if the file exists first
        let file_exists = Path::new(memory_path).exists();

        // If the file already exists, write the summary information to the existing file
        // Otherwise, create a new file with the appropriate headers
        let file_result = if file_exists {
            println!("Opening existing file in append mode");
            File::options().append(true).open(memory_path)
        } else {
            println!("Creating new file");
            match File::create(memory_path) {
                Ok(mut file) => {
                    // Write headers similar to time profile file
                    if let Err(e) = writeln!(file, "# Memory Profile") {
                        println!("Error writing header: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(
                        file,
                        "# Script: {}",
                        std::env::current_exe().unwrap_or_default().display()
                    ) {
                        println!("Error writing script path: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(file, "# Version: {}", env!("CARGO_PKG_VERSION")) {
                        println!("Error writing version: {e}");
                        return;
                    }

                    if let Err(e) =
                        writeln!(file, "# Date: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))
                    {
                        println!("Error writing date: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(file) {
                        println!("Error writing newline: {e}");
                        return;
                    }

                    Ok(file)
                }
                Err(e) => {
                    println!("Error creating file: {e}");
                    Err(e)
                }
            }
        };

        if let Ok(file) = file_result {
            // println!("Successfully opened file");
            let mut writer = io::BufWriter::new(file);

            // Get all task allocations
            let task_allocs = { ALLOC_REGISTRY.lock().task_allocations.clone() };
            // let task_ids = { task_allocs.keys().copied().collect::<Vec<_>>() };
            // println!("Task IDs: {:?}", task_ids);

            // Get the task path registry mapping for easier lookup
            let task_paths_map: HashMap<usize, Vec<String>> = {
                let binding = TASK_PATH_REGISTRY.lock();
                // println!("TASK_PATH_REGISTRY has {} entries", binding.len());

                // Dump all entries for debugging
                for (id, path) in binding.iter() {
                    println!("Registry entry: task {id}: path: {:?}", path);
                }

                // Get all entries from the registry
                binding
                    .iter()
                    .map(|(task_id, pat)| (*task_id, pat.clone()))
                    .collect()
            };
            // println!("Task paths map has {} entries", task_paths_map.len());

            // Write profile data
            // let mut lines_written = 0;

            let mut already_written = HashSet::new();

            // First write all tasks with allocations
            // No: this is a duplication
            // for (task_id, allocations) in &task_allocs {
            //     // Skip tasks with no allocations
            //     if allocations.is_empty() {
            //         println!("Task {task_id} has no allocations, skipping");
            //         continue;
            //     }

            //     // Get the path for this task
            //     if let Some(path) = task_paths_map.get(task_id) {
            //         let path_str = path.join(";");
            //         let total_bytes: usize = allocations.iter().map(|(_, size)| *size).sum();
            //         println!("Writing for task {task_id}: '{path_str}' with {total_bytes} bytes");

            //         // Write line to folded format file
            //         match writeln!(writer, "{} {}", path_str, total_bytes) {
            //             Ok(()) => {
            //                 // lines_written += 1;
            //                 already_written.insert(path_str.clone());
            //             }
            //             Err(e) => println!("Error writing line for task {task_id}: {e}"),
            //         }
            //     } else {
            //         println!("No path found for task {task_id}");
            //     }
            // }

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

                println!("Writing for task {task_id} from registry: '{path_str}' with 0 bytes");

                // Write line with zero bytes to maintain call hierarchy
                match writeln!(writer, "{} {}", path_str, 0) {
                    Ok(()) => {
                        // lines_written += 1;
                        already_written.insert(path_str.clone());
                    }
                    Err(e) => println!("Error writing line for task {task_id}: {e}"),
                }
            }

            // Make sure to flush the writer
            if let Err(e) = writer.flush() {
                println!("Error flushing writer: {e}");
            } else {
                // println!("Successfully wrote {lines_written} lines and flushed buffer");
            }
        }
    });
}
