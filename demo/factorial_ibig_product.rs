/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Fast factorial algorithm with arbitrary precision and avoiding inefficient recursion.
/// Closures and functions are effectively interchangeable.
///
/// Using the `std::iter::Product` trait is probably the most concise implementation,
/// but if you want to use it, but unlike the `rug` crate, `ibig` does not implement
/// the Product trait, so we have to wrap the `UBig`. Which of course is pretty verbose
/// in the context of a snippet, but could be useful in an app.
///
//# Demo snippet, `ibig` crate, factorial using `std::iter::Product` trait, workaround for implementing an external trait on an external crate.

use ibig::{ubig, UBig};
use std::env;
use std::io::Read;
use std::iter::successors;
use std::iter::Product;
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
        iter.fold(UBigWrapper(UBig::from(1u32)), |acc, x| {
            UBigWrapper(acc.0 * x.0)
        })
    }
}

impl<'a> Product<&'a UBigWrapper> for UBigWrapper {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(UBigWrapper(UBig::from(1u32)), |acc, x| {
            UBigWrapper(acc.0.clone() * x.0.clone())
        })
    }
}

// Function example using product
fn fac(n: usize) -> UBig {
    if n == 0 {
        UBig::from(0_usize)
    } else {
        (1..=n).map(|i| UBigWrapper(UBig::from(i))).product::<UBigWrapper>().0
    }
}

// Function example using successors
let fac3 = |n: usize| -> UBig {
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

let fac_n = fac(n);

assert_eq!(fac_n, fac3(n));
println!("factorial({n}) = {:#?}", fac_n);
