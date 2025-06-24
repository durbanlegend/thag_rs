/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling"] }
*/

/// This script demonstrates the usage of the `warn_once` pattern for suppressing repeated
/// log messages with minimal runtime overhead.
///
/// The dependency is `thag_profiler` because that's the only place it's used at time of writing, even though this is
/// not in any way a profiling-specific function.
///
/// Disclosure: the `thag_profiler` `warn_once` macro and `warn_once_with_id` function use unsafe code.
///
/// Credit to `Claude 3.7 Sonnet`.
//# Purpose: Demo a macro I found useful, explained and benchmarked here in great detail thanks to Claude.
//# Categories: demo, macros, technique
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Import the warn_once macro from thag_profiler
use thag_profiler::warn_once;

// Counter for tracking warning calls
static WARNING_COUNTER: Mutex<u32> = Mutex::new(0);

// Mock debug_log function for our demo
fn debug_log(msg: &str) {
    println!("[LOG] {}", msg);

    // Update the warning counter when certain messages are logged
    if msg.contains("This warning should only appear once") {
        let mut counter = WARNING_COUNTER.lock().unwrap();
        *counter += 1;
    }
}

/// Simple example that shows a warning only once despite multiple calls
fn demo_simple_warning() {
    println!("\n=== Simple Warning Suppression ===\n");

    // This condition would normally trigger the warning every time
    let condition = true;

    println!("Calling function 5 times...");
    for i in 1..=5 {
        println!("\nCall #{}: ", i);

        // Use warn_once to only show the warning on first occurrence
        warn_once!(condition, || {
            debug_log("This warning should only appear once");
        });

        // This will always execute
        println!("Function work completed");
    }

    // Check how many times the warning was logged
    let counter = *WARNING_COUNTER.lock().unwrap();
    println!("\nWarning was logged {} time(s)", counter);
    assert_eq!(counter, 1, "Warning should be logged exactly once");
}

/// Example with early return pattern
fn demo_early_return() {
    println!("\n=== Early Return Pattern ===\n");

    // Reset counter
    *WARNING_COUNTER.lock().unwrap() = 0;

    println!("Calling function 5 times...");
    for i in 1..=5 {
        println!("\nCall #{}: ", i);

        // The following function will exit early after warning once
        let result = function_with_early_return();

        println!("Result: {}", result);
    }

    // Check how many times the warning was logged
    let counter = *WARNING_COUNTER.lock().unwrap();
    println!("\nWarning was logged {} time(s)", counter);
    assert_eq!(counter, 1, "Warning should be logged exactly once");
}

fn function_with_early_return() -> &'static str {
    // This is our condition that would trigger a warning and early return
    let feature_disabled = true;

    // warn_once with early return
    warn_once!(
        feature_disabled,
        || {
            debug_log("This warning should only appear once");
        },
        return "Feature disabled"
    );

    // This code only runs if the condition is false
    "Feature enabled"
}

/// Example using the ID-based version for multiple independent warnings
fn demo_with_id() {
    println!("\n=== Multiple Independent Warnings Using IDs ===\n");

    // Reset counter
    *WARNING_COUNTER.lock().unwrap() = 0;

    // We'll track multiple different warnings
    let warning_counts = Arc::new(Mutex::new(vec![0, 0]));

    println!("Calling two different warning functions 5 times each...");
    for i in 1..=5 {
        println!("\nIteration #{}:", i);

        // First warning - ID 1
        let counts = warning_counts.clone();
        unsafe {
            thag_profiler::warn_once_with_id(1, true, || {
                println!("[LOG] First warning - should appear once");
                let mut counters = counts.lock().unwrap();
                counters[0] += 1;
            });
        }

        // Second warning - ID 2
        let counts = warning_counts.clone();
        unsafe {
            thag_profiler::warn_once_with_id(2, true, || {
                println!("[LOG] Second warning - should also appear once");
                let mut counters = counts.lock().unwrap();
                counters[1] += 1;
            });
        }
    }

    let counts = warning_counts.lock().unwrap();
    println!("\nWarning #1 was logged {} time(s)", counts[0]);
    println!("Warning #2 was logged {} time(s)", counts[1]);
    assert_eq!(counts[0], 1, "First warning should be logged exactly once");
    assert_eq!(counts[1], 1, "Second warning should be logged exactly once");
}

