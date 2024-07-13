/*[toml]
[dependencies]
clap = { version = "4.5.3", features = ["derive"] }
rug = { version = "1.24.0", features = ["integer"] }
*/

/// Fast Fibonacci with big integers, no recursion.
/// Won't work with default Windows 11 because of `rug` crate.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
/// The `fib_series` closure could equally be implemented as a function here,
/// but closure is arguably easier as you don't have to know or figure out the
/// exact return type (`impl Iterator<Item = Integer>` if you're wondering).
///
/// Using `clap` here is complete overkill, but this is just a demo.
//# Purpose: Demonstrate snippets, closures, `clap` builder and a fast non-recursive fibonacci algorithm using the `successors`.
use clap::{Arg, Command};
use rug::Integer;
use std::iter::{successors, Successors, Take};

let matches = Command::new("fib_big_clap_rug")
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

// Snippet accepts function or closure. This closure returns only the last value Fn.
fn fib_value_n(n: usize) -> Integer {
    successors(Some((Integer::from(0), Integer::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
        .nth(n)
        .unwrap()
        .0
}

// Same formula, but we return the whole series from F0 to Fn. Using a closure is
// easier than a function in this case, as we don't have to bother specifying the
// return type `Take<Successors<Integer, _>>` nor do we need braces since the
// closure contains only one statement and the return type is not specified.
// If you want all values in the series, using this version is much more efficient
// than repeatedly calling fn `fib_value_n`. Obvious maybe, but easily overlooked.
let fib_series = |n: usize|
    successors(Some((Integer::from(0), Integer::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
   .take(n + 1)
   .map(|(a, b)| a);

// Manually working out the series in debug mode to check our work
#[cfg(debug_assertions)]
let (mut x, mut y) = (Integer::from(0), Integer::from(1));

let mut i = 0;
let mut fib_series_n = Integer::from(0);
// for (a: Integer, b: Integer) in fib_series(n) {
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
