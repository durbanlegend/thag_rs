/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast non-recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_recursive.rs` for the
/// original recursive implementation and the back story.
///
/// This version is derived from `demo/fib_doubling_recursive.rs` with the following
/// changes:
///
/// 1. Instead of calculating the `Fi` values in descending order as soon as they are
/// identified, add them to a list and then calculate them from the list in ascending
/// order.
///
/// 2. The list tends to end up containing strings of 3 or more commonly 4 consecutive
/// `i` values for which `Fi` must be calculated. For any `i` that is the 3rd or
/// subsequent entry in such a consecutive run, that is, for which Fi-2 and Fi-1 have
/// already been calculated, compute Fi cheaply as Fi-2 + Fi-1 instead of using the
/// normal multiplication formula.
//# Purpose: Demo fast efficient Fibonacci with big numbers, no recursion, and memoization, and ChatGPT implementation.
use ibig::ubig;
use std::collections::{HashMap, HashSet};
use std::env;
use std::iter::successors;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let mut required_indices = HashSet::new();
    let mut stack = vec![n];
    let cached = 100;

    // Identify all necessary indices
    while let Some(i) = stack.pop() {
        // eprintln!("Popped i={i}");
        if i > cached {
            required_indices.insert(i);
            if i % 2 == 0 {
                let k = i / 2;
                for j in (k - 1)..=(k + 1) {
                    if j > cached && !required_indices.contains(&j) {
                        stack.push(j);
                        // eprintln!("Pushed {j}");
                    }
                }
            } else {
                let k = (i - 1) / 2;
                for j in k..=(k + 1) {
                    if j > cached && !required_indices.contains(&j) {
                        stack.push(j);
                        // eprintln!("Pushed {j}");
                    }
                }
            }
        }
    }

    let mut sorted_indices: Vec<_> = required_indices.into_iter().collect();
    sorted_indices.sort();
    eprintln!("sorted_indices={sorted_indices:#?}");

    let mut memo = HashMap::new();
    let fib_series = |n: usize| {
        successors(Some((ubig!(0), ubig!(1))), |(a, b)| {
            Some((b.clone(), (a + b).into()))
        })
        .map(|(a, _b)| a)
        .take(n + 1)
    };

    let mut i = 0;
    for a in fib_series(cached) {
        memo.insert(i, a);
        i += 1;
    }

    for &i in &sorted_indices {
        if i > 1 {
            let result = if i % 2 == 0 {
                let k = i / 2;
                eprintln!("i={i}, need k={k} and k - 1={}", k - 1);
                let fk = memo[&k].clone();
                let fk_1 = memo[&(k - 1)].clone();
                &fk * (fk_1 + &fk)
            } else {
                let k = (i - 1) / 2;
                let fk = memo[&k].clone();
                let fk_1 = memo[&(k + 1)].clone();
                &fk * &fk + &fk_1 * &fk_1
            };

            memo.insert(i, result.clone());
            eprintln!("Memoised {i}");

            // Purge unnecessary values
            if i % 2 == 1 {
                let k = (i - 1) / 2;
                // memo.remove(&k);
                // eprintln!("Removed k={k}");
                if k > 0 {
                    memo.remove(&(k - 1));
                    eprintln!("Removed k - 1={}", k - 1);
                }
            } else {
                let k = i / 2;
                if k > 1 {
                    memo.remove(&(k - 2));
                    eprintln!("Removed k - 2={}", k - 2);
                }
            }
        }
    }

    // println!("F{} = {}", n, memo[&n]);
}
