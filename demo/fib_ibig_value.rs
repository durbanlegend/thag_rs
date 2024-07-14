/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Fast non-recursive Fibonacci individual calculation with big integers.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demonstrate snippets a fast non-recursive fibonacci algorithm using the `successors`.
use ibig::{ubig, UBig};
use std::env;
use std::io::Read;
use std::iter::{successors, Successors, Take};

// Snippet accepts function or closure. This closure returns only the last value Fn.
fn fib_value_n(n: usize) -> UBig {
    successors(Some((ubig!(0), ubig!(1))), |(a, b)| {
        Some((b.clone(), (a + b).into()))
    })
    .nth(n)
    .unwrap()
    .0
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

println!("Fibonacci F({n}) is {}", fib_value_n(n));
