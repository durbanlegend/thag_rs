/*[toml]
[dependencies]
num-traits = "0.2.19"
*/

/// Purely academic example of a closed-form (direct) calculation of any given
/// number in the Fibonacci sequence using Binet's formula. This is imprecise
/// above about F70, and the `dashu` crate can't help us because it refuses to
/// compute powers of a negative number because they may result in a complex
/// number. Regardless, relying on approximations of irrational numbers lends
/// itself to inaccuracy.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo closed-form Fibonacci computation and the limitations of calculations based on irrational numbers.
use num_traits::Float;
use std::env;

fn fib_closed<T: Float>(n: T) -> T {
    let sqrt_5 = T::from(5.0).unwrap().sqrt();
    let phi = (T::from(1.0).unwrap() + sqrt_5) / T::from(2.0).unwrap();
    let psi = (T::from(1.0).unwrap() - sqrt_5) / T::from(2.0).unwrap();

    (phi.powf(n) - psi.powf(n)) / sqrt_5
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let seq: Vec<f64> = (0..=n).map(|i| fib_closed(i as f64)).collect();
    let mut i = 0;
    for fib in seq {
        println!("F{i} = {}", fib as i64);
        i += 1;
    }
}
