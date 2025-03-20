#![allow(clippy::uninlined_format_args)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling.

use std::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "full_profiling")]
use crate::profiling::{clean_stack_trace, extract_path, extract_raw_frames};

#[cfg(feature = "full_profiling")]
use std::{
    alloc::System,
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock, Mutex,
    },
    thread::{self, ThreadId},
};

#[cfg(feature = "full_profiling")]
const MINIMUM_TRACKED_SIZE: usize = 1024;

// #[cfg(feature = "full_profiling")]
// use backtrace::Backtrace;

#[derive(Debug)]
#[cfg(feature = "full_profiling")]
struct AllocationRegistry {
    // Task ID
    active_profiles: BTreeSet<usize>,

    // Thread ID -> Stack of active task IDs (most recent on top)
    thread_task_stacks: HashMap<ThreadId, Vec<usize>>,

    // Task ID -> Allocations mapping
    task_allocations: HashMap<usize, Vec<(usize, usize)>>,

    // Address -> Task ID mapping for deallocations
    address_to_task: HashMap<usize, usize>,
}

#[cfg(feature = "full_profiling")]
impl AllocationRegistry {
    // Helper method to add to AllocationRegistry or wherever appropriate
    #[allow(dead_code)]
    pub fn log_status(&self) {
        println!("REGISTRY STATUS:");
        println!("  Active threads: {}", self.thread_task_stacks.len());

        for (thread_id, stack) in &self.thread_task_stacks {
            println!(
                "  Thread {:?}: {} tasks - {:?}",
                thread_id,
                stack.len(),
                stack
            );
        }

        println!("  Tracked tasks: {}", self.task_allocations.len());

        for (task_id, allocs) in &self.task_allocations {
            let total = allocs.iter().map(|(_, size)| *size).sum::<usize>();
            println!(
                "    Task {}: {} allocations, {} bytes total",
                task_id,
                allocs.len(),
                total
            );
        }
    }
}

#[cfg(feature = "full_profiling")]
static REGISTRY: LazyLock<Mutex<AllocationRegistry>> = LazyLock::new(|| {
    Mutex::new(AllocationRegistry {
        active_profiles: BTreeSet::new(),
        thread_task_stacks: HashMap::new(),
        task_allocations: HashMap::new(),
        address_to_task: HashMap::new(),
    })
});

/// Task-aware allocator that tracks memory usage per task ID
#[derive(Debug)]
pub struct TaskAwareAllocator<A: GlobalAlloc> {
    /// The inner allocator that actually performs allocation
    inner: A,
    // /// Counter for generating unique task IDs
    // next_task_id: AtomicUsize,
}

/// Task context for tracking allocations
#[cfg(feature = "full_profiling")]
#[derive(Debug, Clone)]
pub struct TaskMemoryContext {
    task_id: usize,
    allocator: &'static TaskAwareAllocator<System>,
}

