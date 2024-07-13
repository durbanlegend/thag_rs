#! /usr/bin/env /Users/donf/projects/rs-script/target/debug/rs_script
/*[toml]
[dependencies]
itertools = "0.12.1"
*/
/// Fast non-recursive fibonacci sequence calculation. Can't recall the exact source
/// but see for example https://users.rust-lang.org/t/fibonacci-sequence-fun/77495
/// for a variety of alternative approaches.
//# Purpose: Demo fast limited-scale fibonacci using Rust primitives and `itertools` crate.
use itertools::iterate;
use std::env;

let fib = |n: usize| -> usize {
    match n {
        0 => 0_usize,
        1 => 1_usize,
        _ => {
            iterate((0, 1), |&(a, b)| (b, a + b))
                .nth(n - 1)
                .unwrap()
                .1
        }
    }
};

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>, where 0 <= n <= 92", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid integer between 0 and 92");

let f = fib(n);
println!("Number {n} in the Fibonacci sequence is {f}");
