/// Demo of the timing attribute macro that adds automatic execution time measurement.
///
/// This macro demonstrates simple but effective attribute macro patterns by wrapping
/// functions with timing logic. It automatically measures and displays execution time
/// for any function, making it invaluable for performance analysis and optimization.
//# Purpose: Demonstrate automatic function timing and performance measurement
//# Categories: technique, proc_macros, attribute_macros, performance, timing
use std::time::Duration;
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::timing;

/// Fast computation - should show minimal timing
#[timing]
fn fast_computation() -> i32 {
    let mut sum = 0;
    for i in 1..=1000 {
        sum += i;
    }
    sum
}

/// Medium-speed computation with visible timing
#[timing]
fn medium_computation() -> Vec<u32> {
    let mut numbers = Vec::new();
    for i in 1..=50000 {
        if i % 7 == 0 {
            numbers.push(i);
        }
    }
    numbers
}

/// Slow computation with artificial delay
#[timing]
fn slow_computation(delay_ms: u64) -> String {
    std::thread::sleep(Duration::from_millis(delay_ms));
    format!("Completed after {}ms delay", delay_ms)
}

/// Recursive function with timing at each level
#[timing(expand)]
fn recursive_fibonacci(n: u32) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        recursive_fibonacci(n - 1) + recursive_fibonacci(n - 2)
    }
}

/// Function that might fail, showing timing regardless
#[timing]
fn risky_operation(should_fail: bool) -> Result<String, &'static str> {
    // Simulate some work
    std::thread::sleep(Duration::from_millis(100));

    if should_fail {
        Err("Something went wrong!")
    } else {
        Ok("Operation successful!".to_string())
    }
}

/// Complex data processing with multiple steps
#[timing]
fn data_processing(data: Vec<i32>) -> (i32, i32, f64) {
    let sum: i32 = data.iter().sum();
    let max = *data.iter().max().unwrap_or(&0);
    let average = if data.is_empty() {
        0.0
    } else {
        sum as f64 / data.len() as f64
    };

    // Simulate extra processing time
    std::thread::sleep(Duration::from_millis(50));

    (sum, max, average)
}

/// Function with generic parameters
#[timing]
fn generic_processor<T>(items: Vec<T>) -> usize
where
    T: std::fmt::Debug,
{
    for (i, item) in items.iter().enumerate() {
        if i < 3 {
            // Only print first few for brevity
            println!("    Processing item {}: {:?}", i, item);
        }
    }
    items.len()
}

/// Async-like simulation (using blocking operations)
#[timing]
fn simulated_network_call(url: &str) -> Result<String, String> {
    println!("    Making request to: {}", url);

    // Simulate network latency
    let delay = match url {
        "fast.api.com" => 50,
        "medium.api.com" => 200,
        "slow.api.com" => 800,
        _ => 300,
    };

    std::thread::sleep(Duration::from_millis(delay));

    if url.contains("fail") {
        Err(format!("Failed to connect to {}", url))
    } else {
        Ok(format!("Data from {}", url))
    }
}

fn main() {
    println!("⏱️  Timing Attribute Macro Demo");
    println!("===============================\n");

    // Example 1: Basic timing measurement
    println!("1. Basic timing measurements:");
    println!("   Fast computation:");
    let result1 = fast_computation();
    println!("     Result: {}\n", result1);

    println!("   Medium computation:");
    let result2 = medium_computation();
    println!("     Found {} numbers divisible by 7\n", result2.len());

    // Example 2: Configurable delays
    println!("2. Configurable delay timing:");
    let delays = vec![100, 250, 500];
    for delay in delays {
        println!("   Testing {}ms delay:", delay);
        let result = slow_computation(delay);
        println!("     {}\n", result);
    }

    // Example 3: Recursive function timing
    println!("3. Recursive function timing:");
    println!("   Computing fibonacci numbers:");
    for n in vec![5, 8, 10] {
        println!("   fibonacci({}):", n);
        let result = recursive_fibonacci(n);
        println!("     Result: {}\n", result);
    }

    // Example 4: Error handling with timing
    println!("4. Error handling with timing:");
    println!("   Successful operation:");
    match risky_operation(false) {
        Ok(msg) => println!("     Success: {}", msg),
        Err(e) => println!("     Error: {}", e),
    }

    println!("\n   Failed operation:");
    match risky_operation(true) {
        Ok(msg) => println!("     Success: {}", msg),
        Err(e) => println!("     Error: {}", e),
    }

    // Example 5: Data processing timing
    println!("\n5. Data processing timing:");
    let test_data = vec![10, 25, 30, 15, 40, 35, 20];
    println!("   Processing data: {:?}", test_data);
    let (sum, max, avg) = data_processing(test_data);
    println!("     Sum: {}, Max: {}, Average: {:.2}\n", sum, max, avg);

    // Example 6: Generic function timing
    println!("6. Generic function timing:");
    println!("   Processing string vector:");
    let strings = vec!["hello", "world", "rust", "macro", "timing"];
    let count = generic_processor(strings);
    println!("     Processed {} items\n", count);

    println!("   Processing number vector:");
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let count = generic_processor(numbers);
    println!("     Processed {} items\n", count);

    // Example 7: Simulated network operations
    println!("7. Simulated network timing:");
    let urls = vec![
        "fast.api.com",
        "medium.api.com",
        "slow.api.com",
        "fail.api.com",
    ];

    for url in urls {
        println!("   Network call to {}:", url);
        match simulated_network_call(url) {
            Ok(data) => println!("     Success: {}", data),
            Err(e) => println!("     Error: {}", e),
        }
        println!();
    }

    // Example 8: Comparison of similar operations
    println!("8. Performance comparison:");
    println!("   Comparing different fibonacci calculations:");

    for n in vec![12, 15, 18] {
        println!("   fibonacci({}):", n);
        let _ = recursive_fibonacci(n);
    }

    println!("\n🎉 Timing attribute macro demo completed successfully!");
    println!("\nKey features demonstrated:");
    println!("  - Automatic execution time measurement");
    println!("  - Works with any function signature");
    println!("  - Measures both successful and failed operations");
    println!("  - Compatible with generic functions");
    println!("  - Zero-overhead when macro is not applied");
    println!("  - Clear, readable timing output");

    println!("\nUse cases for #[timing]:");
    println!("  - Performance profiling and optimization");
    println!("  - Debugging slow operations");
    println!("  - API endpoint performance monitoring");
    println!("  - Database query timing");
    println!("  - Algorithm comparison and benchmarking");
    println!("  - Identifying performance bottlenecks");
}
