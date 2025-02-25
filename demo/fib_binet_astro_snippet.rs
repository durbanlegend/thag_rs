/*[toml]
[dependencies]
astro-float = "0.9.4"
*/

/// Academic / recreational example of a closed-form (direct) calculation of a
/// given number in the Fibonacci sequence using Binet's formula. This is imprecise
/// above about F70, and the `dashu` crate can't help us because it does not support
/// computing powers of a negative number since they may result in a complex
/// number. Regardless, relying on approximations of irrational numbers lends
/// itself to inaccuracy.
///
/// Shout-out to the `expr!` macro of the `astro-float` crate, which reduces very
/// complex representations back to familiar expressions.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo closed-form Fibonacci computation and the limitations of calculations based on irrational numbers, also `astro-float` crate..
//# Categories: big_numbers, learning, math, recreational, technique
//# Sample arguments: `-- 100`
use astro_float::{expr, BigFloat, Consts, RoundingMode};
use astro_float::ctx::Context;
use std::env;
use std::str::FromStr;

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

let p = 128;
let rm = RoundingMode::Up;
let cc = Consts::new().expect("Failed to allocate constants cache");
let emin = -10000;
let emax = 10000;

// Create a context.
let mut ctx = Context::new(p, rm, cc, emin, emax);

let sqrt_5 = f64::from(5.0).sqrt();
let phi = (f64::from(1.0) + sqrt_5) / 2.0_f64;
let psi = (f64::from(1.0) - sqrt_5) / 2.0_f64;
// println!("sqrt_5={}, phi={:?}, psi={:?}", sqrt_5, phi, psi);

for i in 0..=n {
    let f = i as f64;
    let fib_big_float = expr!((pow(phi, f) - pow(psi, f)) / sqrt_5, &mut ctx);
    let fib_str = format!("{fib_big_float}");
    let fib = f64::from_str(&fib_str).expect("Failed to parse number into f64") as u128;
    println!("F{i} = {}", fib);
}
