/*[toml]
[dependencies]
itertools = "0.13.0"
rug = "1.24.1"
*/
use itertools::iterate;
use rug::Integer;
use std::iter::successors;

fn main() {
    let fac = |n: usize| -> Integer {
        if n == 0 {
            Integer::from(0_usize)
        } else {
            (1..=n).map(Integer::from).product()
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
                    .0
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
                    .0
            }
        }
    };

    let limit = 50_usize;
    (0..=limit).for_each(|n| {
        println!("fibonacci({n})={}", fibonacci(n));
        println!("fib1({n})={}", fib1(n));
        println!("fib2({n})={}", fib2(n));
    });

    let mut prev = Integer::from(1);

    for n in 0..=limit {
        let factorial_n = factorial(n);
        // println!(\"factorial({n})={}\", factorial_n);
        let fac_n = fac(n);
        println!("fac({n})={fac_n}");
        assert_eq!(fac_n, factorial_n);
        if n > 0 {
            assert_eq!(fac_n, prev * n);
            prev = fac_n;
        }
    }
}

fn factorial(n: usize) -> Integer {
    match n {
        0 | 1 => Integer::from(n),
        // _ => n * factorial(n - 1),
        _ => {
            let mut b = Integer::from(1_usize);
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
            for _ in 1..n - 1 {
                (a, b) = (b, a + b);
            }
            b
        }
    }
}
