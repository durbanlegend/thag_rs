/*[toml]
[dependencies]
dhat = { version = "0.3", optional = true }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler" }
tokio = { version = "1.36.0", features = ["rt-multi-thread", "macros", "time"], optional = true }
smol = { version = "2.0", optional = true }

[features]
dhat-heap = ["dep:dhat"]
full_profiling = ["thag_profiler/full_profiling", "thag_profiler/tls_allocator"]
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
use std::time::Duration;
use thag_profiler::{enable_profiling, mem_tracking, profiled};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[profiled]
async fn simple_async_work(id: usize) -> String {
    // Small allocation inside async function
    let data = vec![id; 100]; // 400-800 bytes depending on platform

    #[cfg(feature = "tokio-runtime")]
    tokio::time::sleep(Duration::from_millis(1)).await;

    #[cfg(feature = "smol-runtime")]
    smol::Timer::after(Duration::from_millis(1)).await;

    format!("Task {} completed with {} items", id, data.len())
}

#[profiled]
async fn spawn_many_tasks(count: usize) -> Vec<String> {
    println!("spawn_many_tasks called with {} tasks", count);
    println!("Each task should allocate ~400-800 bytes for vec![id; 100]");

    #[cfg(feature = "tokio-runtime")]
    println!("Plus tokio overhead per task");

    #[cfg(feature = "smol-runtime")]
    println!("Plus smol overhead per task");

    let mut handles = Vec::new();

    // Spawn multiple tasks to test async overhead
    for i in 0..count {
        #[cfg(feature = "tokio-runtime")]
        {
            let handle = tokio::spawn(simple_async_work(i));
            handles.push(handle);
        }

        #[cfg(feature = "smol-runtime")]
        {
            let handle = smol::spawn(simple_async_work(i));
            handles.push(handle);
        }
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        #[cfg(feature = "tokio-runtime")]
        {
            results.push(handle.await.unwrap());
        }

        #[cfg(feature = "smol-runtime")]
        {
            results.push(handle.await);
        }
    }

    results
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
    println!("=== Smol Async Memory Benchmark ===");

    println!();

    println!("Test 1: Spawning 20 simple async tasks with spawn_many_tasks(20).await");
    let results = spawn_many_tasks(20).await;
    println!("Completed {} tasks", results.len());
    println!();

    println!("Test 2: Async data processing (10 datasets of 1000 bytes each) with async_data_processing().await");
    let data_sets = async_data_processing().await;
    println!(
        "Processed {} datasets, total size: {} bytes",
        data_sets.len(),
        data_sets.iter().map(|ds| ds.len()).sum::<usize>()
    );
    println!();

    println!("Test 3: Sequential async work with simple_async_work(i).await for i in 0..5");
    for i in 0..5 {
        let result = simple_async_work(i).await;
        println!("Sequential task {}: {}", i, result.len());
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
