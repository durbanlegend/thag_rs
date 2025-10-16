/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling", "demo"] }

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
use std::io;
use std::iter::successors;
use std::thread;
use std::time::Duration;
use thag_demo_proc_macros::cached;
use thag_profiler::{
    enable_profiling, profiled, prompted_analysis, timing, AnalysisType, ProfileType,
};

const FIB_N: usize = 42;
const HUNDREDFOLD: usize = FIB_N * 100;
const THOUSANDFOLD: usize = FIB_N * 1000;

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

// #[profiled]
fn nested_function_calls() {
    cpu_intensive_work();
    simulated_io_work();

    // Calculate some fibonacci numbers
    println!();
    println!(
        "Let's calculate the {FIB_N}{} Fibonacci number recursively, because {FIB_N} will take a few seconds, but not ages.", get_ordinal_suffix(FIB_N)
    );
    println!("Elapsed time for recursion increases exponentially with the Fibonacci number, so we don't want to overdo it.");
    println!();
    await_enter();
    println!("Processing, please wait...");

    // First recursively - bad idea as O(2^n)
    fibonacci_recursions(FIB_N);

    println!();
    println!("Well, that was quite slow. And it will quickly get a LOT worse for bigger numbers, because this recursion's performance is O(2^n).");
    println!("Also, the call stack will soon overflow.");
}

// #[profiled]
fn alt_fibonacci_cached() {
    println!();
    println!("Let's try speeding it up by simply adding thag's demo #[cached] attribute to the recursive fibonacci function (-> `fn fibonacci_cached`).");
    println!();

    await_enter();
    println!("Processing, please wait...");

    // Then with cached functions
    fibonacci_recursions_cached(FIB_N);

    println!();
    println!("That probably ran a whole lot faster!");
    println!();
    println!("Lets try going up two orders of magnitude and calculating the {HUNDREDFOLD}th number, still with #[cached]. I'm predicting we'll still come out way ahead!");
    println!();

    await_enter();

    // Then with cached functions
    fibonacci_recursions_cached(HUNDREDFOLD);

    println!();
    println!("What a difference! Recursion is not always your friend, but #[cached] is your friend - at least up until the stack still overflows from too much recursion.");
}

// #[profiled]
fn alt_fibonacci_iter() {
    println!();
    println!("What if we try Rust iterators instead, and go up yet another order of magnitude to F({THOUSANDFOLD}) to make sure that it will still be visible on the flamegraph?");
    println!();

    await_enter();
    println!("Processing, please wait...");

    // Non-nested with Rust iterator. Even then, will it show up in profiling?
    fibonacci_iter(THOUSANDFOLD);

    println!();
    println!("🤯 Already a big improvement, but with this approach (Rust iterators) we can go a lot bigger and faster still with no overflows.");
    println!("If you want, you can go down the fibonacci rabbit hole in the demo collection of the `thag_rs` crate.");
}

#[enable_profiling(time, function(none))]
fn demo() {
    println!("🔥 Basic Profiling Demo");
    println!("═══════════════════════");
    println!();

    println!("Running nested function calls with profiling...");
    nested_function_calls();

    // Separate function to help in drilling down
    alt_fibonacci_cached();

    // Separate function to help in drilling down
    alt_fibonacci_iter();
}

// #[profiled]
fn await_enter() {
    println!("Press Enter to continue...");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
}

fn get_ordinal_suffix(n: usize) -> String {
    if n == 0 {
        return "th".to_string(); // Or handle as an error/special case if 0 shouldn't have a suffix
    }

    let last_digit = n % 10;
    let last_two_digits = n % 100;

    if last_two_digits == 11 || last_two_digits == 12 || last_two_digits == 13 {
        "th".to_string()
    } else {
        match last_digit {
            1 => "st".to_string(),
            2 => "nd".to_string(),
            3 => "rd".to_string(),
            _ => "th".to_string(),
        }
    }
}
fn main() {
    // Ensure no stack overflow at hundredfold scale on all platforms
    let child = thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            demo();
        })
        .unwrap();

    let _ = child.join().unwrap();

    prompted_analysis(&file!(), ProfileType::Time, AnalysisType::Flamechart);

    println!("✅ Demo completed!");
    println!("📊 Check the generated flamechart files for visual analysis.");
    println!("🔍 Use 'thag_profile' command to analyze the profiling data.");
}
