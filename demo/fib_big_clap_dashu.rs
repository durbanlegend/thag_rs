/*[toml]
[dependencies]
clap = { version = "4.5.3", features = ["derive"] }
dashu = "0.4.2"
*/

/// Fast Fibonacci with big integers, no recursion.
/// Should work with default Windows.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
/// Limited to Rust u128 types, thus it supports a maximum of fib(92) and fac(34).
///
/// The `fib` and `fac` closures could equally be implemented as functions here.
//# Purpose: Demonstrate snippets and a fast non-recursive fibonacci algorithm.

use clap::{Arg, Command};
use dashu::integer::IBig;
use std::iter::successors;

let matches = Command::new("fib_big_clap")
    .arg(
        Arg::new("number")
            .help("The numeric value to process")
            .required(true)
            .index(1),
    )
    .get_matches();

// Extract the parsed usize value
let n: usize = matches
    .get_one::<String>("number")
    .unwrap()
    .parse::<usize>()
    .unwrap();

// Snippet accepts function or closure
let fib = |n: usize| -> IBig {
    match n {
        0 => IBig::from(0),
       1 => IBig::from(1),
       _ =>
       {
        successors(Some((IBig::from(0), IBig::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
               .take(n)
               .last()
               .unwrap()
               .1
       }
    }
};

// (0..=n).for_each(|i| {
//     println!("Fibonacci F({i})={}", fib(i));
// });

println!("Fibonacci F({n}) is {}", fib(n));
