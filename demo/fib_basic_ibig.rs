/*[toml]
[dependencies]
ibig = "0.3.6"
itertools = "0.13.0"
*/
/// Big-number (and thus more practical) version of `demo/fib_basic.rs`.
///
//# Purpose: Demo using a big-number crate to avoid the size limitations of primitive integers.
//# Categories: big_numbers, learning, math, recreational, technique
//# Sample arguments: `-- 100`
use ibig::{ubig, UBig};
use itertools::iterate;
use std::env;

// Closure that uses `itertools` to return a single Fibonacci value. We could just as well
// use a function.
let fib_value_n = |n: usize|
    iterate((ubig!(0), ubig!(1)), |(a, b)| (b.clone(), a + b))
        .map(|(a, b)| a)
        .nth(n)
        .unwrap();

let fib_series = |n: usize|
    iterate((ubig!(0), ubig!(1)), |(a, b)| (b.clone(), a + b))
        .map(|(a, _b)| a)
        .take(n + 1);

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>, where n >=0", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a positive integer");

// let f = fib(n);
// println!("Number {n} in the Fibonacci sequence is {f}");

// Manually working out the series in debug mode to check our work
#[cfg(debug_assertions)]
// let Some((mut x, mut y)) = Some((ubig!(0), ubig!(1)));
let (mut x, mut y) = (ubig!(0), ubig!(1));

let mut i = 0;
let mut fib_series_n = ubig!(0);
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
