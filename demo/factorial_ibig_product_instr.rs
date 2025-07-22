/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling", "demo"] }
*/

/// A version of `demo/factorial_ibig_product.rs` instrumented for profiling.
///
/// Run this version in the normal way, then run `tools/thag_profile.rs` to analyse the profiling data.
//# Purpose: Demo `thag_rs` execution timeline and memory profiling.
//# Categories: profiling
//# Sample arguments: `-- 50`
use ibig::{ubig, UBig};
use std::env;
use std::iter::{successors, Product};
use std::ops::{Deref, DerefMut};
use thag_profiler::{enable_profiling, profiled, timing, visualization, AnalysisType, ProfileType};

// Step 1: Define the Wrapper Type
#[derive(Debug, Clone)]
struct UBigWrapper(UBig);

// Step 2: Implement Deref and DerefMut
impl Deref for UBigWrapper {
    type Target = UBig;
    #[profiled]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UBigWrapper {
    #[profiled]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Step 3: Implement the Product Trait
impl Product for UBigWrapper {
    #[profiled]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(UBigWrapper(ubig!(1)), |acc, x| UBigWrapper(acc.0 * x.0))
    }
}

impl<'a> Product<&'a UBigWrapper> for UBigWrapper {
    #[profiled]
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(UBigWrapper(ubig!(1)), |acc, x| {
            UBigWrapper(acc.0.clone() * x.0.clone())
        })
    }
}

// Function example using Product
#[profiled]
fn fac_product(n: usize) -> UBig {
    // Create a dummy object just to prove that memory profiling is happening here
    let _create_something = vec![
        "Hello".to_string(),
        "world".to_string(),
        "testing".to_string(),
        "testing".to_string(),
    ];

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
#[profiled]
fn fac_successors(n: usize) -> UBig {
    successors(Some((ubig!(1), ubig!(1))), |(i, acc)| {
        Some((i + 1, acc * (i + 1)))
    })
    .map(|(_a, b)| b)
    .nth(n - 1)
    .unwrap()
}

#[enable_profiling]
fn demo(n: usize) {
    let fac_prod_n = fac_product(n);

    assert_eq!(fac_prod_n, fac_successors(n));
    println!("factorial({n}) = {fac_prod_n}");
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

let run_analysis = async || {
    // Interactive visualization: must run AFTER function with `enable_profiling` profiling attribute,
    // because profile output is only available after that function completes.
    if let Err(e) = visualization::show_interactive_prompt(
        "factorial_ibig_product_instr",
        &ProfileType::Time,
        &AnalysisType::Flamechart,
    )
    .await
    {
        eprintln!("⚠️ Could not show interactive memory visualization: {e}");
    }
};

demo(n);

smol::block_on(run_analysis());
