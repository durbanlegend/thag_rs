/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_iterative.rs` for
/// a non-recursive variation.
///
/// I'm not sure of the theory and I'm sure this is well known, but I stumbled
/// across an apparent pattern in the Fibonacci sequence:
/// `For n > k: Fn = Fk-1.Fn-k + Fk.Fn-k+1.`
///
/// I noticed a special case when n = 2k or 2k+1, which ChatGPT tells me are the
/// well-known "doubling identities":
///
/// For even indices: `Fn = F2k = Fk x (Fk-1 + Fk+1)` = `2Fk-1.Fk + Fk^2`.
/// For odd indices: `Fn = F2k+1 = Fk^2 + Fk+1^2` = `Fk-1^2 + 2Fk-1.Fk + 2Fk^2`.
/// So we can always express Fn in terms of Fk-1 and Fk.
///
/// So we should be able to compute a given Fibonacci number F2k or F2k+1 by recursively
/// expressing it in terms of Fk-1, Fk and Fk+1.
///
/// I suggested this to ChatGPT, as well as the idea of pre-computing and storing the
/// first 10 or 100 Fibonacci numbers to save repeated recalculation. ChatGPT went
/// one better by memoizing all computed numbers. As there is a great deal of repetition
/// and fanning out of calls to fib(), the memoization drastically cuts down recursion.
///
//# Purpose: Demo fast efficient Fibonacci with big numbers, limited recursion, and memoization, and a good job by ChatGPT.
use ibig::{ubig, UBig};
use std::collections::HashMap;

fn fib(n: usize, memo: &mut HashMap<usize, UBig>) -> UBig {
    if let Some(result) = memo.get(&n) {
        eprintln!("Entered fib but found n={n}");
        return result.clone();
    }

    eprintln!("Entered fib with new n={n}");
    let result: UBig = if n % 2 == 0 {
        // F2k = 2Fk-1.Fk + Fk^2 = Fk(2Fk-1 + Fk)
        let k = n / 2;
        let fk = fib(k, memo);
        let fk1 = fib(k - 1, memo);
        &fk * (2 * &fk1 + &fk)
    } else {
        // F2k+1 = Fk-1^2 + 2Fk-1.Fk + 2Fk^2
        let k = n / 2;
        let fk = fib(k, memo);
        let fk1 = fib(k - 1, memo);
        &fk1 * &fk1 + 2 * &fk1 * (&fk1 + &fk)
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

    let result = fib(n, &mut memo);
    println!("Fibonacci number F({}) is {}", n, result);
}