// Define registry-specific methods for System allocator
#[cfg(feature = "full_profiling")]
impl TaskAwareAllocator<System> {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize task data
        if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
            task_map.insert(
                task_id,
                TaskData {
                    // allocations: Vec::new(),
                    active: false,
                },
            );
        }

        // Also initialize in registry
        if let Ok(mut registry) = REGISTRY.lock() {
            registry.task_allocations.insert(task_id, Vec::new());
        }

        TaskMemoryContext {
            task_id,
            allocator: self,
        }
    }

    #[allow(clippy::unused_self)]
    pub fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        REGISTRY.lock().map_or(None, |registry| {
            registry
                .task_allocations
                .get(&task_id)
                .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
        })
    }

    #[allow(clippy::unused_self)]
    pub fn enter_task(&self, task_id: usize) -> Result<(), String> {
        let thread_id = thread::current().id();

        REGISTRY.lock().map_or_else(
            |_| Err("Failed to lock registry".to_string()),
            |mut registry| {
                // Get or create task stack for this thread
                let task_stack = registry.thread_task_stacks.entry(thread_id).or_default();

                // Push this task onto the stack
                task_stack.push(task_id);

                // println!("ENTER: Thread {:?} task stack: {:?}", thread_id, task_stack);

                // Initialize allocation tracking if needed
                registry.task_allocations.entry(task_id).or_default();

                // registry.log_status();
                Ok(())
            },
        )
    }

    #[allow(clippy::unused_self)]
    pub fn exit_task(&self, task_id: usize) -> Result<(), String> {
        let thread_id = thread::current().id();

        if let Ok(mut registry) = REGISTRY.lock() {
            // Get stack for this thread
            if let Some(task_stack) = registry.thread_task_stacks.get_mut(&thread_id) {
                // Check if our task is on top of the stack
                if let Some(&top_task) = task_stack.last() {
                    if top_task == task_id {
                        // Pop our task off the stack
                        task_stack.pop();

                        // println!("EXIT: Thread {:?} task stack: {:?}", thread_id, task_stack);

                        // If stack is now empty, remove it
                        if task_stack.is_empty() {
                            registry.thread_task_stacks.remove(&thread_id);
                        }

                        return Ok(());
                    }
                    println!(
                        "Task conflict detected between {} and {}",
                        task_id, top_task
                    );
                    compare_task_paths(task_id, top_task);

                    return Err(format!(
                        "Task stack corruption: trying to exit task {} but {} is on top",
                        task_id, top_task
                    ));
                }
            }

            Err(format!("Thread {:?} has no active tasks", thread_id))
        } else {
            Err("Failed to lock registry".to_string())
        }
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        #[cfg(feature = "full_profiling")]
        if !ptr.is_null() {
            // Prevent recursion during allocation tracking
            thread_local! {
                static TRACKING_ALLOCATION: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
            }

            let should_track = TRACKING_ALLOCATION.with(|tracking| {
                if *tracking.borrow() {
                    false
                } else {
                    *tracking.borrow_mut() = true;
                    true
                }
            });

            if should_track {
                // Execute tracking code and reset flag afterward
                // eprintln!("{:#?}", backtrace::Backtrace::new());

                let _guard = scopeguard::guard((), |()| {
                    TRACKING_ALLOCATION.with(|tracking| {
                        *tracking.borrow_mut() = false;
                    });
                });

                let use_backtrace = layout.size() >= MINIMUM_TRACKED_SIZE;
                let maybe_path = if use_backtrace {
                    let start_pattern = "::TaskAwareAllocator";

                    let raw_frames = extract_raw_frames(start_pattern);

                    // Process the collected frames to collapse patterns and clean up
                    let cleaned_stack = clean_stack_trace(&raw_frames);
                    // eprintln!("cleaned_stack={cleaned_stack:?}");

                    Some(extract_path(&cleaned_stack))
                } else {
                    None
                };

                // Record allocation
                let address = ptr as usize;
                let size = layout.size();
                let thread_id = thread::current().id();

                // println!("{:#?}", backtrace::Backtrace::new());
                // Get current thread's active task stack
                if let Ok(mut registry) = REGISTRY.try_lock() {
                    let maybe_task_id = if use_backtrace {
                        maybe_path.and_then(|path| find_matching_profile(&path, &registry))
                    } else {
                        None
                    };
                    let maybe_task_id = if maybe_task_id.is_some() {
                        maybe_task_id
                    } else if let Some(task_stack) = registry.thread_task_stacks.get(&thread_id) {
                        // Attribute to the topmost task on the stack, if available
                        task_stack.last().copied()
                    } else {
                        None
                    };
                    if let Some(task_id) = maybe_task_id {
                        // Record allocation for this task
                        registry
                            .task_allocations
                            .entry(task_id)
                            .or_default()
                            .push((address, size));

                        // // Temp display to verify allocations
                        // // Count total memory for this task
                        // let task_total = registry.task_allocations[&top_task_id]
                        //     .iter()
                        //     .map(|(_, s)| *s)
                        //     .sum::<usize>();

                        // println!(
                        //     "ALLOC: Task {} +{} bytes (total: {} bytes)",
                        //     top_task_id, size, task_total
                        // );

                        // Map address to task for deallocation
                        registry.address_to_task.insert(address, task_id);
                    }
                }
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        #[cfg(feature = "full_profiling")]
        if !ptr.is_null() {
            // Prevent recursion during deallocation tracking
            thread_local! {
                static TRACKING_DEALLOCATION: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
            }

            let should_track = TRACKING_DEALLOCATION.with(|tracking| {
                if *tracking.borrow() {
                    false
                } else {
                    *tracking.borrow_mut() = true;
                    true
                }
            });

            if should_track {
                // Execute tracking code and reset flag afterward
                let _guard = scopeguard::guard((), |()| {
                    TRACKING_DEALLOCATION.with(|tracking| {
                        *tracking.borrow_mut() = false;
                    });
                });

                let address = ptr as usize;

                // Record deallocation
                if let Ok(mut registry) = REGISTRY.try_lock() {
                    if let Some(task_id) = registry.address_to_task.remove(&address) {
                        // // Get size before removing
                        // let size = registry
                        //     .task_allocations
                        //     .get(&task_id)
                        //     .and_then(|allocs| {
                        //         allocs
                        //             .iter()
                        //             .find(|(addr, _)| *addr == address)
                        //             .map(|(_, size)| *size)
                        //     })
                        //     .unwrap_or(0);

                        // Remove from task's allocation list
                        if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                            if let Some(pos) =
                                allocations.iter().position(|(addr, _)| *addr == address)
                            {
                                allocations.swap_remove(pos);

                                // // Temp display: Report total after removal
                                // let task_total = allocations.iter().map(|(_, s)| *s).sum::<usize>();
                                // println!(
                                //     "DEALLOC: Task {} -{} bytes (total: {} bytes)",
                                //     task_id, size, task_total
                                // );
                            }
                        }
                    }
                }
            }
        }

        self.inner.dealloc(ptr, layout);
    }
}

