/*[toml]
[dependencies]
thag_profiler = { version = "1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Test recursion detection - shows how thag_profiler handles recursive functions
/// This demo tests the recursion detection feature to prevent exponential profiling overhead
//# Purpose: Test recursion detection in thag_profiler
//# Categories: profiling, demo, recursion, testing
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn recursive_fibonacci(n: u32) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        recursive_fibonacci(n - 1) + recursive_fibonacci(n - 2)
    }
}

#[profiled]
fn non_recursive_work() {
    let mut sum = 0u64;
    for i in 0..100_000 {
        sum += i * i;
    }
    println!("Non-recursive work result: {}", sum);
}

#[profiled]
fn test_recursion_detection() {
    println!("Testing recursion detection...");

    // This should only profile the outermost call
    let result = recursive_fibonacci(25);
    println!("Recursive fibonacci(25) = {}", result);

    // This should profile normally
    non_recursive_work();
}

#[enable_profiling(time)]
fn main() {
    println!("🔄 Recursion Detection Test");
    println!("{}", "═".repeat(26));
    println!();

    println!("This demo tests the recursion detection feature.");
    println!("Only the outermost recursive call should be profiled.");
    println!();

    test_recursion_detection();

    println!();
    println!("✅ Test completed!");
    println!("📊 Check the profile files - should show minimal data due to recursion detection.");
    println!("🔍 Only the outermost recursive call should appear in the profile.");
}
