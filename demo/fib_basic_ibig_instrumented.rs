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
use std::iter::{successors, Successors, Take};
use std::time::Instant;

// Snippet accepts function or closure. This closure returns only the last value Fn.
fn fib_value_n(n: usize) -> UBig {
    successors(Some((ubig!(0), ubig!(1))), |(a, b)| {
        Some((b.clone(), (a + b).into()))
    })
    .map(|(a, _b)| a)
    .nth(n)
    .unwrap()
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");
let n_disp = n
    .to_string()
    .as_bytes()
    .rchunks(3)
    .rev()
    .map(std::str::from_utf8)
    .collect::<Result<Vec<&str>, _>>()
    .unwrap()
    .join(",");

let start = Instant::now();

// println!("Fibonacci F({n}) is {}", fib_value_n(n));
let fib_n = fib_value_n(n);

let dur = start.elapsed();
println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

let fib_n_str = fib_n.to_string();

if n <= 1000 {
    println!("F({n})={fib_n}");
} else if n >= 1000000 {
    println!("F({n_disp}) ends in ...{}", fib_n % ubig!(1000000000));
} else {
    let l = fib_n_str.len();
    println!(
        "F({}) = {}...{}",
        n_disp,
        &fib_n_str[0..20],
        &fib_n_str[l - 20..l - 1]
    );
}
