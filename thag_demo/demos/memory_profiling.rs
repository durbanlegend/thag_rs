/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Memory profiling demo - shows how to use thag_profiler for memory allocation tracking
/// This demo demonstrates memory profiling features of thag_profiler
//# Purpose: Demonstrate memory allocation tracking with thag_profiler
//# Categories: profiling, demo, memory
use std::collections::HashMap;
use thag_profiler::{enable_profiling, profiled};

#[profiled(mem_detail)]
fn allocate_vectors() -> Vec<Vec<u64>> {
    let mut outer = Vec::new();

    for i in 0..100 {
        let mut inner = Vec::with_capacity(1000);
        for j in 0..1000 {
            inner.push(i * 1000 + j);
        }
        outer.push(inner);
    }

    outer
}

#[profiled(mem_summary)]
fn process_strings() -> HashMap<String, usize> {
    let mut map = HashMap::new();

    for i in 0..10_000 {
        let key = format!("key_{}", i);
        let value = format!("value_{}_with_some_longer_content", i);
        map.insert(key, value.len());
    }

    map
}

#[profiled(mem_detail)]
fn memory_intensive_computation() -> Vec<String> {
    let mut results = Vec::new();

    // Simulate processing that creates many temporary allocations
    for i in 0..1000 {
        let mut temp = String::new();
        for j in 0..100 {
            temp.push_str(&format!("item_{}_{} ", i, j));
        }

        // Keep only every 10th result to show deallocation
        if i % 10 == 0 {
            results.push(temp);
        }
        // temp is dropped here for other iterations
    }

    results
}

#[profiled(mem_summary)]
fn nested_allocations() {
    println!("Starting nested allocations...");

    let vectors = allocate_vectors();
    println!("Allocated {} vectors", vectors.len());

    let map = process_strings();
    println!("Created map with {} entries", map.len());

    let results = memory_intensive_computation();
    println!("Generated {} results", results.len());

    // All data structures will be deallocated when this function ends
}

#[enable_profiling(memory)]
fn main() {
    println!("🧠 Memory Profiling Demo");
    println!("========================");
    println!();

    println!("Running memory-intensive operations with profiling...");
    nested_allocations();

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated memory flamegraph files for allocation analysis.");
    println!("🔍 Use 'thag_profile' command to analyze memory usage patterns.");
    println!("💡 Notice the difference between memory_summary and memory_detail profiling.");
}