#[cfg(feature = "full_profiling")]
impl TaskMemoryContext {
    /// Gets the unique ID for this task
    pub const fn id(&self) -> usize {
        self.task_id
    }

    /// Gets current memory usage for this task
    pub fn memory_usage(&self) -> Option<usize> {
        self.allocator.get_task_memory_usage(self.task_id)
    }
}

// Provide a dummy TaskMemoryContext type for when full_profiling is disabled
#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskMemoryContext;

/// Creates a standalone memory guard that activates the given task ID
///
/// # Errors
///
/// This function will bubble up any error from `TaskAwareAllocator::enter_task`.
#[cfg(feature = "full_profiling")]
pub fn create_memory_guard(task_id: usize) -> Result<TaskGuard, String> {
    // Get the allocator
    let allocator = get_allocator();

    // Enter the task (now thread-aware)
    match allocator.enter_task(task_id) {
        Ok(()) => {
            // Create a guard that's tied to this thread and task
            let task_guard = TaskGuard::new(task_id);
            println!(
                "GUARD CREATED: Task {} on thread {:?}",
                task_id,
                thread::current().id()
            );
            Ok(task_guard)
        }
        Err(e) => Err(e),
    }
}
// Task tracking state
#[cfg(feature = "full_profiling")]
struct TaskState {
    // Maps task IDs to their tracking state
    task_map: Mutex<HashMap<usize, TaskData>>,
    // Counter for generating task IDs
    next_task_id: AtomicUsize,
}

// Per-task data
#[cfg(feature = "full_profiling")]
struct TaskData {
    // allocations: Vec<(usize, usize)>,
    active: bool,
}

// Global task state
#[cfg(feature = "full_profiling")]
static TASK_STATE: LazyLock<TaskState> = LazyLock::new(|| {
    // println!("Initializing TASK_STATE with next_task_id = 1");
    TaskState {
        task_map: Mutex::new(HashMap::new()),
        next_task_id: AtomicUsize::new(1),
    }
});

// To handle active task tracking, instead of thread-locals, we'll use task-specific techniques
#[cfg(feature = "full_profiling")]
#[derive(Debug)]
pub struct TaskGuard {
    task_id: usize,
}

#[cfg(feature = "full_profiling")]
impl TaskGuard {
    pub const fn new(task_id: usize) -> Self {
        Self { task_id }
    }
}

#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskGuard;

#[cfg(feature = "full_profiling")]
impl Drop for TaskGuard {
    fn drop(&mut self) {
        // Try to exit task cleanly
        if let Err(e) = get_allocator().exit_task(self.task_id) {
            // Just log errors, don't panic in drop
            eprintln!("Error exiting task {}: {}", self.task_id, e);
        }

        // Also update the task's active status
        if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
            if let Some(data) = task_map.get_mut(&self.task_id) {
                data.active = false;
            }
        }
        deactivate_profile(self.task_id);
        println!(
            "GUARD DROPPED: Task {} on thread {:?}",
            self.task_id,
            thread::current().id()
        );
    }
}

#[cfg(feature = "full_profiling")]
#[global_allocator]
static ALLOCATOR: TaskAwareAllocator<System> = TaskAwareAllocator { inner: System };

// Helper to get the allocator instance
#[cfg(feature = "full_profiling")]
pub fn get_allocator() -> &'static TaskAwareAllocator<System> {
    &ALLOCATOR
}

/// Creates a new task context for memory tracking.
#[cfg(feature = "full_profiling")]
pub fn create_memory_task() -> TaskMemoryContext {
    let allocator = get_allocator();
    allocator.create_task_context()
}

// Task Path Registry for debugging
// 1. Declare the registry
#[cfg(feature = "full_profiling")]
static TASK_PATH_REGISTRY: LazyLock<Mutex<BTreeMap<usize, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

// 2. Function to add a task's path to the registry
#[cfg(feature = "full_profiling")]
pub fn register_task_path(task_id: usize, path: Vec<String>) {
    if let Ok(mut registry) = TASK_PATH_REGISTRY.lock() {
        registry.insert(task_id, path);
    } else {
        eprintln!(
            "Failed to lock task path registry when registering task {}",
            task_id
        );
    }
}

