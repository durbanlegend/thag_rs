/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast non-recursive calculation of an individual Fibonacci number using the
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
//# Purpose: Demo fast efficient Fibonacci with big numbers and no recursion, and a good job by ChatGPT.
use ibig::ubig;
use std::collections::HashSet;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let mut required_indices = HashSet::new();
    let mut i = n;

    // Identify required indices
    while i > 1 {
        required_indices.insert(i);
        if i % 2 == 0 {
            i /= 2;
        } else {
            i = (i - 1) / 2;
        }
    }
    required_indices.insert(0);
    required_indices.insert(1);

    // Sort indices in ascending order
    let mut sorted_indices: Vec<_> = required_indices.into_iter().collect();
    sorted_indices.sort();
    eprintln!("sorted_indices={sorted_indices:#?}");

    let mut memo = std::collections::HashMap::new();
    memo.insert(0, ubig!(0));
    memo.insert(1, ubig!(1));

    // Compute and memoize Fibonacci numbers
    for &i in &sorted_indices {
        if i == 0 || i == 1 {
            continue;
        }

        if i % 2 == 0 {
            let k = i / 2;
            eprintln!("i={i}, k={k}, k-1={}", k - 1);

            let fk = &memo[&k];
            let fk_1 = &memo[&(k - 1)];
            memo.insert(i, fk * (fk_1 + fk));
        } else {
            let k = (i - 1) / 2;
            eprintln!("i={i}, k={k}, k+1={}", k + 1);
            let fk = &memo[&k];
            let fk_1 = &memo[&(k + 1)];
            memo.insert(i, fk * fk + fk_1 * fk_1);
        }
    }

    println!("F{} = {}", n, memo[&n]);
}
