/*[toml]
[dependencies]
ibig = "0.3.6"
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["core", "simplelog"] }
*/

/// A version of `demo/factorial_ibig_product.rs` converted to a program and instrumented for profiling using
/// `tools/profile_instr.rs`.
///
/// Run this version in the normal way, then run `tools/thag_profile.rs` to analyse the profiling data.
//# Purpose: Demo `thag_rs` execution timeline and memory profiling.
//# Categories: profiling
//# Sample arguments: `-- 50`
use ibig::{ubig, UBig};
use std::env;
use std::iter::{successors, Product};
use std::ops::{Deref, DerefMut};

use thag_rs::{enable_profiling, profile, profiling, Profile};

// Step 1: Define the Wrapper Type
#[derive(Debug, Clone)]
struct UBigWrapper(UBig);

// Step 2: Implement Deref and DerefMut
impl Deref for UBigWrapper {
    type Target = UBig;
    #[profile]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UBigWrapper {
    #[profile]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Step 3: Implement the Product Trait
impl Product for UBigWrapper {
    #[profile]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(UBigWrapper(ubig!(1)), |acc, x| UBigWrapper(acc.0 * x.0))
    }
}

impl<'a> Product<&'a UBigWrapper> for UBigWrapper {
    #[profile]
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(UBigWrapper(ubig!(1)), |acc, x| {
            UBigWrapper(acc.0.clone() * x.0.clone())
        })
    }
}

// Function example using Product
#[profile]
fn fac_product(n: usize) -> UBig {
    if n == 0 {
        ubig!(0)
    } else {
        (1..=n)
            .map(|i| UBigWrapper(UBig::from(i)))
            .product::<UBigWrapper>()
            .0
    }
}

// Function example using successors
#[profile]
fn fac_successors(n: usize) -> UBig {
    successors(Some((ubig!(1), ubig!(1))), |(i, acc)| {
        Some((i + 1, acc * (i + 1)))
    })
    .map(|(_a, b)| b)
    .nth(n - 1)
    .unwrap()
}

#[enable_profiling]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let fac_prod_n = fac_product(n);

    assert_eq!(fac_prod_n, fac_successors(n));
    println!("factorial({n}) = {fac_prod_n}");
}
