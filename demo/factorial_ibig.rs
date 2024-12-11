/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Fast factorial algorithms with arbitrary precision and avoiding recursion.
/// A version using `std::Iterator::fold` and one using `std::iter::Successors:successors`
/// are executed and compared to ensure they agree before printing out the value.
/// Closures and functions are effectively interchangeable here.
///
/// `let foo = |args| -> T {};` is equivalent to `fn foo(args) -> T {}`
///
/// See also `demo/factorial_ibig_product.rs` for yet another version where we implement
/// the `std::iter::Product` trait on a wrapped `ibig::UBig` in order to use the
/// otherwise most concise, simple and approach. A very similar cross-platform implementation
/// without the need for such Product scaffolding (since `dashu` implements `Product`)
/// is `demo/factorial_dashu_product.rs`. The fastest by far is `demo/factorial_main_rug_product.rs`
/// backed by GNU libraries, but unfortunately it does not support the Windows MSVC, although it
/// may be possible to get it working with MSYS2.
///
/// Before running any benchmarks based on these scripts, don't forget that some of them
/// only run one algorithm while others are slowed down by running and comparing two different
/// algorithms.
//# Purpose: Demo snippets with functions and closures, `ibig` cross-platform big-number crate.
//# Categories: big_numbers, educational, math, recreational, technique
//# Sample arguments: `-- 50`
use ibig::{ubig, UBig};
use std::env;
use std::io::Read;
use std::iter::successors;

// Closure example using fold.
let fac_fold = |n: usize| -> UBig {
    if n == 0 {
        ubig!(0)
    } else {
        (1..=n).fold(ubig!(1), |acc: UBig, i: usize| acc * UBig::from(i))
    }
};

// Closure example using successors.
let fac_successors = |n: usize| -> UBig {
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

let fac_fold_n = fac_fold(n);

assert_eq!(fac_fold_n, fac_successors(n));
println!("factorial({n}) = {:#?}", fac_fold_n);
