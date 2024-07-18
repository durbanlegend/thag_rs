/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// When implemented using ibig, this loses to `demo/fib_doubling_iterative*.rs`.
/// `rug` version is by far the fastest, but `rug` is based on GNU C libs GMP, MPFR and MPC.
/// https://users.rust-lang.org/t/optimizing-fast-fibonacci-computation/56933/23
use ibig::{ubig, UBig};
use std::collections::HashMap;
use std::env;
use std::time::Instant;

pub fn fast_fibonacci(target_n: usize) -> UBig {
    let cache: HashMap<usize, UBig> = HashMap::new();
    let (result, _) = fib_dijk(target_n, cache);
    return result;
}

fn is_even(n: usize) -> bool {
    return n & 1 == 0;
}

fn fib_dijk_helper(
    target_n: usize,
    cache: HashMap<usize, UBig>,
) -> (UBig, HashMap<usize, UBig>) {
    if target_n <= 1 {
        return (UBig::from(target_n), cache);
    } else {
        let half_n = target_n >> 1;
        let (x, cache) = fib_dijk(half_n, cache);
        let x_2 = UBig::from((&x).pow(2));
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

fn fib_dijk(target_n: usize, cache: HashMap<usize, UBig>) -> (UBig, HashMap<usize, UBig>) {
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
let n_disp = n.to_string()
    .as_bytes()
    .rchunks(3)
    .rev()
    .map(std::str::from_utf8)
    .collect::<Result<Vec<&str>, _>>()
    .unwrap()
    .join(",");

let start = Instant::now();
let fib_n = fast_fibonacci(n);

let dur = start.elapsed();
println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

let fib_n_str = fib_n.to_string();

if n <= 1000 {
    println!("F({n})={fib_n}");
} else if n >= 1000000 {
    println!("F({n_disp}) ends in ...{}", fib_n % ubig!(1000000000));
} else {
    let l = fib_n_str.len();
    println!(
        "F({}) = {}...{}",
        n_disp,
        &fib_n_str[0..20],
        &fib_n_str[l - 20..l - 1]
    );
}
