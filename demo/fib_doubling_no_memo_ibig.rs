/// A version of `demo/fib_doubling_recursive.rs`, minus the memoization.
/// This serves to prove that the memoization is faster, although
/// not dramatically so.
///
//# Purpose: Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.
//# Categories: big_numbers, learning, math, recreational, technique
//# Sample arguments: `-- 100`
use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn fib(n: usize) -> UBig {
    if n == 0 {
        // eprintln!("Entered fib but returning n={n}");
        return ubig!(0);
    } else if n == 1 {
        // eprintln!("Entered fib but returning n={n}");
        return ubig!(1);
    }

    // eprintln!("Entered fib with n={n}");
    if n % 2 == 0 {
        let k = n / 2;
        let fk = fib(k);
        let fk_minus_1 = fib(k - 1);
        &fk * (2 * &fk_minus_1 + &fk)
    } else {
        let k = (n + 1) / 2;
        let fk = fib(k);
        let fk_minus_1 = fib(k - 1);
        &fk * &fk + &fk_minus_1 * &fk_minus_1
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
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
