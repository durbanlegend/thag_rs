/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Try a version based on reverse engineering the `fibo_new / fibo_new_work` functions of `demo/fib_4784969_cpp_ibig.rs`
/// This approach passes the pair `Fn, Fn+1` `(a, b)` and applies some funky calculations. I'll pay my dues here by doing
/// the derivation.
///
/// This version uses immutable arguments to the `fib` method.
///
/// Starting with the usual formulae used by doubling methods.
/// For even indices:
///
///     F2n  = 2Fn.Fn+1 - Fn^2
///
///          = Fn(2Fn+1 - Fn).   // i.e. a(2b - a)
///
/// For odd indices:
///
///     F2n+1 = Fn^2 + Fn+1^2.
///
/// To the odd-index case we apply Cassini's identity: Fn^2 = Fn-1.Fn+1 - (-1)^n:
///
///     F2n+1 = Fn+1^2 + Fn^2 +
///
///           = Fn+1^2 + Fn+1Fn-1 - (-1)^n          // since by Cassini Fn^2 = Fn-1.Fn+1 - (-1)^n
///
///           = Fn+1^2 + Fn+1(Fn+1 - Fn) - (-1)^n   // substituting for Fn-1
///
///           = 2Fn+1^2 - Fn.Fn+1 - (-1)^n
///
///           = Fn+1(2Fn+1 - Fn) - (-1)^n           // i.e. b(2b - a) - (-1)^n
///
/// If n is odd, then a = F2n+1 and b = 2Fn+2, so we must derive the latter:
///
///     F2n+2 = F2m where m = n+1 = Fm(2Fm+1 - Fm)
///
///           = Fn+1(2F(n+2) - Fn+1)
///
///           = Fn+1(2Fn+1 + 2Fn - Fn+1)            // Since Fn+2 = Fn + Fn+1
///
///           = Fn+1(Fn+1 + 2Fn)                    // i.e. b(b+2a)
//# Purpose: Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.
//# Categories: big_numbers, educational, math, recreational, technique
//# Sample arguments: `-- 100`
use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn fib(k: usize, (a, b): (&UBig, &UBig)) -> (UBig, UBig) {
    if k == 0 {
        eprintln!("Entered fib but returning (ubig!(0), ubig!(1))");
        return (ubig!(0), ubig!(1));
    }
    if k == 1 {
        eprintln!("Entered fib but returning (ubig!(1), ubig!(2)");
        return (ubig!(1), ubig!(2));
    }

    let j = k / 2;
    let (a, b) = fib(j, (a, b));

    // Now if k is odd, then k = 2j + 1, thus a = F2j+1 and b = F2j_2

    // let subtr: i32 = (-1_i32).pow(k as u32 % 2).try_into().unwrap();
    let (c, d): (UBig, UBig) = if k % 2 == 0 {
        // a is F2j, b is F2j+1
        let mult1 = 2 * &b - &a;
        (&a * &mult1, &b * &b + &a * &a)
    } else {
        // a is F2j+1, b is F(2(j+1))
        let mult2 = 2 * &a + &b;
        let new_b = &a + &b;
        (&b * &mult2, &b * &b + &new_b * &new_b)
    };

    // eprintln!("k={k}, a={a}, b={b}, mult1={mult1}, mult2={mult2}, c={c}, d={d}");
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

    let fib = fib(n / 2, (&ubig!(0), &ubig!(1)));
    let fib_n = if n % 2 == 0 { fib.0 } else { fib.1 };
    // let fib_n = fib.0;

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
