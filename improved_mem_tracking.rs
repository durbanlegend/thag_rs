use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::RwLock;
use std::cell::Cell;

// Structure to hold the current allocator state with thread safety
struct AllocatorState {
    // Keeps track of the allocator stack for each thread
    allocator_stack: Vec<Allocator>,
}

impl AllocatorState {
    fn new() -> Self {
        Self {
            allocator_stack: vec![Allocator::TaskAware], // Default allocator
        }
    }

    // Get the current allocator (top of stack)
    fn current(&self) -> Allocator {
        *self.allocator_stack.last().unwrap_or(&Allocator::TaskAware)
    }

    // Push a new allocator onto the stack
    fn push(&mut self, allocator: Allocator) {
        self.allocator_stack.push(allocator);
    }

    // Pop the top allocator from the stack
    fn pop(&mut self) -> Option<Allocator> {
        if self.allocator_stack.len() > 1 {
            self.allocator_stack.pop()
        } else {
            // Never pop the last element (default allocator)
            None
        }
    }
}

// Global allocator state protected by a RwLock
// Using RwLock for better performance (many readers, few writers)
static ALLOCATOR_STATE: parking_lot::RwLock<AllocatorState> = parking_lot::RwLock::new(AllocatorState::new());

// Thread-local recursion counter to optimize allocator switches
thread_local! {
    static RECURSION_COUNTER: Cell<usize> = Cell::new(0);
}

/// Get the current allocator safely
pub fn current_allocator() -> Allocator {
    // Use a read lock for better concurrency
    let state = ALLOCATOR_STATE.read();
    state.current()
}

/// Run a function with the system allocator
/// This correctly handles nesting and concurrency
pub fn with_sys_alloc<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Use RAII pattern with AllocatorGuard
    struct AllocatorGuard;
    
    impl Drop for AllocatorGuard {
        fn drop(&mut self) {
            // On scope exit, restore the previous allocator
            let mut state = ALLOCATOR_STATE.write();
            let _ = state.pop();
            
            // Update recursion counter
            RECURSION_COUNTER.with(|counter| {
                let current = counter.get();
                if current > 0 {
                    counter.set(current - 1);
                }
            });
        }
    }
    
    // Check if we're already in a with_sys_alloc call
    let is_nested = RECURSION_COUNTER.with(|counter| {
        let current = counter.get();
        counter.set(current + 1);
        current > 0
    });
    
    // Only modify the allocator stack if this isn't a nested call
    if !is_nested {
        let mut state = ALLOCATOR_STATE.write();
        state.push(Allocator::System);
    }
    
    // Create guard to ensure we restore state on return or panic
    let _guard = AllocatorGuard;
    
    // Execute the function
    f()
}

/// Run a function with the specified allocator
/// This handles nesting and properly restores the previous allocator
pub fn with_allocator<F, R>(allocator: Allocator, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Use RAII pattern with AllocatorGuard
    struct AllocatorGuard;
    
    impl Drop for AllocatorGuard {
        fn drop(&mut self) {
            // On scope exit, restore the previous allocator
            let mut state = ALLOCATOR_STATE.write();
            let _ = state.pop();
            
            // Update recursion counter
            RECURSION_COUNTER.with(|counter| {
                let current = counter.get();
                if current > 0 {
                    counter.set(current - 1);
                }
            });
        }
    }
    
    // Track recursion to optimize
    RECURSION_COUNTER.with(|counter| {
        counter.set(counter.get() + 1);
    });
    
    // Modify the allocator stack
    let mut state = ALLOCATOR_STATE.write();
    state.push(allocator);
    drop(state); // Release the lock before executing the function
    
    // Create guard to ensure we restore state on return or panic
    let _guard = AllocatorGuard;
    
    // Execute the function
    f()
}

/// For the global allocator implementation
/// This should be used in the GlobalAlloc implementations
pub fn get_current_allocator_for_dispatcher() -> Allocator {
    current_allocator()
}

// Example usage in GlobalAlloc implementation:
// 
// unsafe impl GlobalAlloc for Dispatcher {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         match get_current_allocator_for_dispatcher() {
//             Allocator::System => self.system.alloc(layout),
//             Allocator::TaskAware => self.task_aware.alloc(layout),
//         }
//     }
//     
//     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
//         match get_current_allocator_for_dispatcher() {
//             Allocator::System => self.system.dealloc(ptr, layout),
//             Allocator::TaskAware => self.task_aware.dealloc(ptr, layout),
//         }
//     }
// }