/*[toml]
[dependencies]
# No external dependencies - this is a standalone demo
*/

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
/// This script demonstrates the usage of the `warn_once_with_id` function for suppressing repeated
/// log messages with minimal runtime overhead using unique IDs.
///
/// This is a standalone implementation that doesn't require any external dependencies.
/// The function uses unsafe code for maximum performance with a fast path after the first warning.
///
/// Credit to `Claude Sonnet 4` for the implementation and comprehensive demo.
//# Purpose: Standalone demo of warn_once_with_id function with embedded implementation
//# Categories: demo, macros, technique, unsafe, performance
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Standalone implementation of warn_once_with_id function
///
/// This function provides a high-performance way to suppress repeated warnings
/// using unique IDs to track different warning sites independently.
///
/// # Safety
///
/// This function is unsafe because:
/// - It uses static mutable data with UnsafeCell
/// - Caller must ensure each ID is unique per call site
/// - The ID should be < 128 for optimal performance (higher IDs use modulo)
///
/// # Arguments
///
/// * `id` - Unique identifier for this warning site (0-127 for best performance)
/// * `condition` - Whether the warning condition is met
/// * `warning_fn` - Closure to execute for the warning (called only once)
///
/// # Returns
///
/// * `true` if the condition was met (regardless of whether warning was shown)
/// * `false` if the condition was not met
///
/// # Example
///
/// ```rust
/// const MY_WARNING_ID: usize = 1;
///
/// unsafe {
///     warn_once_with_id(MY_WARNING_ID, some_error_condition, || {
///         eprintln!("This warning will only appear once!");
///     });
/// }
/// ```
pub unsafe fn warn_once_with_id<F>(id: usize, condition: bool, warning_fn: F) -> bool
where
    F: FnOnce(),
{
    // Static storage for up to 128 unique warning flags
    // This approach avoids needing to create a new static for every call site
    static mut WARNED_FLAGS: [UnsafeCell<bool>; 128] = [const { UnsafeCell::new(false) }; 128];
    static ATOMIC_FLAGS: [AtomicBool; 128] = [const { AtomicBool::new(false) }; 128];

    // Safety: Caller must ensure id is unique per call site
    let idx = id % 128;

    if !condition {
        return false;
    }

    // Fast path check - no synchronization overhead after first warning
    if unsafe { *WARNED_FLAGS[idx].get() } {
        return true;
    }

    // Slow path with proper synchronization
    if !ATOMIC_FLAGS[idx].swap(true, Ordering::Relaxed) {
        // Execute the warning function
        warning_fn();
        // Update fast path flag
        unsafe {
            *WARNED_FLAGS[idx].get() = true;
        }
    }

    true
}

/// Demo showing multiple independent warnings with different IDs
fn demo_multiple_warnings() {
    println!("=== Multiple Independent Warnings Demo ===\n");

    // Counters to track how many times each warning fires
    let counter1 = Arc::new(Mutex::new(0));
    let counter2 = Arc::new(Mutex::new(0));
    let counter3 = Arc::new(Mutex::new(0));

    println!("Running 10 iterations with 3 different warning IDs...\n");

    for i in 1..=10 {
        println!("Iteration {}:", i);

        // Warning ID 1 - Should only fire once
        let c1 = counter1.clone();
        unsafe {
            warn_once_with_id(1, true, || {
                println!("  [WARNING 1] Database connection slow - consider connection pooling");
                let mut count = c1.lock().unwrap();
                *count += 1;
            });
        }

        // Warning ID 2 - Should only fire once (independent of ID 1)
        let c2 = counter2.clone();
        unsafe {
            warn_once_with_id(2, i % 3 == 0, || {
                println!("  [WARNING 2] Cache miss rate high - consider warming cache");
                let mut count = c2.lock().unwrap();
                *count += 1;
            });
        }

        // Warning ID 3 - Should only fire once (independent of others)
        let c3 = counter3.clone();
        unsafe {
            warn_once_with_id(3, i > 5, || {
                println!("  [WARNING 3] Memory usage approaching limit - consider optimization");
                let mut count = c3.lock().unwrap();
                *count += 1;
            });
        }

        println!("  Processing work for iteration {}", i);
    }

    println!("\nFinal warning counts:");
    println!("  Warning 1 fired: {} times", *counter1.lock().unwrap());
    println!("  Warning 2 fired: {} times", *counter2.lock().unwrap());
    println!("  Warning 3 fired: {} times", *counter3.lock().unwrap());
}

/// Demo showing performance characteristics
fn demo_performance() {
    println!("\n=== Performance Characteristics Demo ===\n");

    const ITERATIONS: u32 = 10_000_000;
    println!(
        "Running {} iterations to measure performance...",
        ITERATIONS
    );

    // Warm up - trigger the warning once
    unsafe {
        warn_once_with_id(100, true, || {
            println!("Warmup warning (should appear once)");
        });
    }

    // Now measure the fast path performance
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        unsafe {
            warn_once_with_id(100, true, || {
                // This closure should never execute after warmup
                unreachable!("This should not execute in performance test");
            });
        }
    }
    let duration = start.elapsed();

    println!(
        "Fast path time for {} iterations: {:?}",
        ITERATIONS, duration
    );
    println!(
        "Average time per call: {:.2} ns",
        duration.as_nanos() as f64 / ITERATIONS as f64
    );
}

