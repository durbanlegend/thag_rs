/*[toml]
[dependencies]
rug = "1.24.1"
*/

/// Very fast non-recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_recursive.ibig.rs` for the
/// original recursive implementation and the back story.
/// Won't work with default Windows 11 because of `rug` crate.
/// On Linux you may need to install the m4 package.
///
/// This version is derived from `demo/fib_doubling_iterative.rs` with the following
/// change: that we reduce bloat as best we can  by purging redundant entries from the memo
/// cache as soon as it's safe to do so.
//# Purpose: Demo fast efficient Fibonacci with big numbers, no recursion, and memoization, and ChatGPT implementation.
use rug::ops::Pow;
use rug::{Assign, Integer};
use std::collections::{HashMap, HashSet};
use std::env;
use std::iter::successors;
use std::time::Instant;

fn invert_order(n: usize, cached: usize) -> Vec<usize> {
    let mut required_indices = HashSet::new();
    let mut stack = vec![n];

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
    sorted_indices
}

fn fib(n: usize, cached: usize, sorted_indices: &[usize]) -> Integer {
    if n < cached {
        return successors(Some((Integer::from(0), Integer::from(1))), |(a, b)| {
            Some((b.clone(), (a + b).into()))
        })
        .map(|(a, _b)| a)
        .nth(n)
        .expect("Fib failed");
    }

    let mut memo = HashMap::new();
    let fib_series = |n: usize| {
        successors(Some((Integer::from(0), Integer::from(1))), |(a, b)| {
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
    let mut fib_n = Integer::from(0);
    sorted_indices.iter().enumerate().for_each(|(index, &i)| {
        if i == 0 || i == 1 {}

        // If the 2 prior numbers are in the list, simply create this one
        // by adding them according to the definition of F(i).
        if index > 1 && sorted_indices[index - 2] == i - 2 && sorted_indices[index - 1] == i - 1 {
            // F_n = F_{n-2} + F_{n-1})
            let fi_2 = &memo[&(i - 2)];
            let fi_1 = &memo[&(i - 1)];
            memo.insert(i, Integer::from(fi_2 + fi_1));
        } else {
            if i % 2 == 0 {
                let k = i / 2;
                // eprintln!("i={i}, need {}, {k} and {}", k - 1, k + 1);
                // F_{2k} = F_k x (F_{k-1} + F_{k+1})
                let fk = &memo[&k];
                let fk_1 = &memo[&(k - 1)];
                let fk_2 = &memo[&(k + 1)];
                let mut temp = Integer::new();
                temp.assign(fk_1 + fk_2);
                let fib = Integer::from(fk * temp);
                if index == len - 1 {
                    fib_n.assign(fib);
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
                let mut temp1 = Integer::new();
                let mut temp2 = Integer::new();
                temp1.assign(fk * fk);
                temp2.assign(fk_1 * fk_1);
                let fib = Integer::from(temp1 + temp2);
                if index == len - 1 {
                    fib_n.assign(fib);
                    // let dur = start.elapsed();
                    // println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());
                } else {
                    memo.insert(i, fib);
                }
            }
        }

        // Purge unnecessary values
        // let start_purge = Instant::now();
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

        if n > cached && !purged_cache && i > &cached * 2 + 2 {
            for j in 0..=cached {
                if memo.contains_key(&j) {
                    memo.remove(&j);
                    // eprintln!("Removed j={}", j);
                }
            }
            purged_cache = true;
        }
        // let dur = start_purge.elapsed();
        // println!("Purged in {}.{}s", dur.as_secs(), dur.subsec_millis());
    });
    fib_n
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
    let cached: usize = 100;

    let sorted_indices = invert_order(n, cached);
    // eprintln!("sorted_indices={sorted_indices:#?}");

    let fib_n = fib(n, cached, &sorted_indices);

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
            fib_n % (Integer::from(10).pow(20))
        );
    }
}
