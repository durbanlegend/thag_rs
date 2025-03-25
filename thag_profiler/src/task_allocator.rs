#![allow(clippy::uninlined_format_args)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling.

use std::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "full_profiling")]
use crate::profiling::{extract_callstack_from_backtrace, extract_path};

#[cfg(feature = "full_profiling")]
use std::{
    alloc::System,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    io::Write,
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock,
    },
    thread::{self, ThreadId},
};

#[cfg(feature = "full_profiling")]
use backtrace::Backtrace;

#[cfg(feature = "full_profiling")]
use parking_lot::Mutex;

#[cfg(feature = "full_profiling")]
const MINIMUM_TRACKED_SIZE: usize = 64;

/// Registry for tracking memory allocations and deallocations
#[derive(Debug)]
#[cfg(feature = "full_profiling")]
struct AllocationRegistry {
    /// Task ID -> Allocations mapping: [(address, size)]
    task_allocations: HashMap<usize, Vec<(usize, usize)>>,

    /// Address -> Task ID mapping for deallocations
    address_to_task: HashMap<usize, usize>,
}

#[cfg(feature = "full_profiling")]
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
#[cfg(feature = "full_profiling")]
static ALLOC_REGISTRY: LazyLock<Mutex<AllocationRegistry>> =
    LazyLock::new(|| Mutex::new(AllocationRegistry::new()));

// Thread-local buffers for pending allocation operations
#[cfg(feature = "full_profiling")]
thread_local! {
    // Buffer for pending allocations: (task_id, address, size)
    static ALLOCATION_BUFFER: RefCell<Vec<(usize, usize, usize)>> =
        RefCell::new(Vec::with_capacity(100));

    // Buffer for pending deallocations: address
    static DEALLOCATION_BUFFER: RefCell<Vec<usize>> =
        RefCell::new(Vec::with_capacity(100));
}

// ---------- Profile Registry ----------

/// Registry for tracking active profiles and task stacks
#[derive(Debug)]
#[cfg(feature = "full_profiling")]
struct ProfileRegistry {
    /// Set of active task IDs
    active_profiles: BTreeSet<usize>,

    /// Thread ID -> Stack of active task IDs (most recent on top)
    thread_task_stacks: HashMap<ThreadId, Vec<usize>>,
}

#[cfg(feature = "full_profiling")]
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
#[cfg(feature = "full_profiling")]
static PROFILE_REGISTRY: LazyLock<Mutex<ProfileRegistry>> =
    LazyLock::new(|| Mutex::new(ProfileRegistry::new()));

// ---------- Public Registry API ----------

/// Add a task to active profiles
#[cfg(feature = "full_profiling")]
pub fn activate_task(task_id: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
        PROFILE_REGISTRY.lock().activate_task(task_id);
    });
}

/// Remove a task from active profiles
#[cfg(feature = "full_profiling")]
pub fn deactivate_task(task_id: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
        // Process any pending allocations before deactivating
        process_pending_allocations();

        PROFILE_REGISTRY.lock().deactivate_task(task_id);
    });
}

/// Get the memory usage for a specific task
#[cfg(feature = "full_profiling")]
pub fn get_task_memory_usage(task_id: usize) -> Option<usize> {
    // Process any pending allocations first
    process_pending_allocations();

    ALLOC_REGISTRY.lock().get_task_memory_usage(task_id)
}

/// Add a task to a thread's stack
#[cfg(feature = "full_profiling")]
pub fn push_task_to_stack(thread_id: ThreadId, task_id: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
        PROFILE_REGISTRY
            .lock()
            .push_task_to_stack(thread_id, task_id);
    });
}

/// Remove a task from a thread's stack
#[cfg(feature = "full_profiling")]
pub fn pop_task_from_stack(thread_id: ThreadId, task_id: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
        PROFILE_REGISTRY
            .lock()
            .pop_task_from_stack(thread_id, task_id);
    });
}

/// Get active tasks
#[cfg(feature = "full_profiling")]
pub fn get_active_tasks() -> Vec<usize> {
    let mut active_tasks = Box::new(vec![]);
    MultiAllocator::with(AllocatorTag::System, || {
        let active = PROFILE_REGISTRY.lock().get_active_tasks();
        active_tasks = Box::new(active);
    });
    *active_tasks
}