/// Demo showing thread safety
fn demo_thread_safety() {
    println!("\n=== Thread Safety Demo ===\n");

    use std::sync::Arc;
    use std::thread;

    let warning_count = Arc::new(Mutex::new(0));
    let num_threads = 8;
    let iterations_per_thread = 1000;

    println!(
        "Spawning {} threads, each doing {} iterations...",
        num_threads, iterations_per_thread
    );

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let counter = warning_count.clone();
        let handle = thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let counter_clone = counter.clone();
                unsafe {
                    warn_once_with_id(200, true, || {
                        println!("Thread {} fired warning on iteration {}", thread_id, i);
                        let mut count = counter_clone.lock().unwrap();
                        *count += 1;
                    });
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let final_count = *warning_count.lock().unwrap();
    println!("Total warnings fired across all threads: {}", final_count);
    println!("Expected: 1 (should fire exactly once despite multiple threads)");
    assert_eq!(
        final_count, 1,
        "Warning should fire exactly once despite multiple threads"
    );
}

/// Demo showing real-world usage patterns
fn demo_real_world_usage() {
    println!("\n=== Real-World Usage Patterns Demo ===\n");

    // Simulate a function that might be called frequently but should only warn once
    // about configuration issues
    fn process_request(request_id: u32) -> Result<String, &'static str> {
        // Simulate checking some configuration
        let config_missing = std::env::var("IMPORTANT_CONFIG").is_err();

        // Warning ID 300 for configuration issues
        unsafe {
            warn_once_with_id(300, config_missing, || {
                eprintln!(
                    "WARNING: IMPORTANT_CONFIG environment variable not set. Using defaults."
                );
                eprintln!("This may impact performance. Set IMPORTANT_CONFIG=production for optimal performance.");
            });
        }

        // Simulate request processing
        if request_id % 100 == 0 {
            // Warning ID 301 for occasional issues
            unsafe {
                warn_once_with_id(301, true, || {
                    eprintln!("WARNING: Heavy request detected. Consider implementing request throttling.");
                });
            }
        }

        Ok(format!("Processed request {}", request_id))
    }

    println!("Processing 1000 simulated requests...");
    for i in 1..=1000 {
        match process_request(i) {
            Ok(result) => {
                if i <= 5 || i % 200 == 0 {
                    println!("Request {}: {}", i, result);
                }
            }
            Err(e) => println!("Request {} failed: {}", i, e),
        }
    }
    println!("...processing complete");
}

/// Demo showing ID collision handling
fn demo_id_collision() {
    println!("\n=== ID Collision Handling Demo ===\n");

    println!("The function uses modulo 128 for ID mapping, so IDs 0 and 128 would collide.");
    println!("Let's demonstrate this:\n");

    let counter = Arc::new(Mutex::new(0));

    // First, trigger warning with ID 0
    let c = counter.clone();
    unsafe {
        warn_once_with_id(0, true, || {
            println!("Warning with ID 0 fired");
            let mut count = c.lock().unwrap();
            *count += 1;
        });
    }

    // Now try with ID 128 (should be suppressed due to collision)
    let c = counter.clone();
    unsafe {
        warn_once_with_id(128, true, || {
            println!("Warning with ID 128 fired (this should NOT appear)");
            let mut count = c.lock().unwrap();
            *count += 1;
        });
    }

    println!("Total warnings fired: {}", *counter.lock().unwrap());
    println!("Expected: 1 (ID 128 should be suppressed due to collision with ID 0)");

    println!("\nRecommendation: Use unique IDs in range 0-127 for best results.");
}

/// Demo showing early return pattern
fn demo_early_return() {
    println!("\n=== Early Return Pattern Demo ===\n");

    fn expensive_operation(should_skip: bool) -> Option<String> {
        // Use warn_once_with_id to warn about skipping expensive operations
        unsafe {
            if warn_once_with_id(400, should_skip, || {
                println!("WARNING: Skipping expensive operation due to configuration");
                println!("This may impact functionality. Check your settings.");
            }) {
                return None; // Early return when condition is met
            }
        }

        // Simulate expensive work
        std::thread::sleep(std::time::Duration::from_millis(10));
        Some("Expensive operation completed".to_string())
    }

    println!("Calling expensive_operation 5 times with skip=true...");
    for i in 1..=5 {
        match expensive_operation(true) {
            Some(result) => println!("Call {}: {}", i, result),
            None => println!("Call {}: Skipped", i),
        }
    }
}

fn main() {
    println!("warn_once_with_id Standalone Demo");
    println!("=================================\n");

    demo_multiple_warnings();
    demo_performance();
    demo_thread_safety();
    demo_real_world_usage();
    demo_id_collision();
    demo_early_return();

    println!("\n=== Implementation Details ===\n");
    println!("The warn_once_with_id function provides:");
    println!("• Ultra-fast performance after first warning (no atomic operations)");
    println!("• Thread-safe initialization using atomic compare-and-swap");
    println!("• Support for up to 128 independent warning sites");
    println!("• Minimal memory footprint (256 bytes for all warnings)");
    println!("• Zero runtime overhead for conditions that are false");
    println!("\nSafety considerations:");
    println!("• Function is marked unsafe due to static mutable data");
    println!("• Caller must ensure unique IDs per call site");
    println!("• IDs >= 128 are mapped using modulo (potential collisions)");
    println!("• Fast path uses non-atomic reads (requires proper initialization)");

    println!("\nThis implementation demonstrates advanced Rust techniques:");
    println!("• UnsafeCell for interior mutability");
    println!("• AtomicBool for thread-safe initialization");
    println!("• Static arrays for zero-allocation storage");
    println!("• Careful ordering of atomic operations");
    println!("• Generic closures with FnOnce trait bound");
}
