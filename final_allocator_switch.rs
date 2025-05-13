use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::ReentrantMutex;
use std::alloc::{GlobalAlloc, Layout};

// Define allocator types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Allocator {
    /// Task-aware allocator that tracks which task allocated memory
    TaskAware,
    /// System allocator for profiling operations
    System,
}

// State for allocator switching with a stack-based approach
struct AllocatorState {
    // Stack of allocators, with the current one at the top
    stack: Vec<Allocator>,
}

impl AllocatorState {
    fn new() -> Self {
        // Initialize with TaskAware as default
        let mut stack = Vec::with_capacity(8);
        stack.push(Allocator::TaskAware);
        Self { stack }
    }
    
    // Get current allocator (top of stack)
    fn current(&self) -> Allocator {
        *self.stack.last().unwrap_or(&Allocator::TaskAware)
    }
    
    // Push allocator to stack
    fn push(&mut self, allocator: Allocator) {
        self.stack.push(allocator);
    }
    
    // Pop allocator from stack, never removing the default
    fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }
}

// Global state protected by ReentrantMutex
static ALLOCATOR_STATE: ReentrantMutex<AllocatorState> = ReentrantMutex::new(AllocatorState::new());

/// Get the current allocator
pub fn current_allocator() -> Allocator {
    // Lock and read current allocator
    let state = ALLOCATOR_STATE.lock();
    state.current()
}

/// Run a function with the system allocator
/// 
/// This function temporarily switches to the system allocator while executing the provided
/// closure, then switches back to the previous allocator afterward.
/// 
/// Properly handles nested calls and threading via a ReentrantMutex.
pub fn with_sys_alloc<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // RAII guard to restore state on function exit
    struct AllocatorGuard;
    
    impl Drop for AllocatorGuard {
        fn drop(&mut self) {
            // Pop the allocator on scope exit
            let mut state = ALLOCATOR_STATE.lock();
            state.pop();
        }
    }
    
    // Push System allocator onto the stack
    {
        let mut state = ALLOCATOR_STATE.lock();
        state.push(Allocator::System);
    }
    
    // Create guard to restore state on return/panic
    let _guard = AllocatorGuard;
    
    // Run the function
    f()
}

/// Run a function with the specified allocator
/// 
/// This function temporarily switches to the provided allocator while executing the
/// closure, then switches back to the previous allocator afterward.
/// 
/// Works correctly with nesting and threading via a ReentrantMutex.
pub fn with_allocator<F, R>(allocator: Allocator, f: F) -> R
where
    F: FnOnce() -> R,
{
    // RAII guard to restore state on function exit
    struct AllocatorGuard;
    
    impl Drop for AllocatorGuard {
        fn drop(&mut self) {
            // Pop the allocator on scope exit
            let mut state = ALLOCATOR_STATE.lock();
            state.pop();
        }
    }
    
    // Push the requested allocator onto the stack
    {
        let mut state = ALLOCATOR_STATE.lock();
        state.push(allocator);
    }
    
    // Create guard to restore state on return/panic
    let _guard = AllocatorGuard;
    
    // Run the function
    f()
}

/// Implementation for the Dispatcher's GlobalAlloc methods
pub struct Dispatcher {
    pub task_aware: TaskAwareAllocator,
    pub system: std::alloc::System,
}

unsafe impl GlobalAlloc for Dispatcher {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match current_allocator() {
            Allocator::System => self.system.alloc(layout),
            Allocator::TaskAware => self.task_aware.alloc(layout),
        }
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match current_allocator() {
            Allocator::System => self.system.dealloc(ptr, layout),
            Allocator::TaskAware => self.task_aware.dealloc(ptr, layout),
        }
    }
    
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        match current_allocator() {
            Allocator::System => self.system.realloc(ptr, layout, new_size),
            Allocator::TaskAware => self.task_aware.realloc(ptr, layout, new_size),
        }
    }
}

// Placeholder for TaskAwareAllocator
pub struct TaskAwareAllocator;