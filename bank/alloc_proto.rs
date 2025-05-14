/*[toml]
[dependencies]
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
#thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/

use parking_lot::RwLock;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fmt,
    sync::{Arc, LazyLock},
};

struct AllocatorState {
    curr_alloc: Allocator,
}

impl AllocatorState {
    const fn new() -> Self {
        Self {
            curr_alloc: Allocator::TaskAware,
        }
    }
}

static ALLOCATOR_STATE: LazyLock<Arc<RwLock<AllocatorState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AllocatorState::new())));

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
    let current;
    {
        let state = ALLOCATOR_STATE.read();
        current = state.curr_alloc;
        // println!("Before upgrading, current allocator: {:?}", current);
    } // drop the read lock
    if current == Allocator::System {
        f()
    } else {
        // Now get a write lock
        {
            let mut state = ALLOCATOR_STATE.write();
            state.curr_alloc = Allocator::System;
        } // drop the write lock
        let result = f();
        // Now get a write lock
        {
            let mut state = ALLOCATOR_STATE.write();
            state.curr_alloc = Allocator::TaskAware;
        } // drop the write lock
        result
    }
}

pub fn current_allocator() -> Allocator {
    let state = ALLOCATOR_STATE.read();
    state.curr_alloc
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

            // println!("In system allocator for size {}", layout.size());

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
    // let data1: Vec<u8> = vec![0; 1024];

    // let data2: Vec<u8> = with_sys_alloc(|| vec![0; 2048]);

    // println!("data1={data1:#?}, data2={data2:#?}");
    // with_sys_alloc(|| println!("data2={data2:#?}"));
    with_sys_alloc(|| println!("Hello world!"));
}
