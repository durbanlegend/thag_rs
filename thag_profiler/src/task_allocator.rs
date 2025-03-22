#![allow(clippy::uninlined_format_args)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling.

use std::{
    alloc::{GlobalAlloc, Layout},
    // collections::HashSet,
    // time::Duration,
};

#[cfg(feature = "full_profiling")]
use crate::profiling::{extract_callstack, extract_path};

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
const MINIMUM_TRACKED_SIZE: usize = 64;

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
    // // Helper method to add to AllocationRegistry or wherever appropriate
    // #[allow(dead_code)]
    // pub fn log_status(&self) {
    //     println!("REGISTRY STATUS:");
    //     println!("  Active threads: {}", self.thread_task_stacks.len());

    //     for (thread_id, stack) in &self.thread_task_stacks {
    //         println!(
    //             "  Thread {:?}: {} tasks - {:?}",
    //             thread_id,
    //             stack.len(),
    //             stack
    //         );
    //     }

    //     println!("  Tracked tasks: {}", self.task_allocations.len());

    //     for (task_id, allocs) in &self.task_allocations {
    //         let total = allocs.iter().map(|(_, size)| *size).sum::<usize>();
    //         println!(
    //             "    Task {}: {} allocations, {} bytes total",
    //             task_id,
    //             allocs.len(),
    //             total
    //         );
    //     }
    // }
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
        if let Ok(mut task_map) = TASK_STATE.task_map.try_lock() {
            task_map.insert(
                task_id,
                TaskData {
                    // allocations: Vec::new(),
                    active: false,
                },
            );
        } else {
            eprintln!(
                "Failed to lock TASK_STATE to initialize task data: {}",
                task_id
            );
        }

        // Also initialize in registry
        // eprintln!("About to try_lock registry to initialize task data");
        // if let Ok(mut registry) = REGISTRY.try_lock() {
        //     registry.task_allocations.insert(task_id, Vec::new());
        //     registry.active_profiles.insert(task_id);
        // } else {
        //     eprintln!(
        //         "Failed to lock registry to initialize task data: {}",
        //         task_id
        //     );
        // }
        activate_task(task_id);

        TaskMemoryContext {
            task_id,
            allocator: self,
        }
    }

    #[allow(clippy::unused_self)]
    pub fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        // // eprintln!("About to try_lock registry to get task memory usage");
        // REGISTRY.try_lock().map_or(None, |registry| {
        //     registry
        //         .task_allocations
        //         .get(&task_id)
        //         .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
        // })
        let maybe_task_allocations = get_task_allocations(task_id);
        if let Some(task_allocations) = maybe_task_allocations {
            Some(task_allocations.iter().map(|(_, size)| *size).sum())
        } else {
            None
        }
    }

    #[allow(clippy::unused_self)]
    pub fn enter_task(&self, task_id: usize) -> Result<(), String> {
        // eprintln!("Entering task {}", task_id);
        let thread_id = thread::current().id();

        push_task_to_stack(thread_id, task_id);
        Ok(())
        // REGISTRY.try_lock().map_or_else(
        //     |_| Err("Failed to lock registry".to_string()),
        //     |mut registry| {
        //         // Get or create task stack for this thread
        //         let task_stack = registry.thread_task_stacks.entry(thread_id).or_default();

        //         // Push this task onto the stack
        //         task_stack.push(task_id);

        //         // println!("ENTER: Thread {:?} task stack: {:?}", thread_id, task_stack);

        //         // Initialize allocation tracking if needed
        //         registry.task_allocations.entry(task_id).or_default();

        //         // registry.log_status();
        //         eprintln!("...Entered task {}", task_id);
        //         Ok(())
        //     },
        // )
    }

    #[allow(clippy::unused_self)]
    pub fn exit_task(&self, task_id: usize) -> Result<(), String> {
        // eprintln!("Exiting task {}", task_id);
        let thread_id = thread::current().id();

        pop_task_from_stack(thread_id, task_id);
        Ok(())
        // eprintln!("About to try_lock registry to exit task {}", task_id);
        // if let Ok(mut registry) = REGISTRY.lock() {
        //     // Get stack for this thread
        //     if let Some(task_stack) = registry.thread_task_stacks.get_mut(&thread_id) {
        //         // Find the task in the stack (not just at the top)
        //         if let Some(position) = task_stack.iter().position(|&id| id == task_id) {
        //             // Remove this specific task from the stack
        //             task_stack.remove(position);

        //             // If stack is now empty, remove it
        //             if task_stack.is_empty() {
        //                 registry.thread_task_stacks.remove(&thread_id);
        //             }

        //             return Ok(());
        //         }

        //         // Task wasn't in the stack at all
        //         return Err(format!(
        //             "Task {} not found in thread {:?} stack",
        //             task_id, thread_id
        //         ));
        //     }

        //     eprintln!("...Exited task {}", task_id);
        //     Err(format!("Thread {:?} has no active tasks", thread_id))
        // } else {
        //     eprintln!("...Exited task {}", task_id);
        //     Err("Failed to lock registry to remove task".to_string())
        // }
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        #[cfg(feature = "full_profiling")]
        fn find_latest() -> usize {
            // Assign small allocations to latest task
            let thread_id = std::thread::current().id();

            let task_stack = query_tasks_for_thread(thread_id);
            if let Some(&task_id) = task_stack.last() {
                task_id
            } else {
                0
            }
            // if let Ok(registry) = REGISTRY.try_lock() {
            //     if let Some(task_stack) = registry.thread_task_stacks.get(&thread_id) {
            //         if let Some(&task_id) = task_stack.last() {
            //             task_id
            //         } else {
            //             0
            //         }
            //     } else {
            //         0
            //     }
            // } else {
            //     0
            // }
        }

        #[cfg(feature = "full_profiling")]
        if !ptr.is_null() {
            let task_id = if layout.size() >= MINIMUM_TRACKED_SIZE {
                // Simple recursion prevention
                thread_local! {
                    static IN_TRACKING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
                }

                let already_tracking = IN_TRACKING.with(|flag| {
                    let value = flag.get();
                    if !value {
                        flag.set(true);
                    }
                    value
                });

                if already_tracking {
                    eprintln!("Already tracking, i.e. recursion");
                    find_latest()
                } else {
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
                    let mut task_id = 0;
                    MultiAllocator::with(AllocatorTag::System, || {
                        // Now we can safely use backtrace without recursion!
                        // let start_pattern = "TaskAwareAllocator";
                        let start_pattern = "Profile::new";

                        // eprintln!("Calling extract_callstack");
                        let cleaned_stack = extract_callstack(start_pattern);
                        if cleaned_stack.is_empty() {
                            // eprintln!(
                            //     "Empty cleaned_stack for backtrace\n{:#?}",
                            //     backtrace::Backtrace::new()
                            // );
                            eprintln!("Empty cleaned_stack");
                            task_id = find_latest();
                        } else {
                            // Make sure the use of a separate allocator is working.
                            assert!(!cleaned_stack
                                .iter()
                                .any(|frame| frame.contains("find_matching_profile")));

                            // Make sure the use of a separate allocator is working.
                            assert!(!cleaned_stack
                                .iter()
                                .any(|frame| frame.contains("find_matching_profile")));

                            // eprintln!("Calling extract_path");
                            let path = extract_path(&cleaned_stack);
                            if path.is_empty() {
                                eprintln!(
                                    "...path is empty for thread {:?}, &cleaned_stack:\n{:#?}",
                                    thread::current().id(),
                                    cleaned_stack
                                );
                                task_id = find_latest();
                            } else {
                                // eprintln!("path={path:#?}");

                                task_id = find_matching_profile(&path);
                                // Try to get task ID from registry
                                // task_id = if let Ok(registry) = REGISTRY.try_lock() {
                                //     find_matching_profile(&path, &registry)
                                // } else {
                                //     eprintln!("Falling back to find_latest because failed to acquire registry lock");
                                //     find_latest()
                                // };
                            }
                        }
                    });
                    eprintln!(
                        "alloc found task id {task_id} for allocation of {} bytes",
                        layout.size()
                    );
                    task_id

                    // // Record allocation if task found
                    // if task_id > 0 {
                    //     let address = ptr as usize;
                    //     let size = layout.size();

                    //     if let Ok(mut registry) = REGISTRY.try_lock() {
                    //         registry
                    //             .task_allocations
                    //             .entry(task_id)
                    //             .or_default()
                    //             .push((address, size));

                    //         registry.address_to_task.insert(address, task_id);
                    //     }
                    // }
                }
            } else {
                find_latest()
            };

            // Use the background processor to record the allocation
            if task_id > 0 {
                let address = ptr as usize;
                let size = layout.size();
                record_allocation(task_id, address, size);
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        #[cfg(feature = "full_profiling")]
        if !ptr.is_null() {
            // Similar recursion prevention as in alloc
            thread_local! {
                static IN_TRACKING: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
            }

            let already_tracking = IN_TRACKING.with(|flag| {
                let value = *flag.borrow();
                if !value {
                    *flag.borrow_mut() = true;
                }
                value
            });

            if !already_tracking {
                let _guard = scopeguard::guard((), |()| {
                    IN_TRACKING.with(|flag| *flag.borrow_mut() = false);
                });

                let address = ptr as usize;

                // Record deallocation
                // Send deallocation notification to registry processor
                let _ = REGISTRY_CHANNEL
                    .0
                    .try_send(RegistryMessage::RecordDeallocation { address });
                // println!("About to try_lock registry for deallocation");
                // if let Ok(mut registry) = REGISTRY.try_lock() {
                //     // eprintln!("...success!");
                //     if let Some(task_id) = registry.address_to_task.remove(&address) {
                //         // // Get size before removing
                //         // let size = registry
                //         //     .task_allocations
                //         //     .get(&task_id)
                //         //     .and_then(|allocs| {
                //         //         allocs
                //         //             .iter()
                //         //             .find(|(addr, _)| *addr == address)
                //         //             .map(|(_, size)| *size)
                //         //     })
                //         //     .unwrap_or(0);

                //         // // Remove from task's allocation list
                //         // if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                //         //     if let Some(pos) =
                //         //         allocations.iter().position(|(addr, _)| *addr == address)
                //         //     {
                //         //         allocations.swap_remove(pos);

                //         //         // // Temp display: Report total after removal
                //         //         // let task_total = allocations.iter().map(|(_, s)| *s).sum::<usize>();
                //         //         // println!(
                //         //         //     "DEALLOC: Task {} -{} bytes (total: {} bytes)",
                //         //         //     task_id, size, task_total
                //         //         // );
                //         //     }
                //         // }
                //     }
                // } else {
                //     // eprintln!("Could not lock registry to record deallocation");
                // }
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
        // if let Err(e) = get_allocator().exit_task(self.task_id) {
        //     // Just log errors, don't panic in drop
        //     eprintln!("Error exiting task {}: {}", self.task_id, e);
        // }
        // Send deactivation message via channel
        // let _ = REGISTRY_CHANNEL
        //     .0
        //     .try_send(RegistryMessage::DeactivateTask {
        //         task_id: self.task_id,
        //     });
        deactivate_task(self.task_id);

        // Also update the task's active status
        // eprintln!(
        //     "About to try_lock registry for TaskGuard::drop for task {}",
        //     self.task_id
        // );
        if let Ok(mut task_map) = TASK_STATE.task_map.try_lock() {
            if let Some(data) = task_map.get_mut(&self.task_id) {
                data.active = false;
            }
        } else {
            eprintln!("Failed to lock task map to deactivate task");
        }

        remove_task_path(self.task_id);
        // deactivate_profile(self.task_id);

        println!(
            "GUARD DROPPED: Task {} on thread {:?}",
            self.task_id,
            thread::current().id()
        );
    }
}

#[cfg(feature = "full_profiling")]
// #[global_allocator]
static TASK_AWARE_ALLOCATOR: TaskAwareAllocator<System> = TaskAwareAllocator { inner: System };

// Helper to get the allocator instance
#[cfg(feature = "full_profiling")]
pub fn get_allocator() -> &'static TaskAwareAllocator<System> {
    &TASK_AWARE_ALLOCATOR
}

/// Creates a new task context for memory tracking.
#[cfg(feature = "full_profiling")]
pub fn create_memory_task() -> TaskMemoryContext {
    let allocator = get_allocator();
    allocator.create_task_context()
}

// Task Path Registry for debugging
// 1. Declare the TASK_PATH_REGISTRY
#[cfg(feature = "full_profiling")]
static TASK_PATH_REGISTRY: LazyLock<Mutex<BTreeMap<usize, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

// 2. Function to add a task's path to the TASK_PATH_REGISTRY
#[cfg(feature = "full_profiling")]
pub fn register_task_path(task_id: usize, path: Vec<String>) {
    // eprintln!("About to try_lock TASK_PATH_REGISTRY for register_task_path");
    if let Ok(mut registry) = TASK_PATH_REGISTRY.try_lock() {
        registry.insert(task_id, path);
    } else {
        eprintln!(
            "Failed to lock task path registry to registertask {}",
            task_id
        );
    }
}

// 3. Function to look up a task's path by ID
#[cfg(feature = "full_profiling")]
pub fn lookup_task_path(task_id: usize) -> Option<Vec<String>> {
    // eprintln!("About to try_lock TASK_PATH_REGISTRY for lookup_task_path");
    TASK_PATH_REGISTRY
        .try_lock()
        .ok()
        .and_then(|registry| registry.get(&task_id).cloned())
}

// 4. Function to dump the entire registry
#[allow(dead_code)]
#[cfg(feature = "full_profiling")]
pub fn dump_task_path_registry() {
    // eprintln!("About to try_lock TASK_PATH_REGISTRY for dump_task_path_registry");
    if let Ok(registry) = TASK_PATH_REGISTRY.try_lock() {
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
#[allow(dead_code)]
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

// 7. Function to remove an entry from the TASK_PATH_REGISTRY
#[cfg(feature = "full_profiling")]
pub fn remove_task_path(task_id: usize) {
    if let Ok(mut registry) = TASK_PATH_REGISTRY.try_lock() {
        registry.remove(&task_id);
    } else {
        eprintln!(
            "Failed to lock task path registry to remove task {}",
            task_id
        );
    }
}

// Helper function to find the best matching profile
#[cfg(feature = "full_profiling")]
fn find_matching_profile(path: &[String]) -> usize {
    if let Ok(path_registry) = TASK_PATH_REGISTRY.try_lock() {
        // eprintln!("...success!");
        // For each active profile, compute a similarity score
        let mut best_match = 0;
        let mut best_score = 0;
        let path_len = path.len();

        let mut score = 0;
        let active_profiles = query_active_profiles();
        eprintln!("...active profiles: {:?}", active_profiles);
        for task_id in active_profiles.iter().rev() {
            if let Some(reg_path) = path_registry.get(task_id) {
                score = compute_similarity(path, reg_path);
                eprintln!(
                    "...scored {score} checking task {} with path {:?}",
                    task_id,
                    reg_path.join(" -> ")
                );
                if score > best_score || score == path_len {
                    best_score = score;
                    best_match = *task_id;
                }
                if score == path_len {
                    break;
                }
            }
        }
        if best_score == path.len() {
            // eprintln!("...returning best match with perfect score of {}", score);
        } else {
            eprintln!(
                "...returning best match with imperfect score of {} vs path.len() = {} for path:\n{}",
                best_score,
                path.len(),
                path.join(" -> ")
            );
            println!("==== TASK PATH REGISTRY DUMP ====");
            println!("Total registered tasks: {}", path_registry.len());

            for (task_id, path) in path_registry.iter() {
                println!("Task {}: {}", task_id, path.join(" -> "));
            }
            println!("=================================");

            println!("registry.active_profiles={:#?}", active_profiles);
        }
        return best_match;
    }

    // Fallback: Return the most recently activated profile
    eprintln!("...returning fallback: most recently activated profile");
    // *registry.active_profiles.last().unwrap_or(&0)

    get_last_active_profile()
}

// Compute similarity between a task path and backtrace frames
#[cfg(feature = "full_profiling")]
fn compute_similarity(task_path: &[String], reg_path: &[String]) -> usize {
    if task_path.is_empty() || reg_path.is_empty() {
        eprintln!("task_path.is_empty() || reg_path.is_empty()");
        return 0;
    }

    let score = task_path
        .iter()
        .zip(reg_path.iter())
        .inspect(|(path_func, frame)| {
            eprintln!("Comparing [{}]\n          [{}]", path_func, frame);
        })
        .filter(|(path_func, frame)| frame == path_func)
        // .inspect(|(path_func, frame)| {
        //     let matched = frame == path_func;
        //     eprintln!("frame == path_func? {}", matched);
        //     if matched {
        //         score += 1;
        //     }
        // })
        .count();

    eprintln!("score={score}");
    if score == 0 {
        eprintln!("score = {score} for path of length {}", task_path.len(),);
        // let diff = create_side_by_side_diff(&task_path.join("->"), &reg_path.join("->"), 80);
        // println!("{diff}");
        println!("{}\n{}", task_path.join("->"), reg_path.join("->"));
    }

    score
}

// fn compute_similarity(task_path: &[String], reg_path: &[String]) -> usize {
//     if task_path.is_empty() || reg_path.is_empty() {
//         return 0;
//     }

//     let score = task_path
//         .iter()
//         .zip(reg_path.iter())
//         .inspect(|(path_func, frame)| {
//             eprintln!("Comparing [{}]\n          [{}]", path_func, frame);
//         })
//         .filter(|(path_func, frame)| frame.eq(path_func))
//         .inspect(|(path_func, frame)| {
//             eprintln!("frame == path_func? {}", frame.eq(path_func));
//         })
//         .count();
//     if score == 0 {
//         eprintln!("score = {score} for path of length {}", task_path.len(),);
//         // let diff = create_side_by_side_diff(&task_path.join("->"), &reg_path.join("->"), 80);
//         // println!("{diff}");
//         println!("{}\n{}", task_path.join("->"), reg_path.join("->"));
//     }

//     score
// }

// Merged into fn create_task_context
// // When creating a profile:
// #[cfg(feature = "full_profiling")]
// pub fn activate_profile(task_id: usize) {
//     // eprintln!("About to try_lock registry for activate_profile");
//     if let Ok(mut registry) = REGISTRY.try_lock() {
//         registry.active_profiles.insert(task_id);
//     } else {
//         eprintln!("Failed to lock registry to activate profile: {}", task_id);
//     }
// }

// // When dropping a profile:
// #[cfg(feature = "full_profiling")]
// pub fn deactivate_profile(task_id: usize) {
//     // eprintln!("About to try_lock registry for deactivate_profile");
//     if let Ok(mut registry) = REGISTRY.try_lock() {
//         registry.active_profiles.remove(&task_id);
//     } else {
//         eprintln!("Failed to lock registry activate profile: {}", task_id);
//     }
// }

// #[cfg(feature = "full_profiling")]
// pub fn init_allocator_system() {
//     // This is just to ensure the MultiAllocator is initialized
//     // The actual setup happens through the #[global_allocator] attribute

//     MultiAllocator::get_current_tag();
// }

// Setup for okaoka
#[cfg(feature = "full_profiling")]
okaoka::set_multi_global_allocator! {
    MultiAllocator, // Name of our allocator facade
    AllocatorTag,   // Name of our allocator tag enum
    Default => TaskAwareAllocatorWrapper,  // Our profiling allocator
    System => System,          // Standard system allocator for backtraces
}

// Wrapper to expose our TaskAwareAllocator to okaoka
#[cfg(feature = "full_profiling")]
struct TaskAwareAllocatorWrapper;

#[cfg(feature = "full_profiling")]
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
/// This is called by the main init_profiling function.
#[cfg(feature = "full_profiling")]
pub fn initialize_memory_profiling() {
    // Initialize the registry processor
    initialize_registry_processor();

    println!("Memory profiling initialized with background registry processor");
}

/// Finalize memory profiling and write out data.
/// This is called by the main finalize_profiling function.
#[cfg(feature = "full_profiling")]
pub fn finalize_memory_profiling() {
    // Write out memory profiling data
    if let Ok(registry) = REGISTRY.try_lock() {
        write_memory_profile_data(&registry);
    }
}

/// Write memory profiling data to a file
#[cfg(feature = "full_profiling")]
fn write_memory_profile_data(_registry: &AllocationRegistry) {
    // Logic to write out memory usage data to memory.folded file
    // ...
}

// #[cfg(feature = "full_profiling")]
// mod backtrace_utils {
//     use std::alloc::{GlobalAlloc, Layout, System};
//     use std::cell::RefCell;

//     // Thread-local flag to prevent recursion
//     thread_local! {
//         static IN_BACKTRACE: RefCell<bool> = RefCell::new(false);
//     }

//     // Function to capture backtrace safely
//     pub fn capture_backtrace() -> Option<String> {
//         // Check if we're already capturing to prevent recursion
//         let already_capturing = IN_BACKTRACE.with(|flag| {
//             let value = *flag.borrow();
//             if !value {
//                 *flag.borrow_mut() = true;
//                 false // Not already capturing
//             } else {
//                 true // Already capturing
//             }
//         });

//         if already_capturing {
//             return None;
//         }

//         // Set up cleanup guard
//         struct Guard;
//         impl Drop for Guard {
//             fn drop(&mut self) {
//                 IN_BACKTRACE.with(|flag| *flag.borrow_mut() = false);
//             }
//         }
//         let _guard = Guard;

//         // Use direct approach without any fancy allocation tracking
//         // This may still cause recursive allocation, but we guard against infinite recursion
//         let backtrace = backtrace::Backtrace::new();
//         Some(format!("{:?}", backtrace))
//     }

//     // Process a backtrace to find the appropriate task
//     pub fn find_task_for_backtrace(backtrace: &str) -> usize {
//         // To be implemented - for now just return 0 (no task)
//         0
//     }
// }

// // Re-export for public use
// #[cfg(feature = "full_profiling")]
// pub use backtrace_utils::capture_backtrace;

// // No-op version when profiling is disabled
// #[cfg(not(feature = "full_profiling"))]
// pub fn capture_backtrace() -> Option<String> {
//     None
// }

// Message types for registry operations
#[derive(Debug)]
enum RegistryMessage {
    // Active profiles operations
    ActivateTask {
        task_id: usize,
    },
    DeactivateTask {
        task_id: usize,
    },

    // Thread task stack operations
    PushTaskToStack {
        thread_id: ThreadId,
        task_id: usize,
    },
    PopTaskFromStack {
        thread_id: ThreadId,
        task_id: usize,
    },

    // Allocation tracking
    RecordAllocation {
        task_id: usize,
        address: usize,
        size: usize,
    },
    RecordDeallocation {
        address: usize,
    },

    // Control messages
    Flush,
    QueryTasksForThread {
        thread_id: ThreadId,
        response: channel::Sender<Vec<usize>>,
    },
    QueryActiveProfiles {
        response: channel::Sender<Vec<usize>>,
    },
    QueryTaskAllocations {
        task_id: usize,
        response: channel::Sender<Option<Vec<(usize, usize)>>>,
    },
}

// Global channel for registry updates
use crossbeam::channel;
use once_cell::sync::Lazy;

static REGISTRY_CHANNEL: once_cell::sync::Lazy<(
    channel::Sender<RegistryMessage>,
    channel::Receiver<RegistryMessage>,
)> = Lazy::new(|| {
    let (sender, receiver) = channel::unbounded();
    (sender, receiver)
});

// Initialize processor thread (call this during startup)
pub fn initialize_registry_processor() {
    let receiver = REGISTRY_CHANNEL.1.clone();

    std::thread::Builder::new()
        .name("memory-profiler-registry".to_string())
        .spawn(move || {
            registry_processor_thread(receiver);
        })
        .expect("Failed to spawn registry processor thread");
}

// Background thread for processing registry messages
fn registry_processor_thread(receiver: channel::Receiver<RegistryMessage>) {
    const BATCH_SIZE: usize = 100;
    const PROCESS_INTERVAL_MS: u64 = 5;

    let mut messages = Vec::with_capacity(BATCH_SIZE);

    loop {
        // Collect messages with timeout
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(PROCESS_INTERVAL_MS);

        while std::time::Instant::now() < deadline {
            match receiver.try_recv() {
                Ok(msg) => {
                    // Handle synchronous query messages immediately
                    match &msg {
                        RegistryMessage::QueryTasksForThread {
                            thread_id,
                            response,
                        } => {
                            let tasks = query_tasks_for_thread(*thread_id);
                            let _ = response.send(tasks);
                            continue; // Don't add to batch
                        }
                        RegistryMessage::QueryActiveProfiles { response } => {
                            let profiles = query_active_profiles();
                            let _ = response.send(profiles);
                            continue; // Don't add to batch
                        }
                        RegistryMessage::QueryTaskAllocations { task_id, response } => {
                            let allocs = query_task_allocations(*task_id);
                            let _ = response.send(allocs);
                            continue; // Don't add to batch
                        }
                        RegistryMessage::Flush => {
                            messages.push(msg);
                            break; // Process immediately
                        }
                        _ => {
                            messages.push(msg);
                            if messages.len() >= BATCH_SIZE {
                                break; // Process when batch is full
                            }
                        }
                    }
                }
                Err(channel::TryRecvError::Empty) => {
                    break; // No more messages for now
                }
                Err(channel::TryRecvError::Disconnected) => {
                    return; // Channel closed - exit thread
                }
            }
        }

        // Process any collected messages
        if !messages.is_empty() {
            process_registry_messages(&messages);
            messages.clear();
        }
    }
}

// Process a batch of registry messages
// Helper functions for synchronous queries
fn query_tasks_for_thread(thread_id: ThreadId) -> Vec<usize> {
    if let Ok(registry) = REGISTRY.try_lock() {
        if let Some(task_stack) = registry.thread_task_stacks.get(&thread_id) {
            return task_stack.clone();
        }
    }
    Vec::new()
}

fn query_active_profiles() -> Vec<usize> {
    if let Ok(registry) = REGISTRY.try_lock() {
        return registry.active_profiles.iter().copied().collect();
    }
    Vec::new()
}

fn query_task_allocations(task_id: usize) -> Option<Vec<(usize, usize)>> {
    if let Ok(registry) = REGISTRY.try_lock() {
        return registry.task_allocations.get(&task_id).cloned();
    }
    None
}

// Process a batch of registry messages
// Process a batch of registry messages
fn process_registry_messages(messages: &[RegistryMessage]) {
    // Try to acquire registry lock once for the batch
    if let Ok(mut registry) = REGISTRY.lock() {
        for msg in messages {
            match msg {
                RegistryMessage::ActivateTask { task_id } => {
                    registry.active_profiles.insert(*task_id);
                }
                RegistryMessage::DeactivateTask { task_id } => {
                    registry.active_profiles.remove(task_id);
                }
                RegistryMessage::PushTaskToStack { thread_id, task_id } => {
                    let task_stack = registry.thread_task_stacks.entry(*thread_id).or_default();
                    task_stack.push(*task_id);
                }
                RegistryMessage::PopTaskFromStack { thread_id, task_id } => {
                    // First, check if we need to remove the task
                    let mut should_remove_stack = false;

                    if let Some(task_stack) = registry.thread_task_stacks.get_mut(thread_id) {
                        if let Some(pos) = task_stack.iter().position(|&id| id == *task_id) {
                            task_stack.remove(pos);
                        }

                        // Check if stack is now empty
                        should_remove_stack = task_stack.is_empty();
                    }

                    // Now remove the stack if needed
                    if should_remove_stack {
                        registry.thread_task_stacks.remove(thread_id);
                    }
                }
                RegistryMessage::RecordAllocation {
                    task_id,
                    address,
                    size,
                } => {
                    // Record in task's allocation list
                    registry
                        .task_allocations
                        .entry(*task_id)
                        .or_default()
                        .push((*address, *size));

                    // Map address to task for deallocation
                    registry.address_to_task.insert(*address, *task_id);
                }
                RegistryMessage::RecordDeallocation { address } => {
                    if let Some(task_id) = registry.address_to_task.remove(address) {
                        if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                            if let Some(pos) =
                                allocations.iter().position(|(addr, _)| *addr == *address)
                            {
                                allocations.swap_remove(pos);
                            }
                        }
                    }
                }
                RegistryMessage::Flush => {
                    // Just a trigger for processing, no specific action needed
                }
                // Query messages are handled separately
                RegistryMessage::QueryTasksForThread { .. }
                | RegistryMessage::QueryActiveProfiles { .. }
                | RegistryMessage::QueryTaskAllocations { .. } => {
                    // These are handled synchronously before batching
                }
            }
        }
    }
}

// Helper functions for sending messages to the registry

// Add task to active profiles
pub fn activate_task(task_id: usize) {
    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::ActivateTask { task_id });
}

// Remove task from active profiles
pub fn deactivate_task(task_id: usize) {
    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::DeactivateTask { task_id });
}

// Add task to thread's stack
pub fn push_task_to_stack(thread_id: ThreadId, task_id: usize) {
    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::PushTaskToStack { thread_id, task_id });
}

