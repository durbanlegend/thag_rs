#![deny(unsafe_op_in_unsafe_fn)]
use crate::profiling::extract_callstack_from_alloc_backtrace;
use crate::task_allocator::{activate_task, TaskMemoryContext, MINIMUM_TRACKED_SIZE, TASK_STATE};
use crate::{
    debug_log, extract_path, find_matching_profile, get_last_active_task, regex, trim_backtrace,
    ALLOC_REGISTRY,
};
use backtrace::Backtrace;
use regex::Regex;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Instant;

/// Enum defining all available allocators used by the profiler
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Allocator {
    TaskAware,
    System,
}

// Thread-local current allocator selection
thread_local! {
    static CURRENT_ALLOCATOR: UnsafeCell<Allocator> = const { UnsafeCell::new(Allocator::TaskAware) };
}

// Function to get current allocator type
pub fn current_allocator() -> Allocator {
    CURRENT_ALLOCATOR.with(|curr_alloc| unsafe { *curr_alloc.get() })
}

// Function to run code with a specific allocator
pub fn with_allocator<T, F: FnOnce() -> T>(req_alloc: Allocator, f: F) -> T {
    CURRENT_ALLOCATOR.with(|curr_alloc| {
        // Save the current allocator
        let prev = unsafe { *curr_alloc.get() };

        // Set the new allocator
        unsafe { *curr_alloc.get() = req_alloc };

        // Run the function
        let result = f();

        // Restore the previous allocator
        unsafe { *curr_alloc.get() = prev };

        result
    })
}

/// Task-aware allocator that tracks memory allocations
pub struct TaskAwareAllocator;

#[allow(clippy::unused_self)]
impl TaskAwareAllocator {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize in profile registry
        activate_task(task_id);

        TaskMemoryContext { task_id }
    }
}

unsafe impl GlobalAlloc for TaskAwareAllocator {
    #[allow(clippy::too_many_lines)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };

        if !ptr.is_null() {
            with_allocator(Allocator::System, || {
                // Skip small allocations
                let size = layout.size();
                if size > MINIMUM_TRACKED_SIZE {
                    // Simple recursion prevention without using TLS with destructors
                    static mut IN_TRACKING: bool = false;
                    struct Guard;
                    impl Drop for Guard {
                        fn drop(&mut self) {
                            unsafe {
                                IN_TRACKING = false;
                            }
                        }
                    }

                    // Flag if we're already tracking in case it causes an infinite recursion
                    if unsafe { IN_TRACKING } {
                        debug_log!(
                            "*** Caution: already tracking: proceeding for allocation of {size} B"
                        );
                        // return ptr;
                    }

                    // Set tracking flag and create guard for cleanup
                    unsafe {
                        IN_TRACKING = true;
                    }
                    let _guard = Guard;

                    // Get backtrace without recursion
                    // debug_log!("Attempting backtrace");
                    // Use a different allocator for backtrace operations
                    let start_ident = Instant::now();
                    let mut task_id = 0;
                    // Now we can safely use backtrace without recursion!
                    let start_pattern: &Regex = regex!("thag_profiler::mem_alloc.+Dispatcher");

                    // debug_log!("Calling extract_callstack");
                    let mut current_backtrace = Backtrace::new_unresolved();
                    let cleaned_stack = extract_callstack_from_alloc_backtrace(
                        start_pattern,
                        &mut current_backtrace,
                    );
                    debug_log!("Cleaned_stack for size={size}: {cleaned_stack:?}");
                    let in_profile_code = cleaned_stack.iter().any(|frame| {
                        frame.contains("Backtrace::new") || frame.contains("Profile::new")
                    });

                    if in_profile_code {
                        debug_log!("Ignoring allocation request of size {size} for profiler code");
                        return;
                    }

                    current_backtrace.resolve();

                    if cleaned_stack.is_empty() {
                        debug_log!(
                            "...empty cleaned_stack for backtrace: size={size}:\n{:#?}",
                            trim_backtrace(start_pattern, &current_backtrace)
                        );
                        debug_log!("Getting last active task (hmmm :/)");
                        task_id = get_last_active_task().unwrap_or(0);
                    } else {
                        // Make sure the use of a separate allocator is working.
                        assert!(!cleaned_stack
                            .iter()
                            .any(|frame| frame.contains("find_matching_profile")));

                        debug_log!("Calling extract_path");
                        let path = extract_path(&cleaned_stack);
                        if path.is_empty() {
                            let trimmed_backtrace =
                                trim_backtrace(start_pattern, &current_backtrace);
                            if trimmed_backtrace
                                .iter()
                                .any(|frame| frame.contains("Backtrace::new"))
                            {
                                debug_log!("Ignoring setup allocation of size {size} containing Backtrace::new:");
                                return;
                            }
                            debug_log!(
                                "...path is empty for thread {:?}, size: {size:?}, not eligible for allocation",
                                thread::current().id(),
                            );
                        } else {
                            task_id = find_matching_profile(&path);
                            debug_log!(
                                "...find_matching_profile found task_id={task_id} for size={size}"
                            );
                        }
                    }
                    // with_allocator(Allocator::System, || {
                    debug_log!(
                        "task_id={task_id}, size={size}, time to assign = {}ms",
                        start_ident.elapsed().as_millis()
                    );
                    // });

                    // Record allocation if task found
                    if task_id == 0 {
                        // TODO record in suspense file and allocate later
                        return;
                    }

                    let start_record_alloc = Instant::now();
                    // Use system allocator to avoid recursive allocations
                    // with_allocator(Allocator::System, || {
                    let address = ptr as usize;

                    debug_log!("Recording allocation for task_id={task_id}, address={address:#x}, size={size}");
                    let mut registry = ALLOC_REGISTRY.lock();
                    registry
                        .task_allocations
                        .entry(task_id)
                        .or_default()
                        .push((address, size));

                    registry.address_to_task.insert(address, task_id);
                    let check_map = &registry.task_allocations;
                    let reg_task_id = *registry.address_to_task.get(&address).unwrap();
                    let maybe_vec = check_map.get(&task_id);
                    let (addr, sz) = *maybe_vec
                        .and_then(|v: &Vec<(usize, usize)>| {
                            let last = v.iter().filter(|&(addr, _)| *addr == address).last();
                            last
                        })
                        .unwrap();
                    drop(registry);
                    assert_eq!(sz, size);
                    assert_eq!(addr, address);
                    assert_eq!(reg_task_id, task_id);
                    debug_log!(
                        "Time to record allocation: {}ms",
                        start_record_alloc.elapsed().as_millis()
                    );
                }
            });
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Just forward to system allocator for deallocation
        unsafe { System.dealloc(ptr, layout) };
    }
}

/// Dispatcher allocator that routes allocation requests to the appropriate allocator
pub struct Dispatcher {
    task_aware: TaskAwareAllocator,
    system: System,
}

impl Dispatcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            task_aware: TaskAwareAllocator,
            system: System,
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
        // Get current allocator type from thread-local storage
        match current_allocator() {
            Allocator::TaskAware => unsafe { self.task_aware.alloc(layout) },
            Allocator::System => unsafe { self.system.alloc(layout) },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // For deallocation, we don't need to check the allocator type, as both
        // allocators use System for the actual deallocation.
        unsafe { self.system.dealloc(ptr, layout) };
    }
}

// Create a direct static instance
#[global_allocator]
static ALLOCATOR: Dispatcher = Dispatcher::new();
