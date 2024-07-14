/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast tail-recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity.
///
/// I'm not sure of the theory and I'm sure this is well known, but I stumbled
/// across an apparent pattern in the Fibonacci sequence:
/// `For m > n: Fm = Fn-1.Fm-n + Fn.Fm-n+1.`
///
/// I noticed a special case when m = 2n or 2n+1, which ChatGPT tells me are the
/// well-known "doubling identities":
///
/// For even indices: `F2n = Fn x (Fn-1 + Fn+1)`.
/// For odd indices: `F2n+1 = Fn^2 + Fn+1^2`.
///
/// So we should be able to compute a given Fibonacci number F2n or F2n+1 recursively
/// expressing it in terms of Fn-1, Fn and Fn+1.
///
/// I suggested this and memoizing the first 10 or 100 Fibonacci numbers to ChatGPT,
/// which went one better by memoizing all computed numbers. As there is a great deal
/// of repetition and fanning out of calls to fib() the memoization drastically cuts down recursion
///
//# Purpose: Demo fast efficient Fibonacci with big numbers and limited recursion, and a good job by ChatGPT.
use ibig::{ubig, UBig};
use std::collections::HashMap;

fn fib(n: usize, memo: &mut HashMap<usize, UBig>) -> UBig {
    if let Some(result) = memo.get(&n) {
        // eprintln!("Entered fib but found n={n}");
        return result.clone();
    }

    // eprintln!("Entered fib with new n={n}");
    let result = if n % 2 == 0 {
        // F_{2n} = F_n \cdot (F_{n-1} + F_{n+1})
        let k = n / 2;
        let fk = fib(k, memo);
        let fk1 = fib(k - 1, memo);
        let fk2 = fib(k + 1, memo);
        &fk * (&fk1 + &fk2)
    } else {
        // F_{2n+1} = F_n^2 + F_{n+1}^2
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
