/*[toml]
[dependencies]
thag_demo_proc_macros = { version = "0.1, thag-auto" }
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Basic profiling demo - shows how to use thag_profiler for function timing
/// This demo demonstrates the core profiling features of thag_profiler
//# Purpose: Demonstrate basic time profiling with thag_profiler
//# Categories: profiling, demo, timing
use num_traits::identities::One;
use std::iter::successors;
use std::thread;
use std::time::Duration;
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use ibig::{ubig, UBig};
use thag_demo_proc_macros::{cached, timing};
use thag_profiler::{enable_profiling, profiled};

#[profiled]
#[timing]
fn fibonacci_recursions(n: usize) {
    let result = fibonacci(n);
    println!("fibonacci({n}) = {result}");
}

// For recursive functions, only time-profile the caller, to avoid
// unfixable multiple counting of elapsed time.
fn fibonacci(n: usize) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[profiled]
#[timing]
fn fibonacci_recursions_cached(n: usize) {
    let result = fibonacci_cached(UBig::from(n));
    println!("fibonacci({n}) (cached) = {result}");
}

// For recursive functions, only time-profile the caller, to avoid
// unfixable multiple counting of elapsed time.
#[cached]
fn fibonacci_cached(n: UBig) -> UBig {
    if n <= UBig::one() {
        n
    } else {
        fibonacci_cached(n.clone() - 1) + fibonacci_cached(n - 2)
    }
}

#[profiled]
#[timing]
fn fibonacci_iter(n: usize) {
    let result = successors(Some((ubig!(0), ubig!(1))), |(a, b)| {
        Some((b.clone(), (a + b).into()))
    })
    .map(|(a, _b)| a)
    .nth(n)
    .unwrap();
    println!("fibonacci({n}) (iter) = {result}");
}

#[profiled]
#[timing]
fn cpu_intensive_work() {
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum += i * i;
    }
    println!("CPU work result: {}", sum);
}

#[timing]
#[profiled]
fn simulated_io_work() {
    println!("Starting simulated I/O work...");
    thread::sleep(Duration::from_millis(100));
    println!("I/O work completed");
}

#[profiled]
#[timing]
fn nested_function_calls() {
    cpu_intensive_work();
    simulated_io_work();

    // Calculate some fibonacci numbers
    const FIB_N: usize = 45;
    const HUNDREDFOLD: usize = FIB_N * 100;

    println!("\nHey, it's-a me, Fibonacci!");
    println!("Let's calculate my {FIB_N}th number recursively, just because we can...");

    // First recursively - bad idea as O(2^n)
    fibonacci_recursions(FIB_N);

    println!("\nOof, bad idea - how about we use thag's demo #[cached] attribute on the fibonacci function?");
    println!(
        "\nA little bird told me I can go up two orders of magnitude and calculate my {}th number and still come out ahead!\n",
        HUNDREDFOLD
    );

    // Then with cached functions
    fibonacci_recursions_cached(HUNDREDFOLD);

    println!("\nHoly smokes! What a difference! Recursion is not always your friend, but #[cached] is your friend.");
    println!(
        "\nWhat if we try Rust iterators instead, still for F({})?\n",
        HUNDREDFOLD
    );

    // Non-nested with Rust iterator. Even then, will it show up in profiling?
    fibonacci_iter(HUNDREDFOLD);

    println!(
        "\n🤯 Not too shabby! But we can go a lot faster still - you can go down the rabbit hole in the thag demo collection. Ciao!"
    );
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
