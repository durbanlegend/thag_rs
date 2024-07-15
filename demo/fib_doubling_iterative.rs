/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast non-recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_recursive.rs` for the
/// original recursive implementation and the back story.
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

    let mut required_indices: HashSet<usize> = HashSet::new();
    let mut stack = vec![n];
    let cached = 100;

    // Identify all necessary indices
    while let Some(i) = stack.pop() {
        eprintln!("Popped i={i}");
        if i > cached {
            required_indices.insert(i);
            if i % 2 == 0 {
                let k = i / 2;
                for j in (k - 1)..=(k + 1) {
                    if j > cached && !required_indices.contains(&j) {
                        stack.push(j);
                        eprintln!("Pushed {j}");
                    }
                }
            } else {
                let k = (i - 1) / 2;
                for j in k..=(k + 1) {
                    if j > cached && !required_indices.contains(&j) {
                        stack.push(j);
                        eprintln!("Pushed {j}");
                    }
                }
            }
        }
    }

    eprintln!("stack={stack:#?}");

    // Sort indices in ascending order
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

    // Compute and memoize Fibonacci numbers
    for &i in &sorted_indices {
        if i == 0 || i == 1 {
            continue;
        }

        if i % 2 == 0 {
            let k = i / 2;
            let fk = &memo[&k];
            let fk_1 = &memo[&(k - 1)];
            let fk_2 = &memo[&(k + 1)];
            memo.insert(i, fk * (fk_1 + fk_2));
        } else {
            let k = (i - 1) / 2;
            let fk = &memo[&k];
            let fk_1 = &memo[&(k + 1)];
            memo.insert(i, fk * fk + fk_1 * fk_1);
        }
    }

    println!("F{} = {}", n, memo[&n]);
}
