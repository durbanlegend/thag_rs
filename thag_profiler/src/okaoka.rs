#![deny(unsafe_op_in_unsafe_fn)]

/// From <https://github.com/emi0x7d1/okaoka/tree/trunk>, no licence
use std::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    marker::PhantomData,
};

thread_local! {
    static ALLOCATOR_TAG: UnsafeCell<u8> = const { UnsafeCell::new(0) };
}

// #[inline(always)]
fn get_allocator_tag() -> u8 {
    ALLOCATOR_TAG.with(|tag| unsafe { *tag.get() })
}

// #[inline(always)]
fn set_allocator_tag(new_tag: u8) {
    ALLOCATOR_TAG.with(|tag| unsafe { *tag.get() = new_tag });
}

/// Allocator that allows you to use multiple allocators and switch between them at runtime
///
/// It uses a hidden tag to keep track of which allocator was used so that it can use the same
/// allocator for deallocation.
///
/// The hidden tag is put before any allocation. The following diagram shows the memory layout:
/// -------------------
/// | Tag | Data .... |
/// -------------------
///       ^---- we return a pointer to this address
pub struct MultiAllocator<T>(PhantomData<T>);

impl<T> MultiAllocator<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

unsafe impl<Backend> GlobalAlloc for MultiAllocator<Backend>
where
    Backend: MultiAllocatorBackend,
{
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        // Make the tag size the same size as the alignment so that
        // we can keep the same alignment for the data.
        let tag_size = layout.align();
        let new_layout =
            unsafe { Layout::from_size_align_unchecked(layout.size() + tag_size, layout.align()) };
        let allocator_tag = get_allocator_tag();
        let ptr = unsafe { Backend::alloc(allocator_tag.into(), new_layout) };
        // Write the allocator tag to the tag address
        unsafe { std::ptr::write(ptr, allocator_tag) };
        // Return a pointer to the address just after the tag
        unsafe { ptr.add(tag_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        let tag_size = layout.align();
        // Subtract `tag_size` to get the original pointer
        let new_ptr = unsafe { ptr.sub(tag_size) };
        // Re-construct the layout with `tag_size`
        let new_layout =
            unsafe { Layout::from_size_align_unchecked(layout.size() + tag_size, layout.align()) };
        // Check the allocator tag used for this allocation
        let tag = unsafe { std::ptr::read(new_ptr) };

        unsafe {
            Backend::dealloc(tag.into(), new_ptr, new_layout);
        }
    }
}

pub trait MultiAllocatorBackend {
    type Tag: Copy + Into<u8> + From<u8>;

    unsafe fn alloc(tag: Self::Tag, layout: Layout) -> *mut u8;
    unsafe fn dealloc(tag: Self::Tag, ptr: *mut u8, layout: Layout);
}

#[macro_export]
macro_rules! create_multi_allocator_backend {
    ($name:ident, $enum_name:ident, $($tag_name:ident => $allocator_name:ident),+$(,)?) => {
        #[derive(Copy, Clone)]
        #[repr(u8)]
        enum $enum_name {
            $($tag_name),+
            ,__END,
        }

        impl From<u8> for $enum_name {
            fn from(raw_tag: u8) -> Self {
                assert!(raw_tag < $enum_name::__END as u8);
                unsafe { std::mem::transmute(raw_tag) }
            }
        }

        impl From<$enum_name> for u8 {
            fn from(tag: $enum_name) -> Self {
                tag as u8
            }
        }

        struct $name;

        impl okaoka::MultiAllocatorBackend for $name {
            type Tag = $enum_name;

            #[inline(always)]
            unsafe fn alloc(tag: Self::Tag, layout: std::alloc::Layout) -> *mut u8 {
                use std::alloc::GlobalAlloc;
                match tag {
                    $($enum_name::$tag_name => $allocator_name.alloc(layout)),+
                    ,$enum_name::__END => unreachable!(),
                }
            }

            #[inline(always)]
            unsafe fn dealloc(tag: Self::Tag, ptr: *mut u8, layout: std::alloc::Layout) {
                use std::alloc::GlobalAlloc;
                match tag {
                    $($enum_name::$tag_name => $allocator_name.dealloc(ptr, layout)),+
                    ,$enum_name::__END => unreachable!(),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! set_multi_global_allocator {
    ($name:ident, $enum_name:ident, $($tag_name:ident => $allocator_name:ident),+$(,)?) => {
        $crate::create_multi_allocator_backend!{
            $name,
            $enum_name,
            $($tag_name => $allocator_name),+
        }

        #[global_allocator]
        static ALLOCATOR: okaoka::MultiAllocator<$name> = okaoka::MultiAllocator::new();

        impl $name {
            pub fn with(tag: <$name as okaoka::MultiAllocatorBackend>::Tag, closure: impl FnMut()) {
                okaoka::with_allocator(tag.into(), closure);
            }
        }
    };
}

/// Set the given allocator inside the closure, restoring the previous allocator after returning
///
/// # Example
///
/// ```rust
/// with_allocator(AllocatorTag::Jemalloc as u8, || {
///   // jemalloc is the default allocator inside this closure
/// });
/// // The previous allocator is restored here
/// ```
///
/// If `allocator_tag` is not a valid tag for the current allocator backend, the allocator will
/// panic during allocation.
pub fn with_allocator(allocator_tag: u8, mut closure: impl FnMut()) {
    let old_tag = get_allocator_tag();
    set_allocator_tag(allocator_tag);
    closure();
    set_allocator_tag(old_tag);
}
