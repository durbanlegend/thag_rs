use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

// Generic container for allocators
pub struct AllocatorContainer<T: GlobalAlloc>(pub T);

// This macro generates an enum with your custom allocators
// and defines the MultiAllocator with the specified allocators
#[macro_export]
macro_rules! define_allocators {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $default:ident = $default_impl:ty,
            $($variant:ident = $variant_impl:ty),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[repr(u8)]
        $vis enum $name {
            $default = 0, // Default is always 0
            $($variant),*
        }

        impl $name {
            const fn default() -> Self {
                Self::$default
            }
        }

        thread_local! {
            static CURRENT_ALLOCATOR: RefCell<$name> = RefCell::new($name::default());
        }

        $vis fn with_allocator<T, F: FnOnce() -> T>(allocator: $name, f: F) -> T {
            CURRENT_ALLOCATOR.with(|current| {
                let prev = *current.borrow();
                *current.borrow_mut() = allocator;

                let result = f();

                *current.borrow_mut() = prev;
                result
            })
        }

        $vis fn current_allocator() -> $name {
            CURRENT_ALLOCATOR.with(|current| *current.borrow())
        }

        // Create the actual allocator struct that holds all the allocator instances
        pub struct AllocatorSet {
            $default: AllocatorContainer<$default_impl>,
            $($variant: AllocatorContainer<$variant_impl>),*
        }

        impl AllocatorSet {
            pub const fn new() -> Self {
                Self {
                    $default: AllocatorContainer(<$default_impl>::new()),
                    $($variant: AllocatorContainer(<$variant_impl>::new())),*
                }
            }

            pub fn get_allocator(&self, tag: $name) -> &dyn GlobalAlloc {
                match tag {
                    $name::$default => &self.default.0,
                    $($name::$variant => &self.$variant.0),*
                }
            }
        }

        // Global allocator instance
        #[global_allocator]
        static ALLOCATOR: MultiAllocator<$name, AllocatorSet> = MultiAllocator::new();
    };
}

// Tagged pointer implementation for tracking allocations
struct TaggedAlloc<T: Copy + 'static> {
    ptr: NonNull<u8>,
    tag: T,
}

impl<T: Copy + 'static> TaggedAlloc<T> {
    fn new(ptr: NonNull<u8>, tag: T) -> Self {
        Self { ptr, tag }
    }

    fn ptr(&self) -> NonNull<u8> {
        self.ptr
    }

    fn tag(&self) -> T {
        self.tag
    }
}

// The main allocator struct - now fully generic
pub struct MultiAllocator<T, A>
where
    T: Copy + 'static,
    A: AllocatorProvider<T>,
{
    allocators: A,
    _marker: PhantomData<T>,
}

// Trait to access allocators by tag
pub trait AllocatorProvider<T: Copy + 'static> {
    fn get_allocator(&self, tag: T) -> &dyn GlobalAlloc;
}

impl<T, A> MultiAllocator<T, A>
where
    T: Copy + 'static,
    A: AllocatorProvider<T>,
{
    pub const fn new() -> Self {
        Self {
            allocators: A::new(),
            _marker: PhantomData,
        }
    }

    unsafe fn allocate(&self, layout: Layout, tag: T) -> *mut u8 {
        let allocator = self.allocators.get_allocator(tag);

        // Calculate the layout with space for our tag
        let (alloc_layout, offset) = self.layout_with_tag(layout);

        let ptr = allocator.alloc(alloc_layout);
        if ptr.is_null() {
            return ptr;
        }

        // Store the tag before the actual memory
        let tag_ptr = ptr as *mut T;
        *tag_ptr = tag;

        // Return the pointer to the actual memory (after the tag)
        ptr.add(offset)
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        // Calculate the layout with space for our tag
        let (alloc_layout, offset) = self.layout_with_tag(layout);

        // Get back to the original pointer (before the tag)
        let original_ptr = ptr.sub(offset);

        // Read the tag to know which allocator to use
        let tag_ptr = original_ptr as *const T;
        let tag = *tag_ptr;

        let allocator = self.allocators.get_allocator(tag);
        allocator.dealloc(original_ptr, alloc_layout);
    }

    fn layout_with_tag(&self, layout: Layout) -> (Layout, usize) {
        let tag_size = std::mem::size_of::<T>();
        let tag_align = std::mem::align_of::<T>();

        // Ensure the layout has at least the alignment of T
        let layout = layout.align_to(tag_align).expect("Failed to align layout");

        // Calculate a new layout with space for the tag
        Layout::from_size_align(
            layout.size() + tag_size,
            std::cmp::max(layout.align(), tag_align),
        )
        .map(|l| (l, tag_size))
        .expect("Failed to create layout with tag")
    }
}

unsafe impl<T, A> GlobalAlloc for MultiAllocator<T, A>
where
    T: Copy + 'static,
    A: AllocatorProvider<T>,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get the current allocator from the thread local
        let tag = current_allocator();
        self.allocate(layout, tag)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocate(ptr, layout)
    }
}

// Example usage with TaskAwareAllocator as default:

// Placeholder for your TaskAwareAllocator
pub struct TaskAwareAllocator;

impl TaskAwareAllocator {
    pub const fn new() -> Self {
        Self
    }
}

unsafe impl GlobalAlloc for TaskAwareAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

// Now define your allocators with TaskAwareAllocator as default
define_allocators! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum AllocatorType {
        TaskAware = TaskAwareAllocator,  // This is now the default
        System = System,
        // Add more allocators as needed
    }
}

// Usage in code
fn main() {
    // Use a specific allocator for a block of code
    let result = with_allocator(AllocatorType::System, || {
        // All allocations in this closure will use the System allocator
        let vec = vec![1, 2, 3];

        // Can return any value from the closure
        vec.len()
    });

    println!("Result: {}", result);

    // By default, TaskAwareAllocator will be used
    let vec = vec![1, 2, 3];

    // Check which allocator is currently being used
    let current = current_allocator();
    println!("Current allocator: {:?}", current); // Should print TaskAware
}