// 3. Function to look up a task's path by ID
#[cfg(feature = "full_profiling")]
pub fn lookup_task_path(task_id: usize) -> Option<Vec<String>> {
    TASK_PATH_REGISTRY
        .lock()
        .ok()
        .and_then(|registry| registry.get(&task_id).cloned())
}

// 4. Function to dump the entire registry
#[allow(dead_code)]
#[cfg(feature = "full_profiling")]
pub fn dump_task_path_registry() {
    if let Ok(registry) = TASK_PATH_REGISTRY.lock() {
        println!("==== TASK PATH REGISTRY DUMP ====");
        println!("Total registered tasks: {}", registry.len());

        for (task_id, path) in registry.iter() {
            println!("Task {}: {}", task_id, path.join("::"));
        }
        println!("=================================");
    } else {
        eprintln!("Failed to lock task path registry for dumping");
    }
}

// 5. Utility function to look up and print a specific task's path
#[allow(dead_code)]
#[cfg(feature = "full_profiling")]
pub fn print_task_path(task_id: usize) {
    match lookup_task_path(task_id) {
        Some(path) => println!("Task {} path: {}", task_id, path.join("::")),
        None => println!("No path registered for task {}", task_id),
    }
}

// 6. Utility to compare two tasks' paths
#[cfg(feature = "full_profiling")]
pub fn compare_task_paths(task_id1: usize, task_id2: usize) {
    let path1 = lookup_task_path(task_id1);
    let path2 = lookup_task_path(task_id2);

    println!("==== TASK PATH COMPARISON ====");
    match (path1, path2) {
        (Some(p1), Some(p2)) => {
            println!("Task {}: {}", task_id1, p1.join("::"));
            println!("Task {}: {}", task_id2, p2.join("::"));

            // Find common prefix
            let common_len = p1.iter().zip(p2.iter()).take_while(|(a, b)| a == b).count();

            if common_len > 0 {
                println!("Common prefix: {}", p1[..common_len].join("::"));
                println!("Diverges at: {}", common_len);
            } else {
                println!("No common prefix");
            }
        }
        (Some(p1), None) => {
            println!("Task {}: {}", task_id1, p1.join("::"));
            println!("Task {}: No path registered", task_id2);
        }
        (None, Some(p2)) => {
            println!("Task {}: No path registered", task_id1);
            println!("Task {}: {}", task_id2, p2.join("::"));
        }
        (None, None) => {
            println!("No paths registered for either task");
        }
    }
    println!("=============================");
}

// Helper function to find the best matching profile
#[cfg(feature = "full_profiling")]
fn find_matching_profile(path: &[String], registry: &AllocationRegistry) -> Option<usize> {
    // Get active profiles with their call stacks
    // let active_profiles = registry
    //     .active_profiles
    //     .iter()
    //     .filter(|(_, is_active)| **is_active)
    //     .map(|(task_id, _)| *task_id)
    //     .collect::<Vec<_>>();

    if registry.active_profiles.is_empty() {
        return None;
    }

    // If we have task path information, use it for matching
    if let Ok(path_registry) = TASK_PATH_REGISTRY.try_lock() {
        // For each active profile, compute a similarity score
        let mut best_match = None;
        let mut best_score = 0;

        for task_id in registry.active_profiles.iter().rev() {
            if let Some(task_path) = path_registry.get(task_id) {
                let score = compute_similarity(task_path, path);
                if score > best_score {
                    best_score = score;
                    best_match = Some(task_id);
                }
            }
        }

        return best_match.copied();
    }

    // Fallback: Return the most recently activated profile
    registry.active_profiles.last().copied()
}

// Compute similarity between a task path and backtrace frames
#[cfg(feature = "full_profiling")]
fn compute_similarity(task_path: &[String], backtrace_frames: &[String]) -> usize {
    let mut score = 0;

    // Count how many functions in the task path appear in the backtrace
    for path_func in task_path {
        if backtrace_frames
            .iter()
            .any(|frame| frame.contains(path_func))
        {
            score += 1;
        }
    }
    score
}

// When creating a profile:
#[cfg(feature = "full_profiling")]
pub fn activate_profile(task_id: usize) {
    if let Ok(mut registry) = REGISTRY.lock() {
        registry.active_profiles.insert(task_id);
    }
}

// When dropping a profile:
#[cfg(feature = "full_profiling")]
pub fn deactivate_profile(task_id: usize) {
    if let Ok(mut registry) = REGISTRY.lock() {
        registry.active_profiles.remove(&task_id);
    }
}
