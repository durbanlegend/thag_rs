/// Prototype of ring-fenced memory allocators for `thag_profiler`.
///
/// The `global_allocator` attribute flags a `Dispatcher` which dispatches each
/// memory allocation, deallocation and reallocation requests to one of two allocators
/// according to the designated current allocator at the moment that it receives
/// the request. The default allocator is `TaskAware` and is used for user code,
/// while the regular system allocator `System` handles requests from profiler code.
/// The role of the `TaskAware` allocator is to record the details of the user code
/// allocation events before passing them to the system allocator.
///
/// To invoke the system allocator directly, profiler code must call a function or
/// closure with fn `with_sys_alloc`, which checks the current allocator, and if it
/// finds it to be `TaskAware`, changes it to `System` and runs the function or closure,
/// with a guard to restore the default to `TaskAware`. If the current allocator is
/// already `System`, `with_sys_alloc` concludes that it must be running nested under
/// another `with_sys_alloc` call, so does nothing except run the function or closure.
///
/// The flaw in this design is its vulnerability to race conditions, e.g. user code
/// in another thread could fail to go through `TaskAware` if `with_sys_alloc` is
/// running concurrently, or conversely an outer `with_sys_alloc` ending in one thread
/// could prematurely reset the current allocator to  `TaskAware` while another
/// instance is still running in another thread. We can and do build in a check in
/// the TaskAware branch to detect and ignore profiler code, but in practice there is
/// little sign of such races being a problem.
///
/// Attempts to resolve this issue with thread-local storage have not borne fruit.
/// For instance async tasks are by no means guaranteed to resume in the same thread
/// after suspension.
/// The ideal would seem to be a reentrant Mutext with mutability - so far tried
/// without success, but a subject for another prototype.
//# Purpose: Prototype of a ring-fenced allocator for memory profiling.
//# Categories: profiling, prototype
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::LazyLock;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fmt,
};

static USING_SYSTEM_ALLOCATOR: LazyLock<Arc<RwLock<bool>>> =
    LazyLock::new(|| Arc::new(RwLock::new(false)));

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Allocator {
    /// Task-aware allocator that tracks which task allocated memory
    TaskAware,
    /// System allocator for profiling operations
    System,
}

impl fmt::Display for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Allocator::TaskAware => write!(f, "TaskAware"),
            Allocator::System => write!(f, "System"),
        }
    }
}

pub fn with_sys_alloc<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    if *USING_SYSTEM_ALLOCATOR.read() {
        eprintln!("Already in SystemAllocator");
        return f();
    }

    // Create struct to handle cleanup on drop
    struct CleanupGuard<'a>(&'a RwLock<bool>);

    impl<'a> Drop for CleanupGuard<'a> {
        fn drop(&mut self) {
            *self.0.write() = false;
        }
    }

    // Set the flag and create a guard
    *USING_SYSTEM_ALLOCATOR.write() = true;
    let guard = CleanupGuard(&USING_SYSTEM_ALLOCATOR);

    // Assert that we've successfully set the flag
    let using_sys_alloc = *USING_SYSTEM_ALLOCATOR.read();
    assert!(using_sys_alloc);

    // Run the function (guard will reset flag on function exit or panic)
    f()
}

pub fn current_allocator() -> Allocator {
    if *USING_SYSTEM_ALLOCATOR.read() {
        // eprintln!("Using system allocator");
        Allocator::System
    } else {
        Allocator::TaskAware
    }
}

// Create a direct static instance
#[global_allocator]
static ALLOCATOR: Dispatcher = Dispatcher::new();

/// Dispatcher allocator that routes allocation requests to the appropriate allocator
pub struct Dispatcher {
    pub task_aware: TaskAwareAllocator,
    pub system: std::alloc::System,
}

impl Dispatcher {
    pub const fn new() -> Self {
        Self {
            task_aware: TaskAwareAllocator,
            system: std::alloc::System,
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for Dispatcher {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let current = current_allocator();

        match current {
            Allocator::System => unsafe { self.system.alloc(layout) },
            Allocator::TaskAware => unsafe { self.task_aware.alloc(layout) },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        match current_allocator() {
            Allocator::System => unsafe { self.system.dealloc(ptr, layout) },
            Allocator::TaskAware => unsafe { self.task_aware.dealloc(ptr, layout) },
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if ptr.is_null() {
            return unsafe {
                self.alloc(Layout::from_size_align_unchecked(new_size, layout.align()))
            };
        }

        match current_allocator() {
            Allocator::System => unsafe { self.system.realloc(ptr, layout, new_size) },
            Allocator::TaskAware => unsafe { self.task_aware.realloc(ptr, layout, new_size) },
        }
    }
}

/// Task-aware allocator that tracks memory allocations
pub struct TaskAwareAllocator;

// Static instance for global access
#[allow(dead_code)]
static TASK_AWARE_ALLOCATOR: TaskAwareAllocator = TaskAwareAllocator;

unsafe impl GlobalAlloc for TaskAwareAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        with_sys_alloc(|| {
            let ptr = unsafe { System.alloc(layout) };

            println!("In TaskAwareAllocator for size {}", layout.size());

            ptr
        })
    }

    #[allow(clippy::too_many_lines)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        with_sys_alloc(|| {
            if !ptr.is_null() {
                // println!("In system deallocator for size {}", layout.size());

                // Forward to system allocator for deallocation
                unsafe { System.dealloc(ptr, layout) };
            }
        });
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, _new_size: usize) -> *mut u8 {
        if !ptr.is_null() {
            with_sys_alloc(|| {
                println!("In system reallocator for size {}", layout.size());
            });
        }
        ptr
    }
}

fn main() {
    let data1: Vec<u8> = vec![0; 1024];

    println!("1. current_allocator()={}", current_allocator());

    let data2: Vec<u8> = with_sys_alloc(|| {
        with_sys_alloc(|| println!("Nested sys alloc"));
        vec![0; 2048]
    });

    println!("2. current_allocator()={}", current_allocator());

    with_sys_alloc(|| println!("data1.len()={}, data2.len()={}", data1.len(), data2.len()));

    println!("3. current_allocator()={}", current_allocator());
}
