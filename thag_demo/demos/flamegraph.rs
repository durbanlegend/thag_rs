/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }
rayon = "1.0"

[profile.release]
debug = true
strip = false
*/

/// Flamegraph demo - shows how to use thag_profiler to generate interactive flamegraphs
/// This demo demonstrates flamegraph generation and analysis features of thag_profiler
//# Purpose: Demonstrate flamegraph generation and analysis with thag_profiler
//# Categories: profiling, demo, flamegraph, visualization
use rayon::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn level_1_function() {
    level_2_function_a();
    level_2_function_b();
    level_2_function_c();
}

#[profiled]
fn level_2_function_a() {
    level_3_function_a1();
    level_3_function_a2();
}

#[profiled]
fn level_2_function_b() {
    // This function takes longer - will show as a wider bar
    thread::sleep(Duration::from_millis(50));
    level_3_function_b1();
}

#[profiled]
fn level_2_function_c() {
    // Parallel processing - will show interesting patterns
    let data: Vec<i32> = (0..1000).collect();
    let _results: Vec<i32> = data.par_iter().map(|x| expensive_computation(*x)).collect();
}

#[profiled]
fn level_3_function_a1() {
    // Quick function
    let _sum: i32 = (0..1000).sum();
}

#[profiled]
fn level_3_function_a2() {
    // Slightly slower function
    let mut map = HashMap::new();
    for i in 0..500 {
        map.insert(i, i * 2);
    }
}

#[profiled]
fn level_3_function_b1() {
    // Moderate duration function
    thread::sleep(Duration::from_millis(20));
    let _fibonacci = compute_fibonacci(30);
}

#[profiled]
fn expensive_computation(n: i32) -> i32 {
    // This will be called many times in parallel
    let mut result = n;
    for _ in 0..100 {
        result = result.wrapping_mul(2).wrapping_add(1);
    }
    result
}

#[profiled]
fn compute_fibonacci(n: u32) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        compute_fibonacci(n - 1) + compute_fibonacci(n - 2)
    }
}

#[profiled]
fn recursive_tree_builder(depth: u32) -> Vec<u32> {
    if depth == 0 {
        vec![1]
    } else {
        let mut result = vec![depth];
        let left = recursive_tree_builder(depth - 1);
        let right = recursive_tree_builder(depth - 1);
        result.extend(left);
        result.extend(right);
        result
    }
}

#[profiled]
fn demonstrate_call_patterns() {
    println!("Creating nested call patterns for flamegraph visualization...");

    // This will create a deep call stack
    level_1_function();

    // This will create a wide, recursive pattern
    let _tree = recursive_tree_builder(8);

    // This will show repeated calls to the same function
    for i in 0..10 {
        level_3_function_a1();
        if i % 2 == 0 {
            level_3_function_a2();
        }
    }
}

#[profiled]
fn cpu_intensive_section() {
    println!("Running CPU-intensive section...");

    // This will show up as a prominent block in the flamegraph
    let mut matrix = vec![vec![0.0; 100]; 100];
    for i in 0..100 {
        for j in 0..100 {
            matrix[i][j] = (i * j) as f64 / 10.0;
            // Add some computation to make it visible
            matrix[i][j] = matrix[i][j].sin().cos().tan();
        }
    }

    println!("CPU-intensive section completed");
}

#[profiled]
fn io_simulation_section() {
    println!("Running I/O simulation section...");

    // This will show up as a distinct block with different characteristics
    for i in 0..5 {
        println!("Simulating I/O operation {}", i + 1);
        thread::sleep(Duration::from_millis(30));
    }

    println!("I/O simulation section completed");
}

#[profiled]
fn mixed_workload_section() {
    println!("Running mixed workload section...");

    // Alternating CPU and I/O work
    for i in 0..3 {
        // CPU work
        let _sum: u64 = (0..100_000).map(|x| x as u64 * x as u64).sum();

        // I/O work
        thread::sleep(Duration::from_millis(10));

        println!("Mixed workload iteration {} completed", i + 1);
    }
}

#[profiled]
fn demonstrate_flamegraph_features() {
    println!("🔥 Demonstrating flamegraph features...");
    println!();

    // Each of these sections will appear as distinct regions in the flamegraph
    demonstrate_call_patterns();
    cpu_intensive_section();
    io_simulation_section();
    mixed_workload_section();

    println!("All flamegraph demonstration sections completed!");
}

#[enable_profiling(time)]
fn main() {
    println!("🔥 Flamegraph Generation Demo");
    println!("=============================");
    println!();

    println!("This demo creates various call patterns to showcase flamegraph visualization.");
    println!("Each function will appear as a colored bar in the flamegraph.");
    println!("Bar width = time spent, bar height = call stack depth.");
    println!();

    demonstrate_flamegraph_features();

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated flamegraph files (.svg) for interactive visualization.");
    println!("🔍 Use 'thag_profile' command to analyze and filter the flamegraph data.");
    println!("💡 Flamegraph reading tips:");
    println!("   • Wide bars = slow functions (hotspots)");
    println!("   • Narrow bars = fast functions");
    println!("   • Stack height = call depth");
    println!("   • Click on bars to zoom in");
    println!("   • Use search to find specific functions");
    println!("🎯 Look for the CPU-intensive section as the widest bar!");
}
