/// Fast factorial algorithm avoiding recursion, but limited to a maximum of `34!` by using only
/// Rust primitives.
//# Purpose: Demo fast limited-scale factorial using Rust primitives and std::iter::Product trait.
//# Categories: educational, math, recreational, technique
use std::env;
use std::io::Result;

fn main() -> Result<()> {
    let fac = |n: u128| -> u128 {
        if n == 0 {
            0
        } else {
            (1..=n).product()
        }
    };

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>, where 0 <= n <= 34", args[0]);
        std::process::exit(1);
    }

    let n: u128 = args[1]
        .parse()
        .expect("Please provide a valid integer between 0 and 34");

    println!("fac({n}) = {}", fac(n));
    Ok(())
}
