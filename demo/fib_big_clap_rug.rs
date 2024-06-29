/*[toml]
[dependencies]
clap = { version = "4.5.3", features = ["derive"] }
rug = { version = "1.24.0", features = ["integer"] }
*/

/// Fast Fibonacci with big integers, no recursion.
/// Won't work with default Windows 11 because of rug crate
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
/// The `fib` and `fac` closures could equally be implemented as functions here.
//# Purpose: Demonstrate snippets and a fast non-recursive fibonacci algorithms.

use clap::{Arg, Command};
use rug::Integer;
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
let fib = |n: usize| -> Integer {
    match n {
        0 => Integer::from(0),
       1 => Integer::from(1),
       _ =>
       {
        successors(Some((Integer::from(0), Integer::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
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
