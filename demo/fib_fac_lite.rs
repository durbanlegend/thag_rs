/*[toml]
[dependencies]
itertools = "0.13.0"
*/
use itertools::iterate;

#[allow(unused_doc_comments)]
/// Windows-friendly lite version of fib_fac.rs. It lists the first 26 fibonacci numbers
/// (0..25), followed by the first 26 factorials.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
/// Limited to Rust u128 types, thus it supports a maximum of fib(92) and fac(34).
///
/// The `fib` and `fac` closures could equally be implemented as functions here.
//# Purpose: Demonstrate snippets and fast non-recursive fibonacci and factorial algorithms.

let fib = |n: usize| -> usize {
    match n {
        0 => 0_usize,
        1 => 1_usize,
        _ => {
            iterate((0, 1), |&(a, b)| (b, a + b))
                .take(n)
                .last()
                .unwrap()
                .1
        }
    }
};

let limit = 25_usize;
(0..=limit).for_each(|n| {
    println!("fibonacci({n})={}", fib(n));
});

println!();

// let fac = |n: u128| -> u128 {
//     if n == 0 {
//         0
//     } else {
//         (1..=n).product()
//     }
// };

// let limit = limit as u128;

// for n in 0..=limit {
//     println!("factorial({n})={}", fac(n));
// }
