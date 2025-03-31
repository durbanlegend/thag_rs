use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;

/// Claude-generated multi-allocator prototype for use in thag_profiler.
/// This uses the tagging technique demonstrated by the unlicensed (as at time of writing)
/// `okaoka` crate at `https://github.com/emi0x7d1/okaoka` and found via
/// `https://www.reddit.com/r/rust/comments/12di5bo/custom_allocators_in_rust/`.
///
//# Purpose: Prototype using separate memory allocators for user and profiler code for use by `thag_profiler` for memory profiling.
//# Categories: prototype, technique
// Trait to access allocators by tag
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

        let ptr = allocator.alloc(alloc_layout);
        if ptr.is_null() {
            return ptr;
        }

        // Store the tag at the beginning of the allocated memory
        // We'll use memcpy to avoid alignment issues
        let tag_ptr = ptr as *mut T;
        std::ptr::write_unaligned(tag_ptr, tag);

        // Return the adjusted pointer (after the tag)
        ptr.add(offset)
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        // Calculate the layout with space for our tag
        let (alloc_layout, offset) = Self::layout_with_tag(layout);

        // Get back to the original pointer (before the tag)
        let original_ptr = ptr.sub(offset);

        // Read the tag to know which allocator to use
        // Use unaligned read to avoid alignment issues
        let tag_ptr = original_ptr as *const T;
        let tag = std::ptr::read_unaligned(tag_ptr);

        let allocator = self.allocators.get_allocator(tag);
        allocator.dealloc(original_ptr, alloc_layout);
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
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum AllocatorType {
    TaskAware = 0, // Default is 0
    SystemAlloc,
}

// Thread-local current allocator
thread_local! {
    static CURRENT_ALLOCATOR: RefCell<AllocatorType> = RefCell::new(AllocatorType::TaskAware);
}

// Function to get current allocator
pub fn current_allocator() -> AllocatorType {
    CURRENT_ALLOCATOR.with(|current| *current.borrow())
}

// Function to run code with a specific allocator
pub fn with_allocator<T, F: FnOnce() -> T>(allocator: AllocatorType, f: F) -> T {
    CURRENT_ALLOCATOR.with(|current| {
        let prev = *current.borrow();
        *current.borrow_mut() = allocator;

        let result = f();

        *current.borrow_mut() = prev;
        result
    })
}

// Placeholder for your TaskAwareAllocator
pub struct TaskAwareAllocator;

unsafe impl GlobalAlloc for TaskAwareAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
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
        self.allocate(layout, tag)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocate(ptr, layout)
    }
}

// Create a direct static instance
#[global_allocator]
static ALLOCATOR: MultiAllocator<AllocatorType, AllocatorSet> =
    MultiAllocator::new(AllocatorSet::new());

// Usage example
fn main() {
    // Use a specific allocator for a block of code
    let result = with_allocator(AllocatorType::SystemAlloc, || {
        // All allocations in this closure will use the System allocator
        let vec = vec![1, 2, 3];

        // Check which allocator is currently being used
        let current = current_allocator();
        println!("Current allocator: {:?}", current); // Should print TaskAware
        assert_eq!(current, AllocatorType::SystemAlloc);

        // Can return any value from the closure
        // vec.len()
        vec
    });

    println!("Result: {:?}", result);

    // By default, TaskAwareAllocator will be used
    let _vec = vec![1, 2, 3];

    // Check which allocator is currently being used
    let current = current_allocator();
    println!("Current allocator: {:?}", current); // Should print TaskAware
    assert_eq!(current, AllocatorType::TaskAware);
}
