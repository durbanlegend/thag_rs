/*[toml]
[dependencies]
okaoka = { git = "https://github.com/emi0x7d1/okaoka" }
*/
use jemallocator::Jemalloc;
use std::alloc::System;

/// Allocator that doesn't deallocate
struct LeakingAllocator;

unsafe impl std::alloc::GlobalAlloc for LeakingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        Jemalloc.alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: std::alloc::Layout) {}
}

okaoka::set_multi_global_allocator! {
    GlobalAllocator, // Name of our allocator facade
    AllocatorTag, // Name of our allocator tag enum
    System => System,
    Jemalloc => Jemalloc,
    Leaking => LeakingAllocator,
}

fn main() {
    let mut x = Box::new(10); // Allocated with the default allocator
    dbg!(&*x as *const i32);
    GlobalAllocator::with(AllocatorTag::Leaking, || {
        x = Box::new(20); // Allocated with the leaking allocator
    }); // Previous allocator restored
    dbg!(&*x as *const i32);
}
