/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Basic profiling demo - shows how to use thag_profiler for function timing
/// This demo demonstrates the core profiling features of thag_profiler
//# Purpose: Demonstrate basic time profiling with thag_profiler
//# Categories: profiling, demo, timing
use std::thread;
use std::time::Duration;
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn fibonacci_recursions() {
    for i in 20..25 {
        let result = fibonacci(i);
        println!("fibonacci({}) = {}", i, result);
    }
}

// For recursive functions, only time-profile the caller, to avoid
// unfixable multiple counting of elapsed time.
fn fibonacci(n: u32) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[profiled]
fn cpu_intensive_work() {
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum += i * i;
    }
    println!("CPU work result: {}", sum);
}

#[profiled]
fn simulated_io_work() {
    println!("Starting simulated I/O work...");
    thread::sleep(Duration::from_millis(100));
    println!("I/O work completed");
}

#[profiled]
fn nested_function_calls() {
    cpu_intensive_work();
    simulated_io_work();

    // Calculate some fibonacci numbers
    fibonacci_recursions();
}

#[enable_profiling(time)]
fn main() {
    println!("🔥 Basic Profiling Demo");
    println!("=====================");
    println!();

    println!("Running nested function calls with profiling...");
    nested_function_calls();

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated flamegraph files for visual analysis.");
    println!("🔍 Use 'thag_profile' command to analyze the profiling data.");
}
