/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Same script as `demo/fib_basic_ibig.rs` with basic instrumentation added for benchmarking
/// against other fibonacci scripts.
/// Scripts can then be selected and run sequentially.
/// E.g. an apples-with-apples comparison of diferent algorithms implemented using the ``ibig` crate:
/// `ls -1 demo/fib*ibig*.rs | grep -v fib_basic_ibig.rs | while read f; do echo $f; rs_script -t $f -- 10000000; done`
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demonstrate instrumenting scripts for benchmarking.
use ibig::{ubig, UBig};
use std::env;
use std::iter::{successors, Successors, Take};
use std::time::Instant;

// Snippet accepts function or closure. This closure returns only the last value Fn.
fn fib_value_n(n: usize) -> UBig {
    successors(Some((ubig!(0), ubig!(1))), |(a, b)| {
        Some((b.clone(), (a + b).into()))
    })
    .map(|(a, _b)| a)
    .nth(n)
    .unwrap()
}

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

// println!("Fibonacci F({n}) is {}", fib_value_n(n));
let fib_n = fib_value_n(n);

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
        fib_n % (ubig!(10).pow(20))
    );
}