/// Get the last active task
#[cfg(feature = "full_profiling")]
pub fn get_last_active_task() -> Option<usize> {
    let mut last_active_task: Box<Option<usize>> = Box::new(None);
    MultiAllocator::with(AllocatorTag::System, || {
        let last_active = PROFILE_REGISTRY.lock().get_last_active_task();
        last_active_task = Box::new(last_active);
    });
    *last_active_task
}

// /// Get the top task for a thread
// #[cfg(feature = "full_profiling")]
// pub fn get_top_task_for_thread(thread_id: ThreadId) -> Option<usize> {
//     let mut top_task_for_thread: Box<Option<usize>> = Box::new(None);
//     MultiAllocator::with(AllocatorTag::System, || {
//         let top_task = PROFILE_REGISTRY
//             .try_lock()
//             .map_or(None, |registry| registry.get_top_task_for_thread(thread_id));
//         top_task_for_thread = Box::new(top_task);
//     });
//     *top_task_for_thread
// }

// ---------- Allocation Tracking ----------

/// Record a memory allocation in the thread-local buffer
#[cfg(feature = "full_profiling")]
pub fn record_allocation(task_id: usize, address: usize, size: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
        // eprintln!("Allocating {} bytes at address {}", size, address);
        ALLOCATION_BUFFER.with(|buffer| {
            let mut allocs = buffer.borrow_mut();
            allocs.push((task_id, address, size));

            // Process if buffer is getting full
            if allocs.len() >= 50 {
                // eprintln!("allocs.len() >= 50, time to process pending allocations");
                // Drop mutable borrow before processing
                drop(allocs);
                // eprintln!("...dropped allocs");
                process_pending_allocations();
                eprintln!("...processed pending allocations");
            }
        });
    });
}

/// Record a memory deallocation in the thread-local buffer
#[cfg(feature = "full_profiling")]
pub fn record_deallocation(address: usize) {
    MultiAllocator::with(AllocatorTag::System, || {
        DEALLOCATION_BUFFER.with(|buffer| {
            let mut deallocs = buffer.borrow_mut();
            deallocs.push(address);

            // Process if buffer is getting full
            if deallocs.len() >= 50 {
                // Drop mutable borrow before processing
                drop(deallocs);
                process_pending_allocations();
            }
        });
    });
}

/// Process pending allocations and deallocations
#[cfg(feature = "full_profiling")]
pub fn process_pending_allocations() {
    MultiAllocator::with(AllocatorTag::System, || {
        // Process allocations
        let allocations: Vec<(usize, usize, usize)> = ALLOCATION_BUFFER.with(|buffer| {
            let mut allocs = buffer.borrow_mut();
            let result = allocs.clone();
            allocs.clear();
            result
        });

        if !allocations.is_empty() {
            let mut registry = ALLOC_REGISTRY.lock();
            for (task_id, address, size) in allocations {
                registry
                    .task_allocations
                    .entry(task_id)
                    .or_default()
                    .push((address, size));

                registry.address_to_task.insert(address, task_id);
            }
        }

        // Process deallocations
        let deallocations: Vec<usize> = DEALLOCATION_BUFFER.with(|buffer| {
            let mut deallocs = buffer.borrow_mut();
            let result = deallocs.clone();
            deallocs.clear();
            result
        });

        if !deallocations.is_empty() {
            let mut registry = ALLOC_REGISTRY.lock();
            for address in deallocations {
                if let Some(task_id) = registry.address_to_task.remove(&address) {
                    if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                        if let Some(pos) = allocations.iter().position(|(addr, _)| *addr == address)
                        {
                            allocations.swap_remove(pos);
                        }
                    }
                }
            }
        }
    });
}

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
    // allocator: &'static TaskAwareAllocator<System>,
}

// Define registry-specific methods for System allocator
#[cfg(feature = "full_profiling")]
#[allow(clippy::unused_self)]
impl TaskAwareAllocator<System> {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize task data
        let _task_map = TASK_STATE.task_map.lock().insert(
            task_id,
            TaskData {
                // allocations: Vec::new(),
                active: false,
            },
        );

