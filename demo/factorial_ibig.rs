/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Fast factorial algorithms with arbitrary precision and avoiding inefficient recursion.
/// Closures and functions are effectively interchangeable.
///
/// `let foo = |args| -> T {};` is equivalent to `fn foo(args) -> T {}`
//# Demo snippets with functions and closures, featured cross-platform big-number crate.

use ibig::{ubig, UBig};
use std::env;
use std::io::Read;
use std::iter::successors;
// Closure example using fold.
let fac1 = |n: usize| -> UBig {
    if n == 0 {
        ubig!(0)
    } else {
        (1..=n).fold(ubig!(1), |acc: UBig, i: usize| acc * UBig::from(i))
    }
};

// Function example using successors
// Can't substitute this in initial values (which hardly matter anyway)
// without getting further down a deferencing rabbit hole and ending up cloning.
let ubig_1 = ubig!(1);

// Using successors is possible, but turns out pretty inscrutable
let fac2 = |n: usize| -> UBig {
    successors(Some((ubig!(1), ubig!(1))), |(a, b)| {
        Some(((*&a + &ubig_1), (*&a + &ubig_1) * b))
    })
        .take(n)
        .last()
        .unwrap()
        .1
};

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

let fac1_n = fac1(n);

assert_eq!(fac1_n, fac2(n));
println!("factorial({n}) = {:#?}", fac1_n);
