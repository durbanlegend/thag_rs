/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Try a version based on reverse engineering the `fibo_new / fibo_new_work` functions of `demo/fib_4784969_cpp_ibig.rs`
/// This approach passes the pair `Fn, Fn+1` `(a, b)` and multiplies both elements by a common multiplier `2b - a`, before
/// adjusting `b` by `(-1)^n`.
/// This algorithm can be derived from Cassini's identity: `Fn^2 = Fn-1.Fn+1 - (-1)^n`. I'll pay my dues here by doing
/// the derivation.
///
/// Starting with the usual formulae used by doubling methods:
///
/// `For even indices: F2n  = 2Fn.Fn+1 - Fn^2
///
///                         = Fn(2Fn+1 - Fn).   // i.e. a(2b - a)
///
/// For odd indices:  F2n+1 = Fn^2 + Fn+1^2.
///
/// To the odd-index case we apply Cassini's identity: Fn^2 = Fn-1.Fn+1 - (-1)^n:
///
/// F2n+1 = Fn+1^2 + Fn^2 +
///
///       = Fn+1^2 + Fn+1Fn-1 - (-1)^n          // since by Cassini Fn^2 = Fn-1.Fn+1 - (-1)^n
///
///       = Fn+1^2 + Fn+1(Fn+1 - Fn) - (-1)^n   // substituting for Fn-1
///
///       = 2Fn+1^2 - Fn.Fn+1 - (-1)^n
///
///       = Fn+1(2Fn+1 - Fn) - (-1)^n           // i.e. b(2b - a) - (-1)^n
//# Purpose: Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.
use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn fib(k: usize, (fk, fk1): (&UBig, &UBig)) -> (UBig, UBig) {
    if k == 0 {
        eprintln!("Entered fib but returning (ubig!(0), ubig!(1))");
        return (ubig!(0), ubig!(1));
    }

    let (a, b) = fib(k / 2, (fk, fk1));

    let mult = 2 * &b - &a;
    let subtr: i32 = (-1_i32).pow(k as u32 % 2).try_into().unwrap();
    let (c, d): (UBig, UBig) = if k % 4 == 0 {
        // a even, b odd
        ((&a * &mult), (&b * &mult - subtr))
    } else {
        // a odd, b even
        ((&a * &mult - subtr), (&b * &mult))
    };

    eprintln!("k={k}, a={a}, b={b}, mult={mult}, c={c}, d={d}");
    (c, d)
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

    let fib_n = fib(n, (&ubig!(0), &ubig!(1))).1;

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