        // Also initialize in profile registry
        activate_task(task_id);
        // // eprintln!("About to try_lock registry to initialize task data");
        // if let Ok(mut registry) = REGISTRY.try_lock() {
        //     registry.task_allocations.insert(task_id, Vec::new());
        //     registry.active_profiles.insert(task_id);
        // } else {
        //     eprintln!(
        //         "Failed to lock registry to initialize task data: {}",
        //         task_id
        //     );
        // }

        TaskMemoryContext {
            task_id,
            // allocator: self,
        }
    }

    // #[allow(clippy::unused_self)]
    // pub fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
    //     // eprintln!("About to try_lock registry to get task memory usage");
    //     REGISTRY.try_lock().map_or(None, |registry| {
    //         registry
    //             .task_allocations
    //             .get(&task_id)
    //             .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
    //     })
    // }

    // #[allow(clippy::unused_self)]
    // pub fn enter_task(&self, task_id: usize) {
    //     // eprintln!("Entering task {}", task_id);
    //     let thread_id = thread::current().id();

    //     push_task_to_stack(thread_id, task_id);
    // }

    // #[allow(clippy::unused_self)]
    // pub fn exit_task(&self, task_id: usize) {
    //     // eprintln!("Exiting task {}", task_id);
    //     let thread_id = thread::current().id();

    //     pop_task_from_stack(thread_id, task_id);
    // }
}

#[cfg(feature = "full_profiling")]
pub fn get_with_system_alloc(closure: impl FnMut()) {
    MultiAllocator::with(AllocatorTag::System, closure);
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        #[cfg(feature = "full_profiling")]
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
                let mut task_id = 0;
                MultiAllocator::with(AllocatorTag::System, || {
                    // Now we can safely use backtrace without recursion!
                    // let start_pattern = "TaskAwareAllocator";
                    let start_pattern = "Profile::new";

                    // eprintln!("Calling extract_callstack");
                    let mut current_backtrace = Backtrace::new_unresolved();
                    let (cleaned_stack, in_backtrace_new) =
                        extract_callstack_from_backtrace(start_pattern, &mut current_backtrace);
                    if in_backtrace_new {
                        // eprintln!("Ignoring allocation request for new backtrace");
                        return;
                    } else {
                        current_backtrace.resolve();
                        // if Backtrace::frames(&current_backtrace)
                        //     .iter()
                        //     .flat_map(backtrace::BacktraceFrame::symbols)
                        //     .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                        //     .filter(|frame| frame.contains("Backtrace::new"))
                        //     .count()
                        //     > 1
                        // {
                        //     eprintln!(
                        //         "Failed to pick up backtrace with multiple Backtrace::new calls"
                        //     );
                        //     Backtrace::frames(&current_backtrace)
                        //         .iter()
                        //         .flat_map(backtrace::BacktraceFrame::symbols)
                        //         .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                        //         .for_each(|frame| {
                        //             eprintln!("frame: {}", frame);
                        //         });
                        // } else {
                        //     eprintln!("All good!");
                        //     // Backtrace::frames(&current_backtrace)
                        //     //     .iter()
                        //     //     .flat_map(backtrace::BacktraceFrame::symbols)
                        //     //     .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                        //     //     .for_each(|frame| {
                        //     //         eprintln!("frame: {}", frame);
                        //     //     });
                        // }
                    }

                    if cleaned_stack.is_empty() {
                        // eprintln!(
                        //     "Empty cleaned_stack for backtrace\n{:#?}",
                        //     backtrace::Backtrace::new()
                        // );
                        // eprintln!("Empty cleaned_stack");
                        task_id = get_last_active_task().unwrap_or(0);
                    } else {
                        // Make sure the use of a separate allocator is working.
                        assert!(!cleaned_stack
                            .iter()
                            .any(|frame| frame.contains("find_matching_profile")));

                        // eprintln!("Calling extract_path");
                        let path = extract_path(&cleaned_stack);
                        if path.is_empty() {
                            if Backtrace::frames(&current_backtrace)
                                .iter()
                                .flat_map(backtrace::BacktraceFrame::symbols)
                                .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                                .any(|frame| frame.contains("Backtrace::new"))
                            {
                                eprintln!("Ignoring setup allocation of size {size} containing Backtrace::new");
                                // Don't record the allocation because it's profiling setup
                                Backtrace::frames(&current_backtrace)
                                    .iter()
                                    .flat_map(backtrace::BacktraceFrame::symbols)
                                    .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                                    .for_each(|frame| {
                                        eprintln!("frame: {}", frame);
                                    });
                                return;
                            }
                            eprintln!(
                                "...path is empty for thread {:?}: assigning to lastest active task. cleaned_stack: {:?}",
                                thread::current().id(),
                                cleaned_stack
                            );
                            eprintln!("...backtrace: {current_backtrace:?}",);
                            task_id = get_last_active_task().unwrap_or(0);
                        } else {
                            // eprintln!("path={path:#?}");

                            task_id = find_matching_profile(&path);
                        }
                    }
                });

                // Record allocation if task found
                // Use okaoka to avoid recursive allocations
                MultiAllocator::with(AllocatorTag::System, || {
                    if task_id > 0 {
                        let address = ptr as usize;
                        // let size = layout.size();

                        // Record in thread-local buffer
                        record_allocation(task_id, address, size);
                    }
                });
            } else {
                // eprintln!("ignoring allocation of {} bytes", layout.size());
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
                // Setup guard
                struct Guard;
                impl Drop for Guard {
                    fn drop(&mut self) {
                        IN_TRACKING.with(|flag| *flag.borrow_mut() = false);
                    }
                }
                let _guard = Guard;

                // Record deallocation
                // Use okaoka to avoid recursive allocations
                MultiAllocator::with(AllocatorTag::System, || {
                    let address = ptr as usize;

                    // Record in thread-local buffer
                    record_deallocation(address);
                });
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
        get_task_memory_usage(self.task_id)
    }
}

