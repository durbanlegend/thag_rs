//\! Task-aware memory allocator for profiling.
//\!
//\! This module provides a memory allocator that tracks allocations by logical tasks
//\! rather than threads, making it suitable for async code profiling.

use std::alloc::System;
use std::alloc::{GlobalAlloc, Layout};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

#[cfg(feature = "full_profiling")]
use std::sync::LazyLock;

// Disable debug prints that were causing stack overflow
#[macro_export]
macro_rules! dbg_removed {
    () => {};
    ($val:expr $(,)?) => {
        { $val }
    };
    ($($val:expr),+ $(,)?) => {
        ($($val),+)
    };
}

/// A thread-safe storage for task-specific allocation tracking
#[derive(Debug)]
struct AllocationRegistry {
    /// Maps task IDs to their allocated memory blocks
    task_allocations: HashMap<usize, Vec<(usize, usize)>>, // task_id -> [(address, size)]

    /// Maps memory addresses to their owning task ID
    address_to_task: HashMap<usize, usize>, // address -> task_id

    /// Current active task ID (if any)
    current_task_id: Option<usize>,
}

/// Task-aware allocator that tracks memory usage per task ID
#[derive(Debug)]
pub struct TaskAwareAllocator<A: GlobalAlloc> {
    /// The inner allocator that actually performs allocation
    inner: A,

    /// Counter for generating unique task IDs
    next_task_id: AtomicUsize,
}

/// Task context for tracking allocations
#[cfg(feature = "full_profiling")]
#[derive(Debug, Clone)]
pub struct TaskMemoryContext {
    task_id: usize,
    allocator: &'static TaskAwareAllocator<System>,
}

#[cfg(feature = "full_profiling")]
static REGISTRY: LazyLock<Mutex<AllocationRegistry>> = LazyLock::new(|| {
    Mutex::new(AllocationRegistry {
        task_allocations: HashMap::new(),
        address_to_task: HashMap::new(),
        current_task_id: None,
    })
});

#[cfg(feature = "full_profiling")]
thread_local! {
    // Track whether we're currently holding the registry lock
    static HOLDING_REGISTRY_LOCK: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

// Define registry-specific methods for System allocator
#[cfg(feature = "full_profiling")]
impl TaskAwareAllocator<System> {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);
        println!("Creating task context for task {}", task_id);

        // Initialize task data
        if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
            task_map.insert(
                task_id,
                TaskData {
                    allocations: Vec::new(),
                    active: false,
                },
            );
            println!("Initialized task data for task {}, active=false", task_id);
        } else {
            println!("Failed to lock task_map when creating task {}", task_id);
        }

        TaskMemoryContext {
            task_id,
            allocator: self,
        }
    }

    pub fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        println!("Getting memory usage for task {}", task_id);
        let result = if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
            if let Some(data) = task_map.get(&task_id) {
                let total = data.allocations.iter().map(|(_, size)| *size).sum();
                println!(
                    "Task {} has {} allocations totaling {} bytes",
                    task_id,
                    data.allocations.len(),
                    total
                );
                Some(total)
            } else {
                println!("Task {} not found in task_map", task_id);
                None
            }
        } else {
            println!("Failed to lock task_map when querying task {}", task_id);
            None
        };
        result
    }

    pub fn enter_task(&self, task_id: usize) -> Result<(), String> {
        dbg_removed!();
        match REGISTRY.lock() {
            Ok(mut registry) => {
                dbg_removed!();
                if registry.current_task_id.is_some() {
                    dbg_removed!();
                    // Already in a task context
                    eprintln!(
                        "Already in a task context for task {:?}",
                        registry.current_task_id
                    );
                    return Err("Already in a task context".to_string());
                }

                registry.current_task_id = Some(task_id);
                dbg_removed!();
                Ok(())
            }
            Err(_) => Err("Failed to lock registry".to_string()),
        }
    }

    pub fn exit_task(&self) -> Result<(), String> {
        dbg_removed!();
        match REGISTRY.lock() {
            Ok(mut registry) => {
                if registry.current_task_id.is_none() {
                    // Not in a task context
                    return Err("Not in a task context".to_string());
                }

                registry.current_task_id = None;
                dbg_removed!();
                Ok(())
            }
            Err(_) => Err("Failed to lock registry".to_string()),
        }
    }

    fn record_alloc(&self, ptr: *mut u8, layout: Layout) {
        dbg_removed!();
        if let Ok(mut registry) = REGISTRY.lock() {
            dbg_removed!();
            if let Some(task_id) = registry.current_task_id {
                let address = ptr as usize;
                let size = layout.size();

                // Record in task's allocation list
                if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                    allocations.push((address, size));
                }
                dbg_removed!();

                // Map address back to task for deallocation
                registry.address_to_task.insert(address, task_id);
            }
        }
    }

    fn record_dealloc(&self, ptr: *mut u8) {
        dbg_removed!();
        if let Ok(mut registry) = REGISTRY.lock() {
            let address = ptr as usize;

            if let Some(task_id) = registry.address_to_task.remove(&address) {
                // Remove from task's allocation list
                if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                    if let Some(index) = allocations.iter().position(|(addr, _)| *addr == address) {
                        allocations.swap_remove(index);
                    }
                }
            }
            dbg_removed!();
        }
        dbg_removed!();
    }
}

