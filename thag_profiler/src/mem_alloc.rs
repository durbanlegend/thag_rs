#![deny(unsafe_op_in_unsafe_fn)]
use crate::profiling::extract_callstack_from_alloc_backtrace;
use crate::task_allocator::{
    activate_task, /*AllocationRegistry,*/ TaskMemoryContext,
    /*TaskState,*/ MINIMUM_TRACKED_SIZE, TASK_STATE,
};
use crate::{
    debug_log, extract_path, find_matching_profile, get_last_active_task, regex, trim_backtrace,
    ALLOC_REGISTRY,
};
use backtrace::Backtrace;
use regex::Regex;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Instant;

pub trait AllocatorProvider<T: Copy + 'static> {
    fn get_allocator(&self, tag: T) -> &dyn GlobalAlloc;
}

// The main allocator struct
pub struct MultiAllocator<T, A>
where
    T: Copy + 'static,
    A: AllocatorProvider<T>,
{
    allocators: A,
    _marker: PhantomData<T>,
}

impl<T, A> MultiAllocator<T, A>
where
    T: Copy + 'static,
    A: AllocatorProvider<T>,
{
    pub const fn new(allocators: A) -> Self {
        Self {
            allocators,
            _marker: PhantomData,
        }
    }

    unsafe fn allocate(&self, layout: Layout, tag: T) -> *mut u8 {
        let allocator = self.allocators.get_allocator(tag);

        // Create a new layout that includes space for the tag and ensures proper alignment
        let (alloc_layout, offset) = Self::layout_with_tag(layout);

        let ptr = unsafe { allocator.alloc(alloc_layout) };
        if ptr.is_null() {
            return ptr;
        }

        // Store the tag at the beginning of the allocated memory
        // We'll use memcpy to avoid alignment issues
        let tag_ptr = ptr.cast::<T>();
        unsafe { std::ptr::write_unaligned(tag_ptr, tag) };

        // Return the adjusted pointer (after the tag)
        unsafe { ptr.add(offset) }
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        // Calculate the layout with space for our tag
        let (alloc_layout, offset) = Self::layout_with_tag(layout);

        // Get back to the original pointer (before the tag)
        let original_ptr = unsafe { ptr.sub(offset) };

        // Read the tag to know which allocator to use
        // Use unaligned read to avoid alignment issues
        let tag_ptr = original_ptr as *const T;
        let tag = unsafe { std::ptr::read_unaligned(tag_ptr) };

        let allocator = self.allocators.get_allocator(tag);
        unsafe { allocator.dealloc(original_ptr, alloc_layout) };
    }

    // This method calculates the layout for an allocation that includes the tag
    fn layout_with_tag(layout: Layout) -> (Layout, usize) {
        unsafe {
            // We can't use std::mem::size_of::<T>() as a const generic,
            // so we'll calculate the values at runtime
            let tag_size = mem::size_of::<T>();
            let tag_align = mem::align_of::<T>();

            // To avoid alignment issues, we'll use the maximum alignment
            let align = layout.align().max(tag_align);

            // Padding to ensure proper alignment after the tag
            let offset = (tag_size + align - 1) & !(align - 1);

            // Total size needed
            let size = offset + layout.size();

            // Create a new layout
            (Layout::from_size_align_unchecked(size, align), offset)
        }
    }
}

// Now let's define concrete types for our allocator implementation
// instead of using generics in constant expressions

// Define our allocator type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum AllocatorType {
    TaskAware = 0, // Default is 0
    SystemAlloc,
}

// Thread-local current allocator
thread_local! {
    static CURRENT_ALLOCATOR: UnsafeCell<AllocatorType> = const { UnsafeCell::new(AllocatorType::TaskAware) };
}

// Function to get current allocator
pub fn current_allocator() -> AllocatorType {
    CURRENT_ALLOCATOR.with(|tag| unsafe { *tag.get() })
}

// Function to run code with a specific allocator
pub fn with_allocator<T, F: FnOnce() -> T>(allocator: AllocatorType, f: F) -> T {
    CURRENT_ALLOCATOR.with(|tag| {
        // Save the current allocator
        let prev = unsafe { *tag.get() };
        
        // Set the new allocator
        unsafe { *tag.get() = allocator };
        
        // Run the function
        let result = f();
        
        // Restore the previous allocator
        unsafe { *tag.get() = prev };
        
        result
    })
}

pub struct TaskAwareAllocator;

// unsafe impl GlobalAlloc for TaskAwareAllocator {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         System.alloc(layout)
//     }

//     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
//         System.dealloc(ptr, layout)
//     }
// }

// Define registry-specific methods for System allocator
#[allow(clippy::unused_self)]
impl TaskAwareAllocator /*<System>*/ {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize in profile registry
        activate_task(task_id);

        TaskMemoryContext { task_id }
    }
}

// unsafe impl<A: GlobalAlloc> GlobalAlloc for TaskAwareAllocator<A> {
unsafe impl GlobalAlloc for TaskAwareAllocator {
    #[allow(clippy::too_many_lines)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };

