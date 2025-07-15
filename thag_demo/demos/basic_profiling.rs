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
use ibig::{ubig, UBig};
use num_traits::identities::One;
use std::io::Write;
use std::iter::successors;
use std::thread;
use std::time::Duration;
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::{cached, timing};
use thag_profiler::{enable_profiling, profiled};

const FIB_N: usize = 45;
const HUNDREDFOLD: usize = FIB_N * 100;
// const MULTIPLIER: usize = 200;
// const MULTIPLIED: usize = FIB_N * MULTIPLIER;

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

#[profiled]
#[timing]
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
    println!("\nHey, it's-a me, Fibonacci!\n");
    println!(
        "Let's calculate my {FIB_N}th Fibonacci number recursively, because it's a chunky computation, but not insanely so."
    );
    println!("Elapsed time for recursion increases exponentially with the Fibonacci number, so we don't want to overdo it.\n");

    // First recursively - bad idea as O(2^n)
    fibonacci_recursions(FIB_N);

    let _ = std::io::stdout().flush();
}

// Pause to display output and help drill down to the tiny flamegraph bars for fast functions
fn pause_awhile() {
    let _ = std::io::stdout().flush();
    thread::sleep(Duration::from_secs(2));
}

#[profiled]
#[timing]
fn alt_fibonacci_strategies() {
    println!("\nOof, bad idea. And it will quickly get a lot worse for bigger numbers.");
    println!("\nHow about we use thag's demo #[cached] attribute on the fibonacci function?\n");

    pause_awhile();

    // Then with cached functions
    fibonacci_recursions_cached(FIB_N);

    println!("\nThat's insane!");
    println!(
        "\nA little bird told me I can go up two orders of magnitude and calculate my {}th number and still come out way ahead!\n",
        HUNDREDFOLD
    );

    pause_awhile();

    // Then with cached functions
    fibonacci_recursions_cached(HUNDREDFOLD);

    println!("\nHoly smokes! What a difference! Recursion is not always your friend, but #[cached] is your friend - at least up until the stack overflows from too much recursion.");
    println!("\nWhat if we try Rust iterators instead, still for F({HUNDREDFOLD})?\n");

    pause_awhile();

    // Non-nested with Rust iterator. Even then, will it show up in profiling?
    fibonacci_iter(HUNDREDFOLD);

    println!(
        "\n🤯 Not too shabby! But we can go a lot bigger and faster still with no overflows - you can go down the rabbit hole in the thag demo collection. Ciao!\n"
    );

    pause_awhile();
}

#[enable_profiling(time)]
fn main() {
    println!("🔥 Basic Profiling Demo");
    println!("=====================");
    println!();

    println!("Running nested function calls with profiling...");
    nested_function_calls();

    // Separate function so that we can drill past it to see the others
    alt_fibonacci_strategies();

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated flamegraph files for visual analysis.");
    println!("🔍 Use 'thag_profile' command to analyze the profiling data.");
}
