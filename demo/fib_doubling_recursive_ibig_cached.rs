/*[toml]
[dependencies]
serde = "=1.0.228"
syn = "2"
*/
//// Very fast recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_iterative.rs` and
/// `demo/fib_doubling_iterative_purge.rs` for non-recursive variations.
///
/// I'm sure this is old hat, but I stumbled across an apparent pattern in the
/// Fibonacci sequence:
/// `For m > n: Fm = Fn-1.Fm-n + Fn.Fm-n+1.`
///
/// This has a special case when m = 2n or 2n+1, which not surprisingly turn out
/// to be well-known "doubling identities". The related technique is known as
/// "fast doubling".
///
/// For even indices: `F2n = Fn x (Fn-1 + Fn+1)`.
/// For odd indices: `F2n+1 = Fn^2 + Fn+1^2`.
///
/// This allows us to compute a given Fibonacci number F2n or F2n+1 by recursively
/// or indeed iteratively expressing it in terms of Fn-1, Fn and Fn+1, or any two
/// of these since Fn+1 = Fn-1 + Fn.
///
/// Caching the recursive function gives a much simpler solution than memoization,
/// with comparable performance.
///
//# Purpose: Demo fast efficient Fibonacci with big numbers and limited, cached recursion.
//# Categories: big_numbers, learning, math, recreational, technique
//# Sample arguments: `-- 100`
use ibig::{ubig, UBig};
use std::time::Instant;
use syn;
use thag_demo_proc_macros::cached;

#[cached]
fn fib(n: usize) -> UBig {
    // eprintln!("Entered fib with new n={n}");
    if n == 0 {
        // eprintln!("Entered fib but returning n={n}");
        return ubig!(0);
    } else if n == 1 {
        // eprintln!("Entered fib but returning n={n}");
        return ubig!(1);
    }

    let result = if n % 2 == 0 {
        // F_{2k} = F_k x (2F_{k-1} + F_{k})
        let k = n / 2;
        let fk = fib(k);
        let fk1 = fib(k - 1);
        &fk * (&2 * fk1 + &fk)
    } else {
        // F_{2k+1} = F_k^2 + F_{k+1}^2
        let k = n / 2;
        let fk = fib(k);
        let fk1 = fib(k + 1);
        &fk * &fk + &fk1 * &fk1
    };

    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");
    let n_disp = n
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");

    let start = Instant::now();

    let fib_n = fib(n);

    let dur = start.elapsed();
    println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    let fib_n_str = fib_n.to_string();
    let l = fib_n_str.len();
    if l <= 100 {
        println!("F({n_disp}) len = {l}, value = {fib_n_str}");
    } else {
        println!(
            "F({n_disp}) len = {l}, value = {} ... {}",
            &fib_n_str[0..20],
            fib_n % (ubig!(10).pow(20))
        );
    }
}