// Provide non-functional implementations for the generic case
#[cfg(not(feature = "full_profiling"))]
impl<A: GlobalAlloc> TaskAwareAllocator<A> {
    pub fn get_task_memory_usage(&self, _task_id: usize) -> Option<usize> {
        None
    }

    pub fn enter_task(&self, _task_id: usize) -> Result<(), String> {
        Ok(())
    }

    pub fn exit_task(&self) -> Result<(), String> {
        Ok(())
    }

    fn record_alloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op when profiling is disabled
    }

    fn record_dealloc(&self, _ptr: *mut u8) {
        // No-op when profiling is disabled
    }
}

// Implement the GlobalAlloc trait for both cases
#[cfg(feature = "full_profiling")]
thread_local! {
    // Track whether we're currently inside an allocation operation
    static INSIDE_ALLOCATION: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        #[cfg(feature = "full_profiling")]
        if !ptr.is_null() {
            // Prevent recursion during allocation tracking
            thread_local! {
                static TRACKING_ALLOCATION: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
            }

            let should_track = TRACKING_ALLOCATION.with(|tracking| {
                // If we're already tracking an allocation, don't track this one to prevent recursion
                if *tracking.borrow() {
                    false
                } else {
                    *tracking.borrow_mut() = true;
                    true
                }
            });

            if should_track {
                // Execute tracking code and reset flag afterward
                let _guard = scopeguard::guard((), |_| {
                    TRACKING_ALLOCATION.with(|tracking| {
                        *tracking.borrow_mut() = false;
                    });
                });

                // Record allocation
                let address = ptr as usize;
                let size = layout.size();

                // Check if we have an active task in the registry
                if let Ok(registry) = REGISTRY.try_lock() {
                    if let Some(task_id) = registry.current_task_id {
                        println!(
                            "Found active task {} in registry when allocating {} bytes",
                            task_id, size
                        );

                        // Record in task map
                        if let Ok(mut task_map) = TASK_STATE.task_map.try_lock() {
                            if let Some(data) = task_map.get_mut(&task_id) {
                                // Record allocation for this task
                                data.allocations.push((address, size));
                                println!(
                                    "Recorded allocation of {} bytes for task {}",
                                    size, task_id
                                );

                                // Map address to task
                                if let Ok(mut addr_map) = ADDRESS_MAP.try_lock() {
                                    addr_map.insert(address, task_id);
                                }
                            } else {
                                println!("Task {} not found in task_map when allocating", task_id);
                            }
                        }
                    } else {
                        println!("No active task in registry when allocating {} bytes", size);
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
                static TRACKING_DEALLOCATION: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
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
                let _guard = scopeguard::guard((), |_| {
                    TRACKING_DEALLOCATION.with(|tracking| {
                        *tracking.borrow_mut() = false;
                    });
                });

                let address = ptr as usize;

                // Find which task owns this allocation
                if let Ok(mut addr_map) = ADDRESS_MAP.try_lock() {
                    if let Some(task_id) = addr_map.remove(&address) {
                        // Remove from task's allocation list
                        if let Ok(mut task_map) = TASK_STATE.task_map.try_lock() {
                            if let Some(data) = task_map.get_mut(&task_id) {
                                if let Some(index) = data
                                    .allocations
                                    .iter()
                                    .position(|(addr, _)| *addr == address)
                                {
                                    data.allocations.swap_remove(index);
                                }
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
    /// Activates this task context for memory tracking
    pub fn enter(&self) -> Result<TaskGuard, String> {
        // Mark task as active
        if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
            if let Some(data) = task_map.get_mut(&self.task_id) {
                if data.active {
                    println!("Task {} is already active", self.task_id);
                    return Err("Task is already active".to_string());
                }
                data.active = true;
                println!("Task {} marked as active", self.task_id);
                return Ok(TaskGuard {
                    task_id: self.task_id,
                });
            } else {
                println!("Task {} not found in task_map", self.task_id);
            }
        } else {
            println!("Failed to lock task_map");
        }
        Err("Task not found".to_string())
    }

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
    // Reset any lingering task contexts first
    reset_task_registry();

    // Get the allocator
    let allocator = get_allocator();

    // Enter the task in the registry
    match allocator.enter_task(task_id) {
        Ok(()) => {
            // Also mark task as active in task map
            if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
                if let Some(data) = task_map.get_mut(&task_id) {
                    data.active = true;
                    println!("Task {} marked as active in task_map", task_id);
                } else {
                    println!(
                        "Warning: Task {} not found in task_map when creating guard",
                        task_id
                    );
                }
            }

            // Create a guard that's tied to the allocator directly
            let task_guard = TaskGuard::new(task_id);
            println!("Successfully created TaskGuard for task {}", task_id);
            Ok(task_guard)
        }
        Err(e) => Err(e),
    }
}

// Task tracking state
struct TaskState {
    // Maps task IDs to their tracking state
    task_map: Mutex<HashMap<usize, TaskData>>,
    // Counter for generating task IDs
    next_task_id: AtomicUsize,
}

// Per-task data
struct TaskData {
    allocations: Vec<(usize, usize)>,
    active: bool,
}

// Global task state
static TASK_STATE: LazyLock<TaskState> = LazyLock::new(|| {
    println!("Initializing TASK_STATE with next_task_id = 1");
    TaskState {
        task_map: Mutex::new(HashMap::new()),
        next_task_id: AtomicUsize::new(1),
    }
});

// Global mapping of addresses to task IDs
static ADDRESS_MAP: LazyLock<Mutex<HashMap<usize, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
        println!("Dropping TaskGuard for task {}", self.task_id);
        // Always reset the registry to ensure we don't leave lingering tasks
        reset_task_registry();

        // Also mark task as inactive in the task map
        if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
            if let Some(data) = task_map.get_mut(&self.task_id) {
                data.active = false;
                println!("TaskGuard: Task {} marked as inactive", self.task_id);
            } else {
                println!("TaskGuard: Task {} not found in task_map", self.task_id);
            }
        } else {
            println!("TaskGuard: Failed to lock task_map");
        }
    }
}

// Implement basic methods for the no-op TaskMemoryContext
#[cfg(not(feature = "full_profiling"))]
impl TaskMemoryContext {
    /// No-op implementation that returns a dummy guard
    #[must_use]
    pub fn enter(&self) -> Result<TaskGuard, String> {
        Ok(TaskGuard)
    }

    /// Returns a dummy ID (0) when profiling is disabled
    #[must_use]
    pub const fn id(&self) -> usize {
        0
    }

    /// Always returns None when profiling is disabled
    #[must_use]
    pub const fn memory_usage(&self) -> Option<usize> {
        None
    }
}

#[cfg(feature = "full_profiling")]
#[global_allocator]
static ALLOCATOR: TaskAwareAllocator<System> = TaskAwareAllocator {
    inner: System,
    next_task_id: AtomicUsize::new(1),
};

// Helper to get the allocator instance
#[cfg(feature = "full_profiling")]
pub fn get_allocator() -> &'static TaskAwareAllocator<System> {
    dbg_removed!();
    &ALLOCATOR
}

/// Creates a new task context for memory tracking.
#[cfg(feature = "full_profiling")]
pub fn create_memory_task() -> TaskMemoryContext {
    dbg_removed!();
    let allocator = get_allocator();
    dbg_removed!();
    allocator.create_task_context()
}

/// No-op version when profiling is disabled.
#[cfg(not(feature = "full_profiling"))]
pub fn create_memory_task() -> TaskMemoryContext {
    TaskMemoryContext
}

#[cfg(feature = "full_profiling")]
pub fn reset_task_registry() {
    // Clear any existing task contexts
    if let Ok(mut registry) = REGISTRY.lock() {
        println!(
            "Resetting task registry, was: {:?}",
            registry.current_task_id
        );
        registry.current_task_id = None;
    } else {
        println!("Failed to lock registry for reset");
    }
}
