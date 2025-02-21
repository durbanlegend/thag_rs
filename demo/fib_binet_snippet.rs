/// Purely academic example of a closed-form (direct) calculation of an individual
/// Fibonacci number using Binet's formula. This is imprecise above about F70, and
/// the `dashu` crate can't help us because it does not support computing powers
/// of a negative number because they may result in a complex number. Regardless,
/// relying on approximations of irrational numbers lends itself to inaccuracy.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo closed-form Fibonacci computation and the limitations of calculations based on irrational numbers..
//# Categories: learning, math, recreational, technique
//# Sample arguments: `-- 100`
use std::env;

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

let sqrt_5 = f64::from(5.0).sqrt();
let phi = (f64::from(1.0) + sqrt_5) / 2.0_f64;
let psi = (f64::from(1.0) - sqrt_5) / 2.0_f64;
// println!("sqrt_5={}, phi={:?}, psi={:?}", sqrt_5, phi, psi);

for i in 0..=n {
    let f = i as f64;
    println!("F{i} = {}", ((phi.powf(f) - psi.powf(f)) / sqrt_5).round());
}
