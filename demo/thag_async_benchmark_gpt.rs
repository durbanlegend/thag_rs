/*[toml]
[dependencies]
dhat = { version = "0.3", optional = true }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler" }
tokio = { version = "1.36.0", features = ["rt-multi-thread", "macros", "time"], optional = true }
smol = { version = "2.0", optional = true }

[features]
dhat-heap = ["dep:dhat"]
full_profiling = ["thag_profiler/full_profiling"]
tokio-runtime = ["dep:tokio"]
smol-runtime = ["dep:smol"]
default = []

[profile.release]
debug-assertions = true
debug = true
strip = false
*/

/// Focused async benchmark comparing tokio vs smol memory profiling with thag_profiler vs dhat-rs.
/// Tests async runtime overhead and task spawning memory usage.
///
/// # Test with tokio + thag_profiler
/// thag --features 'full_profiling,tokio-runtime' tools/thag_async_benchmark.rs -tfm
///
/// # Test with tokio + dhat
/// thag --features 'dhat-heap,tokio-runtime' tools/thag_async_benchmark.rs -tfm
///
/// # Test with smol + thag_profiler
/// thag --features 'full_profiling,smol-runtime' tools/thag_async_benchmark.rs -tfm
///
/// # Test with smol + dhat
/// thag --features 'dhat-heap,smol-runtime' tools/thag_async_benchmark.rs -tfm
//# Purpose: Validate async memory profiling accuracy across different runtimes
//# Categories: async, benchmark, profiling
use futures::future::join_all;
use rand::{rng, Rng};
use std::time::Duration;
use thag_profiler::{enable_profiling, profiled};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[profiled]

async fn simple_async_work(i: usize) -> usize {
    let mut data = vec![0u8; 1000];
    data[i % 1000] = 1;

    let delay = rng().random_range(1..5); // Random delay between 1-4ms
    tokio::time::sleep(Duration::from_millis(delay)).await;

    data.iter().map(|&b| b as usize).sum()
}

#[profiled]
async fn spawn_many_tasks() -> usize {
    let mut tasks = Vec::with_capacity(20);
    for i in 0..20 {
        tasks.push(tokio::spawn(simple_async_work(i)));
    }

    // Await all concurrently — this encourages better scheduling
    let results = join_all(tasks).await;

    // Sum up the outputs, handling any panics
    results
        .into_iter()
        .map(|r| r.unwrap_or(0)) // handle join errors gracefully
        .sum()
}

#[profiled]
async fn async_data_processing() -> Vec<Vec<u8>> {
    let mut data_sets = Vec::new();

    for i in 0..10 {
        let mut data = Vec::with_capacity(1000);
        for j in 0..1000 {
            data.push((i + j) as u8);
        }

        // Async yield point
        #[cfg(feature = "tokio-runtime")]
        tokio::task::yield_now().await;

        #[cfg(feature = "smol-runtime")]
        smol::future::yield_now().await;

        data_sets.push(data);
    }

    data_sets
}

#[cfg(feature = "tokio-runtime")]
#[tokio::main]
#[enable_profiling(memory)]
async fn main() {
    run_benchmark().await;
}

#[cfg(feature = "smol-runtime")]
#[enable_profiling(memory)]
fn main() {
    smol::block_on(run_benchmark());
}

#[cfg(not(any(feature = "tokio-runtime", feature = "smol-runtime")))]
#[enable_profiling(runtime)]
fn main() {
    println!("Please run with either --features tokio-runtime or --features smol-runtime");
    println!("Examples:");
    println!("  thag --features 'full_profiling,tokio-runtime' tools/thag_async_benchmark.rs");
    println!("  thag --features 'full_profiling,smol-runtime' tools/thag_async_benchmark.rs");
    println!("  thag --features 'dhat-heap,tokio-runtime' tools/thag_async_benchmark.rs");
}

async fn run_benchmark() {
    #[cfg(feature = "dhat-heap")]
    let _dhat = dhat::Profiler::new_heap();

    #[cfg(feature = "tokio-runtime")]
    println!("=== Tokio Async Memory Benchmark ===");

    #[cfg(feature = "smol-runtime")]
    println!("=== Smol Async Memory Benchmark ===\n");

    println!("Test 1: Spawning 20 simple async tasks");
    let tasks = spawn_many_tasks().await;
    println!("Completed {tasks} tasks\n");

    println!("Test 2: Async data processing (10 datasets of 1000 bytes each)");
    let data_sets = async_data_processing().await;
    println!(
        "Processed {} datasets, total size: {} bytes",
        data_sets.len(),
        data_sets.iter().map(|ds| ds.len()).sum::<usize>()
    );
    println!();

    println!("Test 3: Sequential async work");
    for i in 0..5 {
        let result = simple_async_work(i).await;
        println!("Sequential task {i}: {result}");
    }
    println!();

    #[cfg(feature = "tokio-runtime")]
    println!("Tokio benchmark completed. Check profiling output for memory usage.");

    #[cfg(feature = "smol-runtime")]
    println!("Smol benchmark completed. Check profiling output for memory usage.");

    println!();
    println!("Expected allocations:");
    println!("- Test 1: ~20 tasks × ~400-800 bytes + async runtime overhead");
    println!("- Test 2: 10 × 1000 = 10,000 bytes + async state machines");
    println!("- Test 3: 5 × ~400-800 bytes + sequential async overhead");
    println!();
    println!("Compare thag_profiler results with dhat-heap.json");
    println!("Note: Async runtimes have significant infrastructure overhead");
    println!("that thag_profiler captures but dhat may filter out.");
}
