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
        dbg!();
        let task_id = self.next_task_id.fetch_add(1, Ordering::SeqCst);
        dbg!();

        // // Initialize tracking for this task
        // if let Ok(mut registry) = REGISTRY.lock() {
        //     registry.task_allocations.insert(task_id, Vec::new());
        // }

        // Create Vec outside the lock scope
        let empty_vec = Vec::new();

        // Temporarily disable allocation tracking to prevent recursion
        INSIDE_ALLOCATION.with(|flag| *flag.borrow_mut() = true);

        // Now lock and modify registry
        if let Ok(mut registry) = REGISTRY.lock() {
            registry.task_allocations.insert(task_id, empty_vec);
        }

        // Re-enable allocation tracking
        INSIDE_ALLOCATION.with(|flag| *flag.borrow_mut() = false);

        dbg!();

        TaskMemoryContext {
            task_id,
            allocator: self,
        }
    }

    pub fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        dbg!();
        match REGISTRY.lock() {
            Ok(registry) => {
                dbg!();
                registry
                    .task_allocations
                    .get(&task_id)
                    .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
            }
            Err(_) => None,
        }
    }

    pub fn enter_task(&self, task_id: usize) -> Result<(), String> {
        dbg!();
        match REGISTRY.lock() {
            Ok(mut registry) => {
                dbg!();
                if registry.current_task_id.is_some() {
                    dbg!();
                    // Already in a task context
                    eprintln!(
                        "Already in a task context for task {:?}",
                        registry.current_task_id
                    );
                    return Err("Already in a task context".to_string());
                }

                registry.current_task_id = Some(task_id);
                dbg!();
                Ok(())
            }
            Err(_) => Err("Failed to lock registry".to_string()),
        }
    }

    pub fn exit_task(&self) -> Result<(), String> {
        dbg!();
        match REGISTRY.lock() {
            Ok(mut registry) => {
                if registry.current_task_id.is_none() {
                    // Not in a task context
                    return Err("Not in a task context".to_string());
                }

                registry.current_task_id = None;
                dbg!();
                Ok(())
            }
            Err(_) => Err("Failed to lock registry".to_string()),
        }
    }

    fn record_alloc(&self, ptr: *mut u8, layout: Layout) {
        dbg!();
        if let Ok(mut registry) = REGISTRY.lock() {
            dbg!();
            if let Some(task_id) = registry.current_task_id {
                let address = ptr as usize;
                let size = layout.size();

                // Record in task's allocation list
                if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                    allocations.push((address, size));
                }
                dbg!();

                // Map address back to task for deallocation
                registry.address_to_task.insert(address, task_id);
            }
        }
    }

    fn record_dealloc(&self, ptr: *mut u8) {
        dbg!();
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
            dbg!();
        }
        dbg!();
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
    static INSIDE_ALLOCATION: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        #[cfg(feature = "full_profiling")]
        if !ptr.is_null() {
            // Check if we're already tracking an allocation to prevent recursion
            let should_record = INSIDE_ALLOCATION.with(|inside| {
                if *inside.borrow() {
                    false // Already tracking, skip to prevent recursion
                } else {
                    *inside.borrow_mut() = true; // Mark that we're tracking
                    true
                }
            });

            if should_record {
                // Record the allocation
                if let Ok(mut registry) = REGISTRY.lock() {
                    if let Some(task_id) = registry.current_task_id {
                        let address = ptr as usize;
                        let size = layout.size();

                        // Record in task's allocation list
                        if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                            allocations.push((address, size));
                        }

                        // Map address back to task for deallocation
                        registry.address_to_task.insert(address, task_id);
                    }
                }

                // Reset the tracking flag
                INSIDE_ALLOCATION.with(|inside| {
                    *inside.borrow_mut() = false;
                });
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        #[cfg(feature = "full_profiling")]
        if !ptr.is_null() {
            // Similar guard for deallocation
            let should_record = INSIDE_ALLOCATION.with(|inside| {
                if *inside.borrow() {
                    false
                } else {
                    *inside.borrow_mut() = true;
                    true
                }
            });

            if should_record {
                // Dealloc using REGISTRY directly
                if let Ok(mut registry) = REGISTRY.lock() {
                    let address = ptr as usize;

                    if let Some(task_id) = registry.address_to_task.remove(&address) {
                        // Remove from task's allocation list
                        if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                            if let Some(index) =
                                allocations.iter().position(|(addr, _)| *addr == address)
                            {
                                allocations.swap_remove(index);
                            }
                        }
                    }
                }

                // Reset the tracking flag
                INSIDE_ALLOCATION.with(|inside| {
                    *inside.borrow_mut() = false;
                });
            }
        }

        self.inner.dealloc(ptr, layout);
    }
}

#[cfg(feature = "full_profiling")]
impl TaskMemoryContext {
    /// Activates this task context for memory tracking
    pub fn enter(&self) -> Result<TaskGuard, String> {
        match self.allocator.enter_task(self.task_id) {
            Ok(()) => Ok(TaskGuard::new(self.task_id, self.allocator)),
            Err(e) => Err(e),
        }
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
pub fn create_memory_guard(task_id: usize) -> Result<TaskGuard<'static>, String> {
    dbg!();
    // Get the allocator
    let allocator = get_allocator();

    dbg!();

    // Enter the task
    match allocator.enter_task(task_id) {
        Ok(()) => {
            dbg!();
            // Create a guard that's tied to the allocator directly,
            // not to a specific TaskMemoryContext
            let task_guard = TaskGuard::new(task_id, allocator);
            dbg!();
            Ok(task_guard)
        }
        Err(e) => Err(e),
    }
}

/// RAII guard for task context
#[cfg(feature = "full_profiling")]
#[derive(Debug)]
pub struct TaskGuard<'a> {
    task_id: usize,
    allocator: &'a TaskAwareAllocator<System>,
}

#[cfg(feature = "full_profiling")]
impl<'a> TaskGuard<'a> {
    pub const fn new(task_id: usize, allocator: &'a TaskAwareAllocator<System>) -> Self {
        Self { task_id, allocator }
    }
}

#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskGuard;

#[cfg(feature = "full_profiling")]
impl Drop for TaskGuard<'_> {
    fn drop(&mut self) {
        let _ = self.allocator.exit_task();
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
    dbg!();
    &ALLOCATOR
}

/// Creates a new task context for memory tracking.
#[cfg(feature = "full_profiling")]
pub fn create_memory_task() -> TaskMemoryContext {
    dbg!();
    let allocator = get_allocator();
    dbg!();
    allocator.create_task_context()
}

/// No-op version when profiling is disabled.
#[cfg(not(feature = "full_profiling"))]
pub fn create_memory_task() -> TaskMemoryContext {
    TaskMemoryContext
}
