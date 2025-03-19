#![allow(clippy::uninlined_format_args)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling.

use std::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "full_profiling")]
use std::{
    alloc::System,
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock, Mutex,
    },
    thread::{self, ThreadId},
};

#[derive(Debug)]
#[cfg(feature = "full_profiling")]
struct AllocationRegistry {
    // Thread ID -> Stack of active task IDs (most recent on top)
    thread_task_stacks: HashMap<ThreadId, Vec<usize>>,

    // Task ID -> Allocations mapping
    task_allocations: HashMap<usize, Vec<(usize, usize)>>,

    // Address -> Task ID mapping for deallocations
    address_to_task: HashMap<usize, usize>,
}

#[cfg(feature = "full_profiling")]
static REGISTRY: LazyLock<Mutex<AllocationRegistry>> = LazyLock::new(|| {
    Mutex::new(AllocationRegistry {
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
        REGISTRY.lock().map_or(None, |registry| {
            registry
                .task_allocations
                .get(&task_id)
                .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
        })
    }

    pub fn enter_task(&self, task_id: usize) -> Result<(), String> {
        let thread_id = thread::current().id();

        REGISTRY.lock().map_or_else(
            |_| Err("Failed to lock registry".to_string()),
            |mut registry| {
                // Get or create task stack for this thread
                let task_stack = registry.thread_task_stacks.entry(thread_id).or_default();

                // Push this task onto the stack
                task_stack.push(task_id);

                // Initialize allocation tracking if needed
                registry.task_allocations.entry(task_id).or_default();

                Ok(())
            },
        )
    }

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

                        // If stack is now empty, remove it
                        if task_stack.is_empty() {
                            registry.thread_task_stacks.remove(&thread_id);
                        }

                        return Ok(());
                    }
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
                let _guard = scopeguard::guard((), |()| {
                    TRACKING_ALLOCATION.with(|tracking| {
                        *tracking.borrow_mut() = false;
                    });
                });

                // Record allocation
                let address = ptr as usize;
                let size = layout.size();
                let thread_id = thread::current().id();

                // Get current thread's active task stack
                if let Ok(mut registry) = REGISTRY.try_lock() {
                    if let Some(task_stack) = registry.thread_task_stacks.get(&thread_id) {
                        // Attribute to the topmost task on the stack
                        if let Some(&top_task_id) = task_stack.last() {
                            // Record allocation for this task
                            registry
                                .task_allocations
                                .entry(top_task_id)
                                .or_insert_with(Vec::new)
                                .push((address, size));

                            // Map address to task for deallocation
                            registry.address_to_task.insert(address, top_task_id);
                        }
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
    allocations: Vec<(usize, usize)>,
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
        // Only pop our task from our thread's stack
        let thread_id = thread::current().id();

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
