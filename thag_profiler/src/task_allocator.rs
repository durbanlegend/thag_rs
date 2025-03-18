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

    /// Registry of allocations organized by task
    registry: Mutex<AllocationRegistry>,
}

/// Task context for tracking allocations
#[cfg(feature = "full_profiling")]
#[derive(Debug, Clone)]
pub struct TaskMemoryContext {
    task_id: usize,
    allocator: &'static TaskAwareAllocator<System>,
}

impl<A: GlobalAlloc> TaskAwareAllocator<A> {
    /// Creates a new task-aware allocator
    pub fn new(inner: A) -> Self {
        Self {
            inner,
            next_task_id: AtomicUsize::new(1), // Start at 1, reserve 0 for "untracked"
            registry: Mutex::new(AllocationRegistry {
                task_allocations: HashMap::new(),
                address_to_task: HashMap::new(),
                current_task_id: None,
            }),
        }
    }
}

// Implement specific methods for the System allocator version
#[cfg(feature = "full_profiling")]
impl TaskAwareAllocator<System> {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = self.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize tracking for this task
        let mut registry = self.registry.lock().unwrap();
        registry.task_allocations.insert(task_id, Vec::new());

        TaskMemoryContext {
            task_id,
            allocator: self,
        }
    }
}

impl<A: GlobalAlloc> TaskAwareAllocator<A> {
    /// Get memory usage statistics for a specific task
    pub fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        let registry = self.registry.lock().unwrap();

        registry
            .task_allocations
            .get(&task_id)
            .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
    }

    /// Enter a task context - all allocations will be attributed to this task
    pub fn enter_task(&self, task_id: usize) -> Result<(), String> {
        let mut registry = self.registry.lock().unwrap();

        if registry.current_task_id.is_some() {
            // Already in a task context
            return Err("Already in a task context".to_string());
        }

        registry.current_task_id = Some(task_id);
        Ok(())
    }

    /// Exit the current task context
    fn exit_task(&self) -> Result<(), String> {
        let mut registry = self.registry.lock().unwrap();

        if registry.current_task_id.is_none() {
            // Not in a task context
            return Err("Not in a task context".to_string());
        }

        registry.current_task_id = None;
        Ok(())
    }

    /// Record an allocation for the current task (if any)
    fn record_alloc(&self, ptr: *mut u8, layout: Layout) {
        let mut registry = self.registry.lock().unwrap();

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

    /// Record a deallocation, removing it from the task's allocation list
    fn record_dealloc(&self, ptr: *mut u8) {
        let mut registry = self.registry.lock().unwrap();
        let address = ptr as usize;

        // Find which task owns this allocation
        if let Some(task_id) = registry.address_to_task.remove(&address) {
            // Remove from task's allocation list
            if let Some(allocations) = registry.task_allocations.get_mut(&task_id) {
                if let Some(index) = allocations.iter().position(|(addr, _)| *addr == address) {
                    allocations.swap_remove(index);
                }
            }
        }
    }
}

// Implement the GlobalAlloc trait
unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        if !ptr.is_null() {
            self.record_alloc(ptr, layout);
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !ptr.is_null() {
            self.record_dealloc(ptr);
        }

        self.inner.dealloc(ptr, layout);
    }
}

#[cfg(feature = "full_profiling")]
impl TaskMemoryContext {
    /// Activates this task context for memory tracking
    pub fn enter(&self) -> Result<TaskGuard, String> {
        match self.allocator.enter_task(self.task_id) {
            Ok(()) => Ok(TaskGuard {
                task_id: self.task_id,
                allocator: self.allocator,
            }),
            Err(e) => Err(e),
        }
    }

    /// Gets the unique ID for this task
    pub fn id(&self) -> usize {
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

/// RAII guard for task context
#[cfg(feature = "full_profiling")]
#[derive(Debug)]
pub struct TaskGuard<'a> {
    task_id: usize,
    allocator: &'a TaskAwareAllocator<System>,
}

#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskGuard;

#[cfg(feature = "full_profiling")]
impl<'a> Drop for TaskGuard<'a> {
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

// Use static initialization that works with non-const HashMap::new
#[cfg(feature = "full_profiling")]
#[global_allocator]
static ALLOCATOR: TaskAwareAllocator<System> = {
    extern crate std;
    use std::alloc::System;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    TaskAwareAllocator {
        inner: System,
        next_task_id: AtomicUsize::new(1),
        // Empty registry that will be initialized at runtime
        registry: Mutex::new(AllocationRegistry {
            task_allocations: HashMap::new(),
            address_to_task: HashMap::new(),
            current_task_id: None,
        }),
    }
};

// Helper to get the allocator instance
#[cfg(feature = "full_profiling")]
pub fn get_allocator() -> &'static TaskAwareAllocator<System> {
    &ALLOCATOR
}

/// Creates a new task context for memory tracking.
#[cfg(feature = "full_profiling")]
pub fn create_memory_task() -> TaskMemoryContext {
    get_allocator().create_task_context()
}

/// No-op version when profiling is disabled.
#[cfg(not(feature = "full_profiling"))]
pub fn create_memory_task() -> TaskMemoryContext {
    TaskMemoryContext
}
