/// Demo of the cached attribute macro that adds automatic memoization to functions.
///
/// This macro demonstrates advanced attribute macro techniques by wrapping functions
/// with caching logic. It automatically stores function results and returns cached
/// values for repeated calls with the same parameters, providing significant
/// performance improvements for expensive computations.
//# Purpose: Demonstrate automatic function memoization with caching
//# Categories: technique, proc_macros, attribute_macros, performance, caching
use std::time::Instant;
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::cached;

/// Expensive computation that benefits from caching
#[cached]
fn fibonacci(n: u32) -> u64 {
    println!("  Computing fibonacci({})", n);
    if n <= 1 {
        n as u64
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

/// Expensive string processing that benefits from caching
#[cached]
fn expensive_string_processing(input: String) -> String {
    println!("  Processing string: '{}'", input);
    // Simulate expensive processing
    std::thread::sleep(std::time::Duration::from_millis(500));
    input.to_uppercase().repeat(2)
}

/// Mathematical computation with multiple parameters
#[cached]
fn complex_calculation(a: i32, b: i32, c: i32) -> i32 {
    println!("  Calculating complex_calculation({}, {}, {})", a, b, c);
    // Simulate expensive calculation
    std::thread::sleep(std::time::Duration::from_millis(200));
    a * a + b * b + c * c
}

/// Prime number checking (expensive operation)
#[cached]
fn is_prime(n: u32) -> bool {
    println!("  Checking if {} is prime", n);
    if n < 2 {
        return false;
    }
    for i in 2..=(n as f64).sqrt() as u32 {
        if n % i == 0 {
            return false;
        }
    }
    true
}

fn time_function<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("  ⏱️  {} took: {:?}", name, duration);
    result
}

fn main() {
    println!("🔄 Cached Attribute Macro Demo");
    println!("==============================\n");

    // Example 1: Fibonacci sequence with caching
    println!("1. Fibonacci sequence caching:");
    println!("   First calculation (cache miss):");
    let fib_result = time_function("fibonacci(10)", || fibonacci(10));
    println!("   Result: {}", fib_result);

    println!("\n   Second calculation (cache hit):");
    let fib_result2 = time_function("fibonacci(10)", || fibonacci(10));
    println!("   Result: {}", fib_result2);

    println!("\n   Related calculation (partial cache hits):");
    let fib_result3 = time_function("fibonacci(12)", || fibonacci(12));
    println!("   Result: {}", fib_result3);

    // Example 2: String processing with caching
    println!("\n2. String processing caching:");
    println!("   First processing (cache miss):");
    let str_result = time_function("expensive_string_processing", || {
        expensive_string_processing("hello".to_string())
    });
    println!("   Result: '{}'", str_result);

    println!("\n   Second processing (cache hit):");
    let str_result2 = time_function("expensive_string_processing", || {
        expensive_string_processing("hello".to_string())
    });
    println!("   Result: '{}'", str_result2);

    println!("\n   Different input (cache miss):");
    let str_result3 = time_function("expensive_string_processing", || {
        expensive_string_processing("world".to_string())
    });
    println!("   Result: '{}'", str_result3);

    // Example 3: Multiple parameter caching
    println!("\n3. Multiple parameter caching:");
    println!("   First calculation (cache miss):");
    let calc_result = time_function("complex_calculation(3, 4, 5)", || {
        complex_calculation(3, 4, 5)
    });
    println!("   Result: {}", calc_result);

    println!("\n   Same parameters (cache hit):");
    let calc_result2 = time_function("complex_calculation(3, 4, 5)", || {
        complex_calculation(3, 4, 5)
    });
    println!("   Result: {}", calc_result2);

    println!("\n   Different parameters (cache miss):");
    let calc_result3 = time_function("complex_calculation(1, 2, 3)", || {
        complex_calculation(1, 2, 3)
    });
    println!("   Result: {}", calc_result3);

    // Example 4: Prime number checking
    println!("\n4. Prime number checking with caching:");
    let test_numbers = vec![97, 98, 99, 100, 97]; // Note: 97 appears twice

    for number in test_numbers {
        println!("\n   Checking if {} is prime:", number);
        let is_prime_result = time_function(&format!("is_prime({})", number), || is_prime(number));
        println!("   Result: {}", is_prime_result);
    }

    // Example 5: Performance comparison
    println!("\n5. Performance comparison:");
    println!("   Computing fibonacci(8) multiple times to show caching benefit:");

    let numbers = vec![8, 8, 8, 9, 8, 10, 8];
    for (i, n) in numbers.iter().enumerate() {
        println!("\n   Call {}: fibonacci({})", i + 1, n);
        let result = time_function(&format!("Call {}", i + 1), || fibonacci(*n));
        println!("   Result: {}", result);
    }

    println!("\n🎉 Cached attribute macro demo completed successfully!");
    println!("\nKey features demonstrated:");
    println!("  - Automatic result caching for repeated function calls");
    println!("  - Thread-safe cache implementation");
    println!("  - Support for functions with multiple parameters");
    println!("  - Significant performance improvements for expensive operations");
    println!("  - Transparent caching (no code changes required)");
    println!("  - Cache key generation based on parameter values");

    println!("\nUse cases for #[cached]:");
    println!("  - Expensive mathematical computations");
    println!("  - Database query results");
    println!("  - File system operations");
    println!("  - API calls with stable results");
    println!("  - Complex data transformations");
}
