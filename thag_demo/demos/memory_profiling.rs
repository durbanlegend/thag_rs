/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling", "demo"] }

[profile.release]
debug = true
strip = false
*/

/// Memory profiling demo - shows how to use thag_profiler for memory allocation tracking
/// This demo demonstrates memory profiling features of thag_profiler
//# Purpose: Demonstrate memory allocation tracking with thag_profiler
//# Categories: profiling, demo, memory
use std::collections::HashMap;
// use std::error::Error;
use thag_profiler::{enable_profiling, profiled, timing, visualization, AnalysisType, ProfileType};

#[timing]
#[profiled(mem_summary)]
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

#[timing]
#[profiled(mem_detail)]
fn process_strings_detail_profile() -> HashMap<String, usize> {
    let mut map = HashMap::new();

    for i in 0..1_000 {
        let key = format!("key_{}", i);
        let value = format!("value_{}_with_some_longer_content", i);
        map.insert(key, value.len());
    }

    map
}

#[timing]
#[profiled(mem_summary)]
fn memory_intensive_computation() -> Vec<String> {
    let mut results = Vec::new();

    // Simulate processing that creates many temporary allocations
    for i in 0..100 {
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

#[timing]
#[enable_profiling(memory, function(mem_summary))]
fn nested_allocations() {
    println!("Starting nested allocations...");

    let vectors = allocate_vectors();
    println!("Allocated {} vectors", vectors.len());

    let map = process_strings_detail_profile();
    println!("Created map with {} entries", map.len());

    let results = memory_intensive_computation();
    println!("Generated {} results", results.len());

    // All data structures will be deallocated when this function ends
}

async fn run_analysis() {
    // Interactive visualization: must run AFTER function with `enable_profiling` profiling attribute,
    // because profile output is only available after that function completes.
    if let Err(e) = visualization::show_interactive_prompt(
        "memory_profiling",
        &ProfileType::Memory,
        &AnalysisType::Flamegraph,
    )
    .await
    {
        eprintln!("⚠️ Could not show interactive memory visualization: {e}");
    }
}

fn main() {
    println!("🧠 Memory Profiling Demo");
    println!("{}", "═".repeat(23));
    println!();

    println!("Running memory-intensive operations with profiling...");
    nested_allocations();

    smol::block_on(run_analysis());

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated memory flamegraph files for allocation analysis.");
    println!("🔍 Use 'thag_profile' command to analyze memory usage patterns.");
    println!("💡 Notice the difference between mem_summary and mem_detail profiling.");

    // Ok(())
}