        if !ptr.is_null() {
            // Skip small allocations
            let size = layout.size();
            if size >= MINIMUM_TRACKED_SIZE {
                // Simple recursion prevention without using TLS with destructors
                static mut IN_TRACKING: bool = false;
                struct Guard;
                impl Drop for Guard {
                    fn drop(&mut self) {
                        unsafe { IN_TRACKING = false; }
                    }
                }
                
                // Only proceed if we're not already tracking
                if unsafe { IN_TRACKING } {
                    return ptr;
                }
                
                // Set tracking flag and create guard for cleanup
                unsafe { IN_TRACKING = true; }
                let _guard = Guard;

                // Get backtrace without recursion
                // debug_log!("Attempting backtrace");
                // Use a different allocator for backtrace operations
                let start_ident = Instant::now();
                let mut task_id = 0;
                with_allocator(AllocatorType::SystemAlloc, || {
                    // Now we can safely use backtrace without recursion!
                    let start_pattern: &Regex = regex!("thag_profiler::okaoka.+MultiAllocator");

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
                        // let bt = Backtrace::new();
                        // debug_log!(""{bt});
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
                                // Don't record the allocation because it's profiling setup
                                // Backtrace::frames(&current_backtrace)
                                //     .iter()
                                //     .flat_map(backtrace::BacktraceFrame::symbols)
                                //     .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                                //     .for_each(|frame| {
                                //         debug_log!("frame: {}", frame);
                                //     });
                                return;
                            }
                            // debug_log!(
                            //     "...path is empty for thread {:?}: assigning to lastest active task.\nCleaned_stack: {:?}",
                            //     thread::current().id(),
                            //     cleaned_stack
                            // );
                            debug_log!(
                                "...path is empty for thread {:?}, size: {size:?}, not eligible for allocation",
                                thread::current().id(),
                            );
                            // debug_log!("...backtrace:\n{trimmed_backtrace:?}");
                            // let last_active_task = get_last_active_task();
                            // task_id = last_active_task.unwrap_or(0);
                            // if task_id == 0 {
                            //     debug_log!(
                            //         "...no active task found, calling get_active_tasks to confirm"
                            //     );
                            //     debug_log!("...active tasks: {:?}", get_active_tasks());
                            // }
                        } else {
                            // debug_log!("path={path:#?}");

                            task_id = find_matching_profile(&path);
                            debug_log!(
                                "...find_matching_profile found task_id={task_id} for size={size}"
                            );
                        }
                    }
                });
                with_allocator(AllocatorType::SystemAlloc, || {
                    debug_log!(
                        "task_id={task_id}, size={size}, time to assign = {}ms",
                        start_ident.elapsed().as_millis()
                    );
                });

                // Record allocation if task found
                if task_id == 0 {
                    return ptr;
                }

                let start_record_alloc = Instant::now();
                // Use system allocator to avoid recursive allocations
                with_allocator(AllocatorType::SystemAlloc, || {
                    let address = ptr as usize;
                    // let size = layout.size();

                    // Record in thread-local buffer
                    // record_allocation(task_id, address, size);
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
                    // debug_log!("Check registry.task_allocations for task_id {task_id}: ({addr:#x}, {sz})");
                    assert_eq!(sz, size);
                    assert_eq!(addr, address);
                    // debug_log! (
                    //     "Check registry.address_to_task for task_id {task_id}: {:?}",
                    //     registry.address_to_task.get(&address)
                    // );
                    assert_eq!(reg_task_id, task_id);
                    // debug_log!(
                    //     "task {task_id} memory usage: {:?}",
                    //     ALLOC_REGISTRY.lock().get_task_memory_usage(task_id)
                    // );
                    debug_log!(
                        "Time to record allocation: {}ms",
                        start_record_alloc.elapsed().as_millis()
                    );
                });
            } else {
                // debug_log!("ignoring allocation of {} bytes", layout.size());
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Don't record deallocation
        unsafe { System.dealloc(ptr, layout) };
    }
}

// Our allocator set - use const fields for static initialization
pub struct AllocatorSet {
    task_aware: TaskAwareAllocator,
    system_alloc: System,
}

// Make the allocator set constructible in const contexts
impl AllocatorSet {
    pub const fn new() -> Self {
        Self {
            task_aware: TaskAwareAllocator,
            system_alloc: System,
        }
    }
}

impl Default for AllocatorSet {
    fn default() -> Self {
        Self::new()
    }
}

impl AllocatorProvider<AllocatorType> for AllocatorSet {
    fn get_allocator(&self, tag: AllocatorType) -> &dyn GlobalAlloc {
        match tag {
            AllocatorType::TaskAware => &self.task_aware,
            AllocatorType::SystemAlloc => &self.system_alloc,
        }
    }
}

// Concrete implementation for our specific types
unsafe impl GlobalAlloc for MultiAllocator<AllocatorType, AllocatorSet> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let tag = current_allocator();
        unsafe { self.allocate(layout, tag) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.deallocate(ptr, layout) }
    }
}

// Create a direct static instance
#[global_allocator]
static ALLOCATOR: MultiAllocator<AllocatorType, AllocatorSet> =
    MultiAllocator::new(AllocatorSet::new());
