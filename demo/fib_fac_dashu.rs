/*[toml]
[dependencies]
itertools = "0.13.0"
dashu = "0.4.2"
*/
use dashu::integer::IBig;
use itertools::iterate;
use std::iter::successors;

//// Windows-friendly version of fib_fac.rs. It lists the first 51 fibonacci numbers
/// (0..50), followed by the first 51 factorials.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
/// The `fib` and `fac` closures could equally be implemented as functions here.
//# Purpose: Demonstrate big integers and fast non-recursive fibonacci and factorial algorithms.

fn main() {
    let fac = |n: usize| -> IBig {
        if n == 0 {
            IBig::from(0_usize)
        } else {
            (1..=n).map(IBig::from).product()
        }
    };

    // This works!
    let fib1 = |n: usize| -> usize {
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

    // This works too!
    let fib2 = |n: usize| -> usize {
        match n {
            0 => 0_usize,
            1 => 1_usize,
            _ => {
                successors(Some((0, 1)), |&(a, b)| Some((b, a + b)))
                    .take(n)
                    .last()
                    .unwrap()
                    .1
            }
        }
    };

    let limit = 49_usize;
    (0..=limit).for_each(|n| {
        assert_eq!(fibonacci(n), fib1(n));
        assert_eq!(fib1(n), fib2(n));
        println!("fibonacci({n})={}", fibonacci(n));
    });

    println!();

    let mut prev = IBig::from(1);

    for n in 0..=limit {
        let factorial_n = factorial(n);
        let fac_n = fac(n);
        println!("fac({n})={fac_n}");
        assert_eq!(fac_n, factorial_n);
        if n > 0 {
            assert_eq!(fac_n, prev * n);
            prev = fac_n;
        }
    }
}

fn factorial(n: usize) -> IBig {
    match n {
        0 | 1 => IBig::from(n),
        // _ => n * factorial(n - 1),
        _ => {
            let mut b = IBig::from(1_usize);
            for a in 2..=n {
                b *= a;
            }
            b
        }
    }
}

fn fibonacci(n: usize) -> usize {
    match n {
        0 => 0_usize,
        1 => 1_usize,
        _ => {
            let (mut a, mut b) = (0, 1);
            for _ in 1..n {
                (a, b) = (b, a + b);
            }
            b
        }
    }
}
