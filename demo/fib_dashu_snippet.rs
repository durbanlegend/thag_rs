/*[toml]
[dependencies]
dashu = "0.4.2"
*/

/// Fast non-recursive Fibonacci sequence calculation with big integers.
/// Should work with default Windows.
///
/// Based on discussion https://users.rust-lang.org/t/fibonacci-sequence-fun/77495
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demonstrate snippets, a fast non-recursive fibonacci algorithm using `successors`, and zipping 2 iterators together.
//# Categories: big_numbers, learning, math, recreational, technique
// Given a non-negative integer n, return an iterator of fibonacci pairs (F0, F1) ... (Fn, Fn+1).
//# Sample arguments: `-- 100`
use dashu::ubig;
use dashu::integer::UBig;
use std::env;
use std::iter::successors;
use std::str::FromStr;

fn fib_fn(n: usize) -> impl Iterator<Item = (UBig, UBig)> {
    successors(Some((ubig!(0), ubig!(1))), |(a, b)| Some((b.clone(), (a + b).into())))
   .take(n + 1)
}

let fib_closure = |n: usize| successors(Some((ubig!(0), ubig!(1))), |(a, b)| Some((b.clone(), (a + b).into())))
    .take(n + 1);

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

let mut i = 0;
// Compare and print the answers from the function and the closure.
for (x, y) in fib_fn(n).zip(fib_closure(n)) {
    assert_eq!(x, y);
    println!("F{i} = {}", x.0);
    i += 1;
}