// Remove task from thread's stack
pub fn pop_task_from_stack(thread_id: ThreadId, task_id: usize) {
    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::PopTaskFromStack { thread_id, task_id });
}

// Record memory allocation
pub fn record_allocation(task_id: usize, address: usize, size: usize) {
    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::RecordAllocation {
            task_id,
            address,
            size,
        });
}

// Record memory deallocation
pub fn record_deallocation(address: usize) {
    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::RecordDeallocation { address });
}

// Get tasks for a thread (synchronous)
pub fn get_tasks_for_thread(thread_id: ThreadId) -> Vec<usize> {
    let (sender, receiver) = channel::bounded(1);

    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::QueryTasksForThread {
            thread_id,
            response: sender,
        });

    // Wait for response with timeout
    match receiver.recv_timeout(std::time::Duration::from_millis(5)) {
        Ok(tasks) => tasks,
        Err(_) => Vec::new(), // Fallback on timeout
    }
}

// Get active profiles (synchronous)
pub fn get_active_profiles() -> Vec<usize> {
    let (sender, receiver) = channel::bounded(1);

    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::QueryActiveProfiles { response: sender });

    // Wait for response with timeout
    match receiver.recv_timeout(std::time::Duration::from_millis(5)) {
        Ok(profiles) => profiles,
        Err(_) => Vec::new(), // Fallback on timeout
    }
}

// Get task allocations (synchronous)
pub fn get_task_allocations(task_id: usize) -> Option<Vec<(usize, usize)>> {
    let (sender, receiver) = channel::bounded(1);

    let _ = REGISTRY_CHANNEL
        .0
        .try_send(RegistryMessage::QueryTaskAllocations {
            task_id,
            response: sender,
        });

    // Wait for response with timeout
    match receiver.recv_timeout(std::time::Duration::from_millis(5)) {
        Ok(allocations) => allocations,
        Err(_) => None, // Fallback on timeout
    }
}

// Get last active profile
pub fn get_last_active_profile() -> usize {
    let profiles = get_active_profiles();
    profiles.last().copied().unwrap_or(0)
}
