/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Minimal recursion test - the simplest possible test to check recursion detection
//# Purpose: Test recursion detection with minimal overhead
//# Categories: profiling, demo, recursion, testing
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn simple_recursive(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        simple_recursive(n - 1) + 1
    }
}

#[enable_profiling(time)]
fn main() {
    println!("🔄 Minimal Recursion Test");
    println!("=========================");

    let result = simple_recursive(5);
    println!("Result: {}", result);

    println!("✅ Test completed!");
}
