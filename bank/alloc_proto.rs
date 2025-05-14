use std::{
    alloc::{GlobalAlloc, Layout, System},
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

static USING_SYSTEM_ALLOCATOR: AtomicBool = AtomicBool::new(false);

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
    if current_allocator() == Allocator::System {
        return f();
    }

    USING_SYSTEM_ALLOCATOR.store(true, Ordering::SeqCst);

    // Create struct to handle cleanup on drop
    struct Cleanup;

    impl Drop for Cleanup {
        fn drop(&mut self) {
            USING_SYSTEM_ALLOCATOR.store(false, Ordering::SeqCst);
        }
    }

    // Create guard to restore on scope exit
    let _cleanup = Cleanup {};
    // Run the function
    f()
}

pub fn current_allocator() -> Allocator {
    if USING_SYSTEM_ALLOCATOR.load(Ordering::Relaxed) {
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

    let data2: Vec<u8> = with_sys_alloc(|| vec![0; 2048]);

    with_sys_alloc(|| println!("data1.len()={}, data2.len()={}", data1.len(), data2.len()));
    println!("Hello world!");
    // with_sys_alloc(|| println!("Hello world!"));
}
