/*[toml]
[dependencies]
dhat = { version = "0.3", optional = true }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler" }

[features]
dhat-heap = ["dep:dhat"]
full_profiling = ["thag_profiler/full_profiling"]
default = []
*/

/// Benchmark comparison between thag_profiler and dhat-rs for memory profiling accuracy.
/// This creates known allocation patterns and compares the results from both profilers.
//# Purpose: Validate thag_profiler accuracy against dhat-rs reference implementation
//# Categories: benchmark, profiling
use std::collections::HashMap;
use thag_profiler::{enable_profiling, profiled};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// Test 1: Allocating 1000 vectors of 1024 bytes each
#[profiled]
fn allocate_vectors(count: usize, size: usize) -> Vec<Vec<u8>> {
    let mut vectors = Vec::new();
    for i in 0..count {
        let mut vec = Vec::with_capacity(size);
        // Fill with data to ensure actual allocation
        for j in 0..size {
            vec.push((i + j) as u8);
        }
        vectors.push(vec);
    }
    vectors
}

// Test 2: Allocating HashMap with 500 entries (800 bytes each)
#[profiled]
fn allocate_hashmap(entries: usize) -> HashMap<String, Vec<u64>> {
    let mut map = HashMap::new();
    for i in 0..entries {
        let key = format!("key_{}", i);
        let value = vec![i as u64; 100]; // 100 u64s per entry
        map.insert(key, value);
    }
    map
}

// Test 3: Allocate and deallocate in loop (2000 iterations)
#[profiled]
fn allocate_and_deallocate(iterations: usize) {
    for i in 0..iterations {
        let size = 1000 + (i % 1000);
        let _temp: Vec<u64> = (0..size).map(|x| x as u64).collect();
        // temp is dropped here
    }
}

// Test 4: Nested data structures
#[profiled]
fn nested_allocations() -> Vec<Vec<Vec<String>>> {
    let mut outer = Vec::new();
    for i in 0..10 {
        let mut middle = Vec::new();
        for j in 0..20 {
            let mut inner = Vec::new();
            for k in 0..30 {
                inner.push(format!("string_{}_{}__{}", i, j, k));
            }
            middle.push(inner);
        }
        outer.push(middle);
    }
    outer
}

fn main() {
    run_profiling();

    // println!("=== Profiling Results ===");

    // println!("\nthag_profiler results should be displayed above.");
    // println!("dhat results will be in dhat-heap.json (if dhat feature enabled)");
    // println!("\nTo compare:");
    // println!("1. Run with: cargo run --features dhat-heap");
    // println!("2. Check dhat-heap.json for dhat results");
    // println!("3. Compare peak memory usage, total allocations, etc.");
    // println!("\nExpected approximate allocations:");
    // println!("- Test 1: ~1MB (1000 * 1024 bytes)");
    // println!("- Test 2: ~400KB (500 * 800 bytes)");
    // println!("- Test 3: Variable (temporary allocations)");
    // println!("- Test 4: Variable (nested strings and vectors)");
}

#[enable_profiling(memory)]
fn run_profiling() {
    // Initialize dhat if feature is enabled
    #[cfg(feature = "dhat-heap")]
    let _dhat = dhat::Profiler::new_heap();

    println!("=== Memory Profiling Comparison: thag_profiler vs dhat-rs ===\n");

    println!("Test 1: Allocating 1000 vectors of 1024 bytes each");
    let vectors = allocate_vectors(1000, 1024);
    println!("Allocated {} vectors", vectors.len());
    drop(vectors);
    println!("Vectors deallocated\n");

    println!("Test 2: Allocating HashMap with 500 entries (800 bytes each)");
    let map = allocate_hashmap(500);
    println!("HashMap has {} entries", map.len());
    drop(map);
    println!("HashMap deallocated\n");

    println!("Test 3: Allocate and deallocate in loop (2000 iterations)");
    allocate_and_deallocate(2000);
    println!("Allocation/deallocation loop completed\n");

    println!("Test 4: Nested data structures");
    let nested = nested_allocations();
    println!(
        "Nested structure created: {}x{}x{} elements",
        nested.len(),
        nested.get(0).map_or(0, |v| v.len()),
        nested.get(0).and_then(|v| v.get(0)).map_or(0, |v| v.len())
    );
    drop(nested);
    println!("Nested structure deallocated\n");
}
