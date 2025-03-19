#![allow(clippy::uninlined_format_args)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling.

use std::alloc::System;
use std::alloc::{GlobalAlloc, Layout};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

#[cfg(feature = "full_profiling")]
use std::sync::LazyLock;

/// A thread-safe storage for task-specific allocation tracking
use std::thread::{self, ThreadId};

#[derive(Debug)]
struct AllocationRegistry {
    // Thread ID -> Task ID mapping
    thread_tasks: HashMap<ThreadId, usize>,

    // Task ID -> Allocations mapping
    task_allocations: HashMap<usize, Vec<(usize, usize)>>,

    // Address -> Task ID mapping for deallocations
    address_to_task: HashMap<usize, usize>,
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
    allocator: &'static TaskAwareAllocator<System>,
}

#[cfg(feature = "full_profiling")]
static REGISTRY: LazyLock<Mutex<AllocationRegistry>> = LazyLock::new(|| {
    Mutex::new(AllocationRegistry {
        thread_tasks: HashMap::new(),
        task_allocations: HashMap::new(),
        address_to_task: HashMap::new(),
    })
});

// #[cfg(feature = "full_profiling")]
// thread_local! {
//     // Track whether we're currently holding the registry lock
//     static HOLDING_REGISTRY_LOCK: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
// }

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
                    allocations: Vec::new(),
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

    pub fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        if let Ok(registry) = REGISTRY.lock() {
            registry
                .task_allocations
                .get(&task_id)
                .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
        } else {
            None
        }
    }

    pub fn enter_task(&self, task_id: usize) -> Result<(), String> {
        let thread_id = thread::current().id();

        if let Ok(mut registry) = REGISTRY.lock() {
            // Check if this thread already has an active task
            if registry.thread_tasks.contains_key(&thread_id) {
                return Err(format!("Thread {:?} already has an active task", thread_id));
            }

            // Activate task for this thread
            registry.thread_tasks.insert(thread_id, task_id);
            Ok(())
        } else {
            Err("Failed to lock registry".to_string())
        }
    }

    pub fn exit_task(&self, task_id: usize) -> Result<(), String> {
        let thread_id = thread::current().id();

        if let Ok(mut registry) = REGISTRY.lock() {
            // Only remove if the thread is running the specified task
            if let Some(&active_id) = registry.thread_tasks.get(&thread_id) {
                if active_id == task_id {
                    registry.thread_tasks.remove(&thread_id);
                    return Ok(());
                }
                return Err(format!(
                    "Thread {:?} is running task {} not {}",
                    thread_id, active_id, task_id
                ));
            }
            Err(format!("Thread {:?} has no active task", thread_id))
        } else {
            Err("Failed to lock registry".to_string())
        }
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
}

// // Implement the GlobalAlloc trait for both cases
// #[cfg(feature = "full_profiling")]
// thread_local! {
//     // Track whether we're currently inside an allocation operation
//     static INSIDE_ALLOCATION: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
// }

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
                let _guard = scopeguard::guard((), |()| {
                    TRACKING_ALLOCATION.with(|tracking| {
                        *tracking.borrow_mut() = false;
                    });
                });

                // Record allocation
                let address = ptr as usize;
                let size = layout.size();
                let thread_id = thread::current().id();

                // Get current thread's task ID from registry
                if let Ok(mut registry) = REGISTRY.try_lock() {
                    if let Some(&task_id) = registry.thread_tasks.get(&thread_id) {
                        // Record in task's allocation list
                        registry
                            .task_allocations
                            .entry(task_id)
                            .or_insert_with(Vec::new)
                            .push((address, size));

                        // Map address to task
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
                        // Remove from task's allocation list
                        if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                            if let Some(pos) =
                                allocations.iter().position(|(addr, _)| *addr == address)
                            {
                                allocations.swap_remove(pos);
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
        let allocator = get_allocator();
        allocator.get_task_memory_usage(self.task_id)
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
    // println!("Initializing TASK_STATE with next_task_id = 1");
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
        // Only deactivate this specific task on this thread
        let thread_id = thread::current().id();

        if let Ok(mut registry) = REGISTRY.lock() {
            // Only remove if it matches our task on our thread
            if let Some(&active_id) = registry.thread_tasks.get(&thread_id) {
                if active_id == self.task_id {
                    registry.thread_tasks.remove(&thread_id);
                }
            }
        }

        // Also update the task's active status
        if let Ok(mut task_map) = TASK_STATE.task_map.lock() {
            if let Some(data) = task_map.get_mut(&self.task_id) {
                data.active = false;
            }
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

/// No-op version when profiling is disabled.
#[cfg(not(feature = "full_profiling"))]
pub fn create_memory_task() -> TaskMemoryContext {
    TaskMemoryContext
}
