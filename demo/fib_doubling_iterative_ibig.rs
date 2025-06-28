/// Very fast non-recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_recursive.rs` for the
/// original recursive implementation and the back story.
///
/// This version is derived from `demo/fib_doubling_recursive_ibig.rs` with the following
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
//# Categories: big_numbers, learning, math, recreational, technique
//# Sample arguments: `-- 100`
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

    let mut required_indices: HashSet<usize> = HashSet::new();
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

    // eprintln!("stack={stack:#?}");

    // Sort indices in ascending order
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
            // F_{2k} = F_k x (F_{k-1} + F_{k+1})
            if i % 2 == 0 {
                let k = i / 2;
                let fk = &memo[&k];
                let fk_1 = &memo[&(k - 1)];
                let fk_2 = &memo[&(k + 1)];
                memo.insert(i, fk * (fk_1 + fk_2));
            } else {
                // F_{2k+1} = F_k^2 + F_{k+1}^2
                let k = (i - 1) / 2;
                let fk = &memo[&k];
                let fk_1 = &memo[&(k + 1)];
                memo.insert(i, fk * fk + fk_1 * fk_1);
            }
        }
    });

    let dur = start.elapsed();
    println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    let fib_n = &memo[&n];

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