/// Performance comparison between naive approach and warn_once
fn demo_performance() {
    println!("\n=== Performance Comparison ===\n");

    const ITERATIONS: u32 = 10_000_000;

    println!("Running {} iterations", ITERATIONS);

    // 1. Naive approach with atomic check every time
    let start = Instant::now();
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static WARNING_SHOWN: AtomicBool = AtomicBool::new(false);

        for _ in 0..ITERATIONS {
            if !WARNING_SHOWN.load(Ordering::Relaxed) {
                if !WARNING_SHOWN.swap(true, Ordering::Relaxed) {
                    // Warning would be logged here, but we skip it in the benchmark
                }
            }
        }
    }
    let naive_time = start.elapsed();

    // 2. warn_once approach with fast path
    let start = Instant::now();
    {
        for _ in 0..ITERATIONS {
            warn_once!(true, || {
                // Warning would be logged here, but we skip it in the benchmark
            });
        }
    }
    let warn_once_time = start.elapsed();

    println!("Naive approach time: {:?}", naive_time);
    println!("warn_once time:      {:?}", warn_once_time);
    println!(
        "Speedup: {:.2}x",
        naive_time.as_nanos() as f64 / warn_once_time.as_nanos() as f64
    );
}

/// Real-world example based on the record_dealloc function
fn demo_record_dealloc_conversion() {
    println!("\n=== Real-world Example: record_dealloc ===\n");

    println!("Before: Complex nested if-statements with duplicate code:");
    println!("```rust");
    println!("fn record_dealloc(address: usize, size: usize) {{");
    println!("    // [recursion prevention code omitted for brevity]");

    println!("    let is_mem_prof = lazy_static_var!(bool,");
    println!("        get_global_profile_type() == ProfileType::Memory || profile_type == ProfileType::Both,");
    println!("    );");

    println!("    if !is_mem_prof {{");
    println!("        // Fast path check - no synchronization overhead after first warning");
    println!("        if unsafe {{ WARNED }} {{");
    println!("            return;");
    println!("        }}");
    println!("        ");
    println!(
        "        // Slow path with proper synchronization - only hit by the first few threads"
    );
    println!(
        "        if !WARNED_ABOUT_SKIPPING.swap(true, std::sync::atomic::Ordering::Relaxed) {{"
    );
    println!("            debug_log!(");
    println!("                \"Skipping deallocation recording because profile_type={{:?}}\",");
    println!("                profile_type");
    println!("            );");
    println!("            // Update fast path flag for future calls");
    println!("            unsafe {{ WARNED = true; }}");
    println!("        }}");
    println!("        return;");
    println!("    }}");

    println!("    // [rest of function omitted for brevity]");
    println!("}}");
    println!("```");

    println!("\nAfter: Using warn_once! macro:");
    println!("```rust");
    println!("fn record_dealloc(address: usize, size: usize) {{");
    println!("    // [recursion prevention code omitted for brevity]");

    println!("    let is_mem_prof = lazy_static_var!(bool,");
    println!("        get_global_profile_type() == ProfileType::Memory || profile_type == ProfileType::Both,");
    println!("    );");

    println!("    // Use warn_once! macro for clean, optimized warning suppression");
    println!("    warn_once!(!is_mem_prof, || {{");
    println!("        debug_log!(");
    println!("            \"Skipping deallocation recording because profile_type={{:?}}\",");
    println!("            profile_type");
    println!("        );");
    println!("    }}, return);");

    println!("    // [rest of function omitted for brevity]");
    println!("}}");
    println!("```");

    println!("\nBenefits:\n");
    println!("1. Code is more readable and maintainable");
    println!("2. Pattern is abstracted for reuse across the codebase");
    println!("3. Same ultra-low overhead fast path");
    println!("4. Thread-safety preserved");
    println!("5. Eliminates possibility of coding errors in the pattern");
}

/// Main entry point
fn main() {
    println!("Demonstrating the warn_once pattern");
    println!("====================================\n");

    demo_simple_warning();
    demo_early_return();
    demo_with_id();
    demo_performance();
    demo_record_dealloc_conversion();

    // Show the implementation
    println!("\n=== warn_once Implementation ===\n");
    println!("```rust");
    println!("macro_rules! warn_once {{");
    println!("    ($condition:expr, $warning_fn:expr) => {{{{");
    println!("        // Fast path using non-atomic bool for zero overhead after first warning");
    println!("        static mut WARNED: bool = false;");
    println!("        // Thread-safe initialization using atomic");
    println!("        static WARNED_ABOUT_SKIPPING: std::sync::atomic::AtomicBool =");
    println!("            std::sync::atomic::AtomicBool::new(false);");

    println!("        if $condition {{");
    println!("            // Fast path check - no synchronization overhead after first warning");
    println!("            if unsafe {{ WARNED }} {{");
    println!("                // Skip - already warned");
    println!("            }} else {{");
    println!("                // Slow path with proper synchronization - only hit once");
    println!("                if !WARNED_ABOUT_SKIPPING.swap(true, std::sync::atomic::Ordering::Relaxed) {{");
    println!("                    // Execute the warning function");
    println!("                    $warning_fn();");
    println!("                    // Update fast path flag for future calls");
    println!("                    unsafe {{ WARNED = true; }}");
    println!("                }}");
    println!("            }}");
    println!("            true // Return true if condition was met");
    println!("        }} else {{");
    println!("            false // Return false if condition was not met");
    println!("        }}");
    println!("    }}}};");
    println!("}}");
    println!("```");
}
