use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::ReentrantMutex;
use std::cell::Cell;

// Define allocator types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Allocator {
    /// Task-aware allocator that tracks which task allocated memory
    TaskAware,
    /// System allocator for profiling operations
    System,
}

// State for allocator switching
struct AllocatorContext {
    // The currently active allocator
    current: Allocator,
    // Stack to track nested calls
    stack: Vec<Allocator>,
}

impl AllocatorContext {
    fn new() -> Self {
        Self {
            current: Allocator::TaskAware, // Default allocator
            stack: Vec::with_capacity(8),  // Pre-allocate for efficiency
        }
    }
    
    fn push(&mut self, allocator: Allocator) -> Allocator {
        let prev = self.current;
        self.stack.push(prev);
        self.current = allocator;
        prev
    }
    
    fn pop(&mut self) -> Allocator {
        if let Some(prev) = self.stack.pop() {
            let current = self.current;
            self.current = prev;
            current
        } else {
            // If stack is empty, keep the current allocator
            self.current
        }
    }
    
    fn current(&self) -> Allocator {
        self.current
    }
}

// Global context protected by a reentrant mutex for thread safety
// ReentrantMutex allows the same thread to lock multiple times
static ALLOCATOR_CONTEXT: ReentrantMutex<AllocatorContext> = ReentrantMutex::new(AllocatorContext::new());

/// Get the current allocator
pub fn current_allocator() -> Allocator {
    let context = ALLOCATOR_CONTEXT.lock();
    context.current()
}

/// Run a function with the system allocator
/// Properly handles nested calls and restores previous allocator
pub fn with_sys_alloc<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Create RAII guard for allocator switching
    struct AllocatorGuard<'a>(&'a ReentrantMutex<AllocatorContext>);
    
    impl<'a> Drop for AllocatorGuard<'a> {
        fn drop(&mut self) {
            // Restore previous allocator on scope exit
            let mut context = self.0.lock();
            context.pop();
        }
    }
    
    // Lock the context and switch to system allocator
    let mut context = ALLOCATOR_CONTEXT.lock();
    context.push(Allocator::System);
    
    // Create guard that will restore allocator on return
    let guard = AllocatorGuard(&ALLOCATOR_CONTEXT);
    
    // Drop the lock before executing the function
    drop(context);
    
    // Execute the function
    f()
    
    // Guard automatically restores previous allocator when it goes out of scope
}

/// Run a function with the specified allocator
/// Properly handles nested calls and restores previous allocator
pub fn with_allocator<F, R>(allocator: Allocator, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Create RAII guard for allocator switching
    struct AllocatorGuard<'a>(&'a ReentrantMutex<AllocatorContext>);
    
    impl<'a> Drop for AllocatorGuard<'a> {
        fn drop(&mut self) {
            // Restore previous allocator on scope exit
            let mut context = self.0.lock();
            context.pop();
        }
    }
    
    // Lock the context and switch allocator
    let mut context = ALLOCATOR_CONTEXT.lock();
    context.push(allocator);
    
    // Create guard that will restore allocator on return
    let guard = AllocatorGuard(&ALLOCATOR_CONTEXT);
    
    // Drop the lock before executing the function
    drop(context);
    
    // Execute the function
    f()
    
    // Guard automatically restores previous allocator when it goes out of scope
}

/// Safe way to update the GlobalAlloc implementation 
/// for the Dispatcher to use the current allocator
///
/// This function should be called from the GlobalAlloc impl methods
pub fn get_current_allocator_for_dispatcher() -> Allocator {
    current_allocator()
}

/// Example GlobalAlloc implementation:
/// 
/// ```
/// unsafe impl GlobalAlloc for Dispatcher {
///     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
///         match get_current_allocator_for_dispatcher() {
///             Allocator::System => self.system.alloc(layout),
///             Allocator::TaskAware => self.task_aware.alloc(layout),
///         }
///     }
///     
///     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
///         match get_current_allocator_for_dispatcher() {
///             Allocator::System => self.system.dealloc(ptr, layout),
///             Allocator::TaskAware => self.task_aware.dealloc(ptr, layout),
///         }
///     }
/// }
/// ```