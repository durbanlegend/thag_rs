/*[toml]
[dependencies]
clap = { version = "4.5.3", features = ["derive"] }
rug = { version = "1.24.0", features = ["integer"] }
*/

/// Fast Fibonacci with big integers, no recursion.
/// Won't work with default Windows 11.
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
    successors(Some((Integer::from(0), Integer::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
        .take(n + 1)
        .last()
        .unwrap()
        .0
};

println!("Value number {n} in the Fibonacci sequence is {}", fib(n));
