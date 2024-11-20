/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast recursive calculation of an individual Fibonacci number using the
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
/// I suggested this to ChatGPT, as well as the idea of pre-computing and storing the
/// first 10 or 100 Fibonacci numbers to save repeated recalculation. ChatGPT went
/// one better by memoizing all computed numbers. As there is a great deal of repetition
/// and fanning out of calls to fib(), the memoization drastically cuts down recursion.
///
//# Purpose: Demo fast efficient Fibonacci with big numbers, limited recursion, and memoization, and a good job by ChatGPT.
//# Categories: big_numbers, educational, math, recreational, technique
use ibig::{ubig, UBig};
use std::collections::HashMap;
use std::time::Instant;

fn fib(n: usize, memo: &mut HashMap<usize, UBig>) -> UBig {
    if let Some(result) = memo.get(&n) {
        // eprintln!("Entered fib but found n={n}");
        return result.clone();
    }

    // eprintln!("Entered fib with new n={n}");
    let result = if n % 2 == 0 {
        // F_{2k} = F_k x (2F_{k-1} + F_{k})
        let k = n / 2;
        let fk = fib(k, memo);
        let fk1 = fib(k - 1, memo);
        &fk * (&2 * fk1 + &fk)
    } else {
        // F_{2k+1} = F_k^2 + F_{k+1}^2
        let k = n / 2;
        let fk = fib(k, memo);
        let fk1 = fib(k + 1, memo);
        &fk * &fk + &fk1 * &fk1
    };

    memo.insert(n, result.clone());
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

    // Precompute and store base Fibonacci numbers
    let mut memo: HashMap<usize, UBig> = HashMap::new();
    memo.insert(0, ubig!(0));
    memo.insert(1, ubig!(1));
    memo.insert(2, ubig!(1));
    memo.insert(3, ubig!(2));
    memo.insert(4, ubig!(3));
    memo.insert(5, ubig!(5));
    memo.insert(6, ubig!(8));
    memo.insert(7, ubig!(13));
    memo.insert(8, ubig!(21));
    memo.insert(9, ubig!(34));
    memo.insert(10, ubig!(55));

    let fib_n = fib(n, &mut memo);

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
