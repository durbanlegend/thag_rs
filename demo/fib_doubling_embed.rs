/*[toml]
[dependencies]
ibig = "0.3.6"
tailcall = "1.0.1"
*/

/// Fast recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity.
/// I'm not sure of the theory and I'm sure this is well known, but I stumbled
/// across an apparent pattern in the Fibonacci sequence:
/// For m > n: Fm = Fn-1.Fm-n + Fn.Fm-n+1.
/// This led straight to what ChatGPT tells me are the well-known "doubling identities":
/// For even indices: `F2n = Fn x (Fn-1 + Fn+1)`.
/// For odd indices: `F2n+1 = Fn^2 + Fn+1^2`.
/// So we should be able to compute a given Fibonacci number F2n or F2n+1 recursively
/// expressing it in terms of Fn-1, Fn and Fn+1
///
///
//# Purpose: An attempt to see if we could avoid implement tail-call recursion or remove it by embedding a method. Ran out of talent so far .
use ibig::{ubig, UBig};
use std::env;

fn fibonacci(n: usize) -> UBig {
    fn fib(n: usize) -> UBig {
        eprintln!("Entered fib with n={n}");
        if n == 0 {
            return ubig!(0);
        } else if n == 1 {
            return ubig!(1);
        }

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
    fib(n)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let result = fibonacci(n);
    println!("Fibonacci number F({}) is {}", n, result);
}
