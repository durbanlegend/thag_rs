/*[toml]
[dependencies]
rug = "1.24.1"
*/

/// Rust of C++ example from `https://github.com/ZiCog/fibo_4784969` - so named because
/// F(4784969) is the first number in the Fibonacci sequence that has one million decimal
/// digits. This contains 3 alternative algorithms to compare their speed, the clear
/// winner being `fibo_new`.
///
/// E.g.:   `thag_rs demo/fib_4784969_cpp_ibig.rs -- 4784969   // or any positive integer`
///
//# Purpose: Demo 3 very fast Fibonacci algorithms (2.6 to 6 sec for me). Indications are that
//# the `rug` crate would be at least 5 times as fast.
use rug::ops::Pow;
use rug::{Complete, Integer};
use std::collections::HashMap;
use std::env;
use std::time::Instant;

fn zero() -> Integer {
    Integer::from(0)
}

fn one() -> Integer {
    Integer::from(1)
}

fn two() -> Integer {
    Integer::from(2)
}

fn is_even(n: usize) -> bool {
    n % 2 == 0
}

fn fibo_ej_olson(n: usize, a: &mut Integer, b: &mut Integer) {
    if n == 0 {
        *a = zero();
        *b = one();
        return;
    }
    fibo_ej_olson(n / 2, a, b);
    let ta = a.clone();
    if is_even(n) {
        *a = &ta * (&(b.clone() * 2).complete() - &ta);
        *b = (&ta.pow(2) + &(b.pow(2))).into();
    } else {
        *a = a.pow(2) + &(b.pow(2));
        *b = &*b * ((&ta + &ta).complete() + b.clone());
    }
}

fn fibo_new_work(n: usize, a: &mut Integer, b: &mut Integer) {
    if n == 0 {
        *a = zero();
        *b = one();
        return;
    }
    fibo_new_work(n / 2, a, b);
    if n % 2 == 0 {
        let t = two() * &*b - &*a;
        *a = (&*a * &t).into();
        *b = (&*b * &t).into();
        if n % 4 == 0 {
            *b = (&*b - &one()).into();
        } else {
            *b = (&*b + &one()).into();
        }
    } else {
        let t = two() * &*a + &*b;
        *b = (&*b * &t).into();
        *a = (&*a * &t).into();
        if n % 4 == 1 {
            *a = (&*a + &one()).into();
        } else {
            *a = (&*a - &one()).into();
        }
    }
}

fn fibo_new(n: usize, b: &mut Integer) {
    if n == 0 {
        *b = zero();
        return;
    }
    if n == 1 {
        *b = one();
        return;
    }
    let mut a = zero();
    fibo_new_work((n - 1) / 2, &mut a, b);
    if n % 2 == 0 {
        let two_a = &a * 2;
        let bee: &Integer = &*b;
        let two_a_plus_b = (two_a + bee).complete();
        *b = bee * two_a_plus_b;
    } else {
        let two_b = *b * 2;
        let t: Integer = &*b * two_b - &a;
        if n % 4 == 1 {
            *b = (&t - &one()).into();
        } else {
            *b = (&t + &one()).into();
        }
    }
}

fn fibo_init() -> HashMap<usize, Integer> {
    let mut memo = HashMap::new();
    memo.insert(0, zero());
    memo.insert(1, one());
    memo.insert(2, one());
    memo
}

fn fibo(n: usize, memo: &mut HashMap<usize, Integer>) -> Integer {
    if let Some(res) = memo.get(&n) {
        return res.clone();
    }

    let k = n / 2;
    let a = fibo(k, memo);
    let b = fibo(k - 1, memo);
    let res: Integer = if is_even(n) {
        &a * (&two() * (&b + &a).complete())
    } else {
        let twoa = (&two() * &a).complete();
        if n % 4 == 1 {
            (&twoa + &b).complete() * (&twoa - &b).complete() + &two()
        } else {
            (&twoa + &b).complete() * (&twoa - &b).complete() - &two()
        }
    };
    memo.insert(n, res.clone());
    res
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

    let mut memo = fibo_init();
    let fib_n = fibo(n, &mut memo);

    let dur = start.elapsed();
    println!("fibo Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else if n > 1000000000 {
        println!("F({n}) ends in {}", fib_n / Integer::from(1000000000));
    } else {
        let fib_n_str = fib_n.to_string();
        let l = fib_n_str.len();
        println!(
            "F({n_disp}) len = {l}, value = {} ... {}",
            &fib_n_str[0..20],
            &fib_n_str[l - 20..l]
        );
    }

    let start = Instant::now();

    let (mut a, mut b) = (Integer::from(0), Integer::from(1));
    fibo_ej_olson(n, &mut a, &mut b);
    let fib_n = a;

    let dur = start.elapsed();
    println!(
        "fibo_ej_olson Done! in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else if n > 1000000000 {
        println!("F({n}) ends in {}", fib_n / Integer::from(1000000000));
    } else {
        let fib_n_str = fib_n.to_string();
        let l = fib_n_str.len();
        println!(
            "F({n_disp}) len = {l}, value = {} ... {}",
            &fib_n_str[0..20],
            &fib_n_str[l - 20..l]
        );
    }

    let start = Instant::now();

    let mut b = Integer::from(1);
    fibo_new(n, &mut b);
    let fib_n = b;

    let dur = start.elapsed();
    println!(
        "fibo_new Done! in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else if n > 1000000000 {
        println!("F({n}) ends in {}", fib_n / Integer::from(1000000000));
    } else {
        let fib_n_str = fib_n.to_string();
        let l = fib_n_str.len();
        println!(
            "F({n_disp}) len = {l}, value = {} ... {}",
            &fib_n_str[0..20],
            &fib_n_str[l - 20..l]
        );
    }
}
