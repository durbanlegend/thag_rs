/// Fast factorial algorithm with arbitrary precision and avoiding recursion.
/// Closures and functions are effectively interchangeable here.
///
/// Using the `std::iter::Product` trait - if implemented - is the most concise factorial
/// implementation. Unfortunately, but unlike the `dashu` and `rug` crates, `ibig` does
/// not implement the Product trait, so we have to wrap the `UBig`. Which of course
/// is pretty verbose in the context of a snippet, but could be useful in an app.
/// The implementation is thanks to GPT-4.
//# Purpose: Demo snippet, `ibig` crate, factorial using `std::iter::Product` trait, workaround for implementing an external trait on an external crate.
//# Categories: big_numbers, learning, math, recreational, technique
//# Sample arguments: `-- 50`
use ibig::{ubig, UBig};
use std::env;
use std::iter::{successors, Product};
use std::ops::{Deref, DerefMut};

// Step 1: Define the Wrapper Type
#[derive(Debug, Clone)]
struct UBigWrapper(UBig);

// Step 2: Implement Deref and DerefMut
impl Deref for UBigWrapper {
    type Target = UBig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UBigWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Step 3: Implement the Product Trait
impl Product for UBigWrapper {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(ubig!(1)), |acc, x| Self(acc.0 * x.0))
    }
}

impl<'a> Product<&'a Self> for UBigWrapper {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self(ubig!(1)), |acc, x| Self(acc.0 * x.0.clone()))
    }
}

// Function example using Product
fn fac_product(n: usize) -> UBig {
    if n == 0 {
        ubig!(0)
    } else {
        (1..=n).map(|i| UBigWrapper(UBig::from(i))).product::<UBigWrapper>().0
    }
}

// Function example using successors
let fac_successors = |n: usize| -> UBig {
    successors(Some((ubig!(1), ubig!(1))), |(i, acc)|
        Some((i + 1, acc * (i + 1))))
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