// Provide a dummy TaskMemoryContext type for when full_profiling is disabled
#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskMemoryContext;

// /// Creates a standalone memory guard that activates the given task ID
// ///
// /// # Errors
// ///
// /// This function will bubble up any error from `TaskAwareAllocator::enter_task`.
// #[cfg(feature = "full_profiling")]
// pub fn create_memory_guard(task_id: usize) -> Result<TaskGuard, String> {
//     // Get the allocator
//     let allocator = get_allocator();

//     // Enter the task (now thread-aware)
//     allocator.enter_task(task_id);

//     // Create a guard that's tied to this thread and task
//     let task_guard = TaskGuard::new(task_id);
//     println!(
//         "GUARD CREATED: Task {} on thread {:?}",
//         task_id,
//         thread::current().id()
//     );
//     Ok(task_guard)
// }

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
        MultiAllocator::with(AllocatorTag::System, || {
            // Process pending allocations before removing the task
            process_pending_allocations();

            // Remove from active profiles
            deactivate_task(self.task_id);

            // Remove from thread stack
            pop_task_from_stack(thread::current().id(), self.task_id);

            // Remove from task path registry
            remove_task_path(self.task_id);
        });
    }
}

#[cfg(feature = "full_profiling")]
pub fn run_with_system_alloc(closure: impl Fn()) {
    MultiAllocator::with(AllocatorTag::System, closure);
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
pub static TASK_PATH_REGISTRY: LazyLock<Mutex<BTreeMap<usize, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

// // 2. Function to add a task's path to the TASK_PATH_REGISTRY
// #[cfg(feature = "full_profiling")]
// pub fn register_task_path(task_id: usize, path: Vec<String>) {
//     // eprintln!("About to try_lock TASK_PATH_REGISTRY for register_task_path");
//     if let Ok(mut registry) = TASK_PATH_REGISTRY.try_lock() {
//         registry.insert(task_id, path);
//     } else {
//         eprintln!(
//             "Failed to lock task path registry to registertask {}",
//             task_id
//         );
//     }
// }

// 3. Function to look up a task's path by ID
#[cfg(feature = "full_profiling")]
pub fn lookup_task_path(task_id: usize) -> Option<Vec<String>> {
    // eprintln!("About to try_lock TASK_PATH_REGISTRY for lookup_task_path");
    let registry = TASK_PATH_REGISTRY.lock();
    registry.get(&task_id).cloned()
}

// 4. Function to dump the entire registry
#[allow(dead_code)]
#[cfg(feature = "full_profiling")]
pub fn dump_task_path_registry() {
    // eprintln!("About to try_lock TASK_PATH_REGISTRY for dump_task_path_registry");
    let registry = TASK_PATH_REGISTRY.lock();
    println!("==== TASK PATH REGISTRY DUMP ====");
    println!("Total registered tasks: {}", registry.len());

    for (task_id, path) in registry.iter() {
        println!("Task {}: {}", task_id, path.join("::"));
    }
    drop(registry);
    println!("=================================");
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
    let mut registry = TASK_PATH_REGISTRY.lock();
    registry.remove(&task_id);
}

// Helper function to find the best matching profile
#[cfg(feature = "full_profiling")]
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
    // if best_score == path.len() {
    //     eprintln!("...returning exact match with 100% score of {}", score);
    // } else {
    //     eprintln!(
    //         "...returning best-match ancestor with score of {} vs path.len() = {}",
    //         best_score,
    //         path.len()
    //     );
    //     // println!("==== TASK PATH REGISTRY DUMP ====");
    //     // println!("Total registered tasks: {}", path_registry.len());

    //     // for (task_id, path) in path_registry.iter() {
    //     //     println!("Task {}: {}", task_id, path.join(" -> "));
    //     // }
    //     // println!("=================================");

    //     // println!("Active tasks={:#?}", get_active_tasks());
    // }

    // Return the best match if found, otherwise fall back to last active task
    if best_match > 0 {
        return best_match;
    }

    // Fallback: Return the most recently activated profile
    eprintln!("...returning fallback: most recently activated profile - for path: {path:?}");
    get_last_active_task().unwrap_or(0)
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
//     // if let Ok(mut registry) = REGISTRY.try_lock() {
//     //     registry.active_profiles.remove(&task_id);
//     // } else {
//     //     eprintln!("Failed to lock registry activate profile: {}", task_id);
//     // }
//     deactivate_task(task_id);
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
/// This is called by the main `init_profiling` function.
#[cfg(feature = "full_profiling")]
pub fn initialize_memory_profiling() {
    // This is called at application startup to set up memory profiling
    MultiAllocator::with(AllocatorTag::System, || {
        println!("Memory profiling initialized");
    });
}

/// Finalize memory profiling and write out data.
/// This is called by the main `finalize_profiling` function.
#[cfg(feature = "full_profiling")]
pub fn finalize_memory_profiling() {
    MultiAllocator::with(AllocatorTag::System, || {
        // Process any pending allocations
        process_pending_allocations();

        // Write memory profile data
        write_memory_profile_data();
    });
}

/// Write memory profile data to a file
#[cfg(feature = "full_profiling")]
fn write_memory_profile_data() {
    use std::{fs::File, io::BufWriter};

    use crate::profiling::get_memory_path;

    MultiAllocator::with(AllocatorTag::System, || {
        // Retrieve registries to get task allocations and names
        // Open memory.folded file
        if let Ok(file) = File::create(get_memory_path().unwrap_or("memory.folded")) {
            let mut writer = BufWriter::new(file);

            let task_allocs = { ALLOC_REGISTRY.lock().task_allocations.clone() };
            let task_ids = { task_allocs.keys().copied().collect::<Vec<_>>() };

            let candidates: Vec<(usize, Vec<String>)> = {
                let binding = TASK_PATH_REGISTRY.lock();
                binding
                    .iter()
                    .filter(|(task_id, _pat)| task_ids.contains(task_id))
                    .map(|(task_id, pat)| (*task_id, pat.clone()))
                    .collect()
            };

            // Write profile data
            for (task_id, allocations) in &task_allocs {
                // Skip tasks with no allocations
                if allocations.is_empty() {
                    continue;
                }

                // Get the path for this task
                if let Some((_, path)) = &candidates.get(*task_id) {
                    let path_str = path.join(";");
                    let total_bytes: usize = allocations.iter().map(|(_, size)| *size).sum();

                    // Write line to folded format file
                    let _ = writeln!(writer, "{} {}", path_str, total_bytes);
                }
            }
        }
    });
}
