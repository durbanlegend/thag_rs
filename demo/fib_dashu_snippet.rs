/*[toml]
[dependencies]
dashu = "0.4.2"
*/

use dashu::integer::IBig;
use std::env;
use std::iter::successors;
use std::str::FromStr;

/// Fast Fibonacci with big integers, no recursion. Should work with default Windows.
///
/// Based on discussion https://users.rust-lang.org/t/fibonacci-sequence-fun/77495
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demonstrate snippets, a fast non-recursive fibonacci algorithm using `successors`, and zipping 2 iterators together.
// Given a non-negative integer n, return an iterator of fibonacci pairs (F0, F1) ... (Fn, Fn+1).
fn fib_fn(n: usize) -> impl Iterator<Item = (IBig, IBig)> {
    successors(Some((IBig::from(0), IBig::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
   .take(n + 1)
}

let fib_closure = |n: usize| successors(Some((IBig::from(0), IBig::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
    .take(n + 1);

// Some simple CLI args requirements...
let n = if let Some(arg) = env::args().nth(1) {
    // usize::from(arg)
    if let Ok(n) = arg.parse::<usize>() {
        n
    } else {
        println!("Usage: {} <n>", env::args().nth(0).unwrap());
        return Ok(());
    }
}
else {
    println!("Usage: {} <n>", env::args().nth(0).unwrap());
    return Ok(());
};

println!("fib_closure({n}) = ");
for (a, b) in fib_closure(n) {
    println!("({a}, {b})");
}

println!();

let mut i = 0;
// Compare and print the answers from the function and the closure.
for (x, y) in fib_fn(n).zip(fib_closure(n)) {
    assert_eq!(x, y);
    println!("F{i} = {}", x.0);
    i += 1;
}
