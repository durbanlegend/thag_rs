#! /usr/bin/env thag
/*[toml]
[dependencies]
itertools = "0.12.1"
*/
/// Fast non-recursive classic Fibonacci calculations for a specific value or an entire sequence.
/// I can't recall the exact source, but see for example https://users.rust-lang.org/t/fibonacci-sequence-fun/77495
/// for a variety of alternative approaches. The various Fibonacci scripts here in the demo
/// directory also show a number of approaches. `demo/fib_basic_ibig.rs` shows the use of
/// the `std::iter::Successors` iterator as well as removing the limitations of Rust
/// primitives. Most of the other examples explore different strategies for rapid computation of
/// large Fibonacci values, and hopefully demonstrate the usefulness of `thag_rs` as a tool
/// for trying out and comparing new ideas.
///
/// As the number of Fibonacci examples here shows, this took me down a Fibonacci rabbit hole.
//# Purpose: Demo fast small-scale fibonacci using Rust primitives and `itertools` crate.
//# Categories: learning, math, recreational, technique
//# Sample arguments: `-- 90`
use itertools::iterate;
use std::env;

// Closure that uses `itertools` to return a single Fibonacci value. We could just as well
// use a function.
let fib_value_n = |n: usize|
    iterate((0, 1), |&(a, b)| (b, a + b))
        .map(|(a, b): (usize, usize)| a)
        .nth(n)
        .unwrap();

let fib_series = |n: usize|
    iterate((0, 1), |&(a, b)| (b, a + b))
        .map(|(a, _b)| a)
        .take(n + 1);

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>, where 0 <= n <= 91", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid integer between 0 and 91");

// let f = fib(n);
// println!("Number {n} in the Fibonacci sequence is {f}");

// Manually working out the series in debug mode to check our work
#[cfg(debug_assertions)]
let (mut x, mut y) = (0, 1);

let mut i = 0;
let mut fib_series_n = 0;
for a in fib_series(n) {
    #[cfg(debug_assertions)]
    {
        assert_eq!(x, a);
        (x, y) = (y.clone(), x + y);
    }
    println!("Fibonacci F({i}) is {a}");
    if i == n {
        fib_series_n = a;
    }
    i += 1;
}
// Note that because of the different signatures, fib_series only calculates the series 0..=n once,
// while fib_value_n has to calculate o..=i from scratch for each i.
assert_eq!(fib_value_n(i - 1), fib_series_n);
