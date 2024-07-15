/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// A version of `demo/fib_doubling_recursive.rs`, minus the memoization.
/// This serves to prove that the memoization is significantly faster, although
/// not dramatically so.
///
//# Purpose: Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.
use ibig::{ubig, UBig};
use std::env;

fn fib(n: usize) -> UBig {
    if n == 0 {
        // eprintln!("Entered fib but returning n={n}");
        return ubig!(0);
    } else if n == 1 {
        // eprintln!("Entered fib but returning n={n}");
        return ubig!(1);
    }

    // eprintln!("Entered fib with n={n}");
    if n % 2 == 0 {
        let k = n / 2;
        let fk = fib(k);
        let fk_minus_1 = fib(k - 1);
        &fk * (2 * &fk_minus_1 + &fk)
    } else {
        let k = (n + 1) / 2;
        let fk = fib(k);
        let fk_minus_1 = fib(k - 1);
        &fk * &fk + &fk_minus_1 * &fk_minus_1
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let result = fib(n);
    println!("Fibonacci number F({}) is {}", n, result);
}
