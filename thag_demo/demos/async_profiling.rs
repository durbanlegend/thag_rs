/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }

[profile.release]
debug = true
strip = false
*/

/// Async profiling demo - shows how to use thag_profiler with async/await functions
/// This demo demonstrates async profiling features of thag_profiler with tokio
//# Purpose: Demonstrate async function profiling with thag_profiler
//# Categories: profiling, demo, async, tokio
use std::time::Duration;
use thag_profiler::{enable_profiling, profiled};
use tokio::time::sleep;

#[profiled]
async fn simulate_database_query(query_id: u32) -> String {
    // Simulate database latency
    sleep(Duration::from_millis(50 + (query_id % 100) as u64)).await;
    format!("Result for query {}", query_id)
}

#[profiled]
async fn simulate_api_call(endpoint: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Simulate API call latency
    sleep(Duration::from_millis(100)).await;

    // Simulate occasional failures
    if endpoint.contains("flaky") {
        return Err("API temporarily unavailable".into());
    }

    Ok(format!("Response from {}", endpoint))
}

#[profiled]
async fn concurrent_operations() {
    println!("Starting concurrent operations...");

    // Run multiple database queries concurrently
    let queries = (1..=5).map(|i| simulate_database_query(i));
    let results = futures::future::join_all(queries).await;

    for (i, result) in results.iter().enumerate() {
        println!("Query {}: {}", i + 1, result);
    }
}

#[profiled]
async fn sequential_operations() {
    println!("Starting sequential operations...");

    // Run operations sequentially
    for i in 1..=3 {
        let result = simulate_database_query(i).await;
        println!("Sequential query {}: {}", i, result);
    }
}

#[profiled]
async fn mixed_async_operations() {
    println!("Starting mixed async operations...");

    // Mix of concurrent and sequential operations
    let api_future = simulate_api_call("user/profile");
    let db_future = simulate_database_query(100);

    // Wait for both to complete
    let (api_result, db_result) = tokio::join!(api_future, db_future);

    match api_result {
        Ok(response) => println!("API response: {}", response),
        Err(e) => println!("API error: {}", e),
    }

    println!("DB result: {}", db_result);

    // Try a flaky API call
    if let Err(e) = simulate_api_call("flaky/endpoint").await {
        println!("Expected error: {}", e);
    }
}

#[profiled]
async fn async_computation_heavy() {
    println!("Starting async computation-heavy work...");

    // Simulate CPU-intensive work that yields periodically
    for i in 0..5 {
        let mut sum = 0u64;
        for j in 0..1_000_000 {
            sum += (i * 1_000_000 + j) as u64;
        }

        // Yield to allow other tasks to run
        tokio::task::yield_now().await;

        println!("Batch {} completed, sum: {}", i, sum);

        // Small delay between batches
        sleep(Duration::from_millis(10)).await;
    }
}

#[profiled]
async fn demonstrate_async_profiling() {
    // Run different types of async operations
    concurrent_operations().await;
    sequential_operations().await;
    mixed_async_operations().await;
    async_computation_heavy().await;
}

#[tokio::main]
#[enable_profiling(time)]
async fn main() {
    println!("🚀 Async Profiling Demo");
    println!("=======================");
    println!();

    println!("Running async operations with profiling...");
    demonstrate_async_profiling().await;

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated flamegraph files for async operation analysis.");
    println!("🔍 Use 'thag_profile' command to analyze async execution patterns.");
    println!("⚡ Notice how concurrent vs sequential operations appear in the flamegraph.");
}
