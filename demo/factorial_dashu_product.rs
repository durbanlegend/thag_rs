/*[toml]
[dependencies]
dashu = "0.4.2"
*/

/// Fast factorial algorithm with arbitrary precision and avoiding recursion.
/// Closures and functions are effectively interchangeable here.
///
///  Using the `std::iter::Product` trait - if implemented - is the most concise
/// factorial implementation. `dashu` implements it, so it's straightforward to use.
///
//# Demo snippet, `dashu` crate, factorial using `std::iter::Product` trait.

use dashu::ubig;
use dashu::integer::UBig;
use std::env;
use std::iter::{successors, Product};
use std::ops::{Deref, DerefMut};

// Function example using Product
fn fac_product(n: usize) -> UBig {
    if n == 0 {
        ubig!(0)
    } else {
        (1..=n).map(|i| UBig::from(i)).product::<UBig>()
    }
}

// Function example using successors
let fac_successors = |n: usize| -> UBig {
    successors(Some((ubig!(1), ubig!(1))), |(i, acc)| {
        Some((i + 1_u8, acc * (i + 1_u8)))
    })
    .map(|(_a, b)| b)
    .nth(n - 1)
    .unwrap()
};

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

let fac_prod_n = fac_product(n);

assert_eq!(fac_prod_n, fac_successors(n));
println!("factorial({n}) = {fac_prod_n}");
