/*[toml]
[dependencies]
dhat = { version = "0.3", optional = true }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler" }

[features]
dhat-heap = ["dep:dhat"]
full_profiling = ["thag_profiler/full_profiling", "thag_profiler/tls_allocator"]
default = []
*/

/// Precise validation test with exactly known allocation sizes to verify
/// thag_profiler accuracy and understand differences with dhat-rs.
//# Purpose: Validate profiler accuracy with precisely measurable allocations
//# Categories: profiling, testing
use std::mem;
use thag_profiler::{enable_profiling, profiled};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[profiled]
fn allocate_exact_vec() -> Vec<u64> {
    // Exactly 1000 u64s = 8000 bytes of data
    // Vec overhead: capacity (8) + length (8) + pointer (8) = 24 bytes on heap metadata
    vec![42u64; 1000]
}

#[profiled]
fn allocate_with_capacity() -> Vec<u64> {
    // Pre-allocate capacity to avoid reallocations
    let mut vec = Vec::with_capacity(1000);
    for i in 0..1000 {
        vec.push(i);
    }
    vec
}

#[profiled]
fn allocate_single_string() -> String {
    // Exactly 100 characters = 100 bytes of UTF-8 data
    "A".repeat(100)
}

#[profiled]
fn allocate_box_array() -> Box<[u64; 1000]> {
    // Exactly 8000 bytes on heap, no Vec overhead
    Box::new([42u64; 1000])
}

#[profiled]
fn allocate_multiple_small() -> Vec<Box<u64>> {
    // 100 separate heap allocations, each 8 bytes
    let mut boxes = Vec::new();
    for i in 0..100 {
        boxes.push(Box::new(i));
    }
    boxes
}

fn print_size_info() {
    println!("=== Size Information ===");
    println!("u64 size: {} bytes", mem::size_of::<u64>());
    println!("Vec<u64> stack size: {} bytes", mem::size_of::<Vec<u64>>());
    println!("Box<u64> stack size: {} bytes", mem::size_of::<Box<u64>>());
    println!("String stack size: {} bytes", mem::size_of::<String>());
    println!(
        "Box<[u64; 1000]> stack size: {} bytes",
        mem::size_of::<Box<[u64; 1000]>>()
    );
    println!();
}

#[enable_profiling(runtime)]
fn main() {
    #[cfg(feature = "dhat-heap")]
    let _dhat = dhat::Profiler::new_heap();

    print_size_info();

    println!("=== Precise Allocation Validation ===");
    println!();

    println!("Test 1: vec![42u64; 1000] - exactly 8000 bytes data");
    let vec1 = allocate_exact_vec();
    println!("Expected: 8000 bytes minimum (+ Vec metadata)");
    println!("Vec length: {}, capacity: {}", vec1.len(), vec1.capacity());
    drop(vec1);
    println!();

    println!("Test 2: Vec::with_capacity(1000) then push - should be identical to test 1");
    let vec2 = allocate_with_capacity();
    println!("Expected: 8000 bytes (+ Vec metadata)");
    println!("Vec length: {}, capacity: {}", vec2.len(), vec2.capacity());
    drop(vec2);
    println!();

    println!("Test 3: String with exactly 100 characters");
    let string = allocate_single_string();
    println!("Expected: 100 bytes minimum (+ String metadata)");
    println!("String length: {} bytes", string.len());
    drop(string);
    println!();

    println!("Test 4: Box<[u64; 1000]> - exactly 8000 bytes, no Vec overhead");
    let boxed = allocate_box_array();
    println!("Expected: exactly 8000 bytes (no Vec metadata)");
    println!("Array size: {} elements", boxed.len());
    drop(boxed);
    println!();

    println!("Test 5: 100 separate Box<u64> allocations");
    let boxes = allocate_multiple_small();
    println!("Expected: 100 * 8 = 800 bytes (+ Vec for holding Box pointers)");
    println!("Number of boxes: {}", boxes.len());
    drop(boxes);
    println!();

    println!("=== Analysis Guide ===");
    println!("Compare results with dhat-heap.json:");
    println!("1. Test 1 vs Test 2 should be identical (same final allocation)");
    println!("2. Test 4 should be closest to exactly 8000 bytes");
    println!("3. Test 5 should show exactly 800 bytes for the u64s");
    println!("4. Differences show what overhead thag_profiler captures");
    println!();
    println!("If thag_profiler shows more than expected:");
    println!("- Likely capturing allocator metadata, alignment, capacity rounding");
    println!("- This is MORE accurate, not less accurate");
    println!("- Real programs have this overhead too");
}
