/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Fast factorial algorithms with arbitrary precision and avoiding inefficient recursion.
/// Closures and functions are effectively interchangeable.
///
/// `let foo = |args| -> T {};` is equivalent to `fn foo(args) -> T {}`
//# Demo snippets with functions and closures, `ibig` cross-platform big-number crate.
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

let fac2 = |n: usize| -> UBig {
    successors(Some((ubig!(1), ubig!(1))), |(i, acc)| {
        Some((i + 1, acc * (i + 1)))
    })
    .map(|(a, b)| b)
    .nth(n - 1)
    .unwrap()
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
