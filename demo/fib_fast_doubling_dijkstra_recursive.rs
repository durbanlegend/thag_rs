/*[toml]
[dependencies]
rug = "1.24.1"
*/

/// Speed demon - but uses rug
/// https://users.rust-lang.org/t/optimizing-fast-fibonacci-computation/56933/23
use rug::ops::Pow;
use rug::Integer;
use std::collections::HashMap;
use std::env;
use std::time::Instant;

pub fn fast_fibonacci(target_n: usize) -> Integer {
    let cache: HashMap<usize, Integer> = HashMap::new();
    let (result, _) = fib_dijk(target_n, cache);
    return result;
}

fn is_even(n: usize) -> bool {
    return n & 1 == 0;
}

fn fib_dijk_helper(
    target_n: usize,
    cache: HashMap<usize, Integer>,
) -> (Integer, HashMap<usize, Integer>) {
    if target_n <= 1 {
        return (Integer::from(target_n), cache);
    } else {
        let half_n = target_n >> 1;
        let (x, cache) = fib_dijk(half_n, cache);
        let x_2 = Integer::from((&x).pow(2));
        if is_even(target_n) {
            let (y, cache) = fib_dijk(half_n - 1, cache);
            let result = 2 * x * y + x_2;
            return (result, cache);
        } else {
            let (z, cache) = fib_dijk(half_n + 1, cache);
            return (x_2 + z.pow(2), cache);
        }
    }
}

fn fib_dijk(target_n: usize, cache: HashMap<usize, Integer>) -> (Integer, HashMap<usize, Integer>) {
    if cache.contains_key(&target_n) {
        return (cache.get(&target_n).unwrap().clone(), cache);
    } else {
        let (result, mut cache) = fib_dijk_helper(target_n, cache);
        cache.insert(target_n, result.clone());
        return (result, cache);
    }
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}

let n: usize = args[1].parse().expect("Please provide a valid number");

let start = Instant::now();
let fib_n = fast_fibonacci(n);

let dur = start.elapsed();
println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

let fib_n = fib_n.to_string();
let l = fib_n.len();
println!("F({}) = {}...{}", n, &fib_n[0..20], &fib_n[l-20..l-1]);
