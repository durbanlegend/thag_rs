/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast non-recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_recursive.rs` for the
/// original recursive implementation and the back story.
///
/// This version is derived from `demo/fib_doubling_iterative.rs` with the following
/// change: that we reduce bloat as best we can  by purging redundant entries from the memo
/// cache as soon as it's safe to do so.
//# Purpose: Demo fast efficient Fibonacci with big numbers, no recursion, and memoization, and ChatGPT implementation.
use ibig::ubig;
use std::collections::{HashMap, HashSet};
use std::env;
use std::iter::successors;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let start = Instant::now();
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
    // eprintln!("sorted_indices={sorted_indices:#?}");

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

    let mut purged_cache = false;
    let len = sorted_indices.len();
    let mut fib_n = ubig!(0);
    sorted_indices.iter().enumerate().for_each(|(index, &i)| {
        if i == 0 || i == 1 {}

        // If the 2 prior numbers are in the list, simply create this one
        // by adding them according to the definition of F(i).
        if index > 1 && sorted_indices[index - 2] == i - 2 && sorted_indices[index - 1] == i - 1 {
            // F_n = F_{n-2} + F_{n-1})
            let fi_2 = &memo[&(i - 2)];
            let fi_1 = &memo[&(i - 1)];
            memo.insert(i, fi_2 + fi_1);
        } else {
            if i % 2 == 0 {
                let k = i / 2;
                // eprintln!("i={i}, need {}, {k} and {}", k - 1, k + 1);
                // F_{2k} = F_k x (F_{k-1} + F_{k+1})
                let fk = &memo[&k];
                let fk_1 = &memo[&(k - 1)];
                let fk_2 = &memo[&(k + 1)];
                let fib = fk * (fk_1 + fk_2);
                if index == len - 1 {
                    fib_n = fib;
                    // let dur = start.elapsed();
                    // println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());
                } else {
                    memo.insert(i, fib);
                }
            } else {
                // F_{2k+1} = F_k^2 + F_{k+1}^2
                let k = (i - 1) / 2;
                // eprintln!("i={i}, need {k} and {}", k + 1);
                let fk = &memo[&k];
                let fk_1 = &memo[&(k + 1)];
                let fib = fk * fk + fk_1 * fk_1;
                if index == len - 1 {
                    fib_n = fib;
                    // let dur = start.elapsed();
                    // println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());
                } else {
                    memo.insert(i, fib);
                }
            }
        }

        // Purge unnecessary values
        if i % 2 == 1 {
            let k = (i - 1) / 2;
            if !(index + 1 < sorted_indices.len() && sorted_indices[index + 1] == i + 1) {
                memo.remove(&k);
                memo.remove(&(k + 1));
                // eprintln!("Removed k={k} and k+1={}", k + 1);
            }
        } else {
            let k = i / 2;
            if !(index + 1 < sorted_indices.len() && sorted_indices[index + 1] == i + 1) {
                memo.remove(&k);
                memo.remove(&(k + 1));
                // eprintln!("Removed k={k} and k+1={}", k + 1);
            }
            memo.remove(&(k - 1));
            // eprintln!("Removed k-1={}", k - 1);
            if k > 1 && memo.contains_key(&(k - 2)) {
                memo.remove(&(k - 2));
                // eprintln!("Removed k-2={}", k - 2);
            }
        }

        if !purged_cache && i > &cached * 2 + 2 {
            for j in 0..=cached {
                if memo.contains_key(&j) {
                    memo.remove(&j);
                    // eprintln!("Removed j={}", j);
                }
            }
            purged_cache = true;
        }
    });

    // eprintln!("memo.keys()={:#?}", memo.keys());
    // println!("F{} = {}", n, memo[&n]);

    let dur = start.elapsed();
    println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else {
        let fib_n = fib_n.to_string();
        let l = fib_n.len();
        println!("F({}) = {}...{}", n, &fib_n[0..20], &fib_n[l - 20..l - 1]);
    }
}
