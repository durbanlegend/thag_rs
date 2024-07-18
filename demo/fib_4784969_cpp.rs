/*[toml]
[dependencies]
ibig = "0.3.6"
*/

use ibig::{ubig, UBig};
use std::collections::HashMap;
use std::env;
use std::time::Instant;

fn zero() -> UBig {
    ubig!(0)
}

fn one() -> UBig {
    ubig!(1)
}

fn two() -> UBig {
    ubig!(2)
}

fn is_even(n: usize) -> bool {
    n % 2 == 0
}

fn fibo_ej_olson(n: usize, a: &mut UBig, b: &mut UBig) {
    if n == 0 {
        *a = zero();
        *b = one();
        return;
    }
    let mut ta = zero();
    fibo_ej_olson(n / 2, a, b);
    ta = a.clone();
    if is_even(n) {
        *a = &ta * (&(b.clone() * 2) - &ta);
        *b = &ta.pow(2) + &(b.pow(2));
    } else {
        *a = a.pow(2) + &(b.pow(2));
        *b = &*b * (&ta + &ta + b.clone());
    }
}

fn fibo_new_work(n: usize, a: &mut UBig, b: &mut UBig) {
    if n == 0 {
        *a = zero();
        *b = one();
        return;
    }
    fibo_new_work(n / 2, a, b);
    if n % 2 == 0 {
        let t = two() * b.clone() - a.clone();
        *a = &*a * &t;
        *b = &*b * &t;
        if n % 4 == 0 {
            *b = &*b - &one();
        } else {
            *b = &*b + &one();
        }
    } else {
        let t = two() * a.clone() + b.clone();
        *b = &*b * &t;
        *a = &*a * &t;
        if n % 4 == 1 {
            *a = &*a + &one();
        } else {
            *a = &*a - &one();
        }
    }
}

fn fibo_new(n: usize, b: &mut UBig) {
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
        *b = &*b * (&a + &a + b.clone());
    } else {
        let t = &*b * (&(b.clone() * 2) - &a);
        if n % 4 == 1 {
            *b = &t - &one();
        } else {
            *b = &t + &one();
        }
    }
}

fn fibo_init() -> HashMap<usize, UBig> {
    let mut memo = HashMap::new();
    memo.insert(0, zero());
    memo.insert(1, one());
    memo.insert(2, one());
    memo
}

fn fibo(n: usize, memo: &mut HashMap<usize, UBig>) -> UBig {
    if let Some(res) = memo.get(&n) {
        return res.clone();
    }

    let k = n / 2;
    let a = fibo(k, memo);
    let b = fibo(k - 1, memo);
    let res = if is_even(n) {
        &a * (&two() * &b + &a)
    } else {
        let twoa = &two() * &a;
        if n % 4 == 1 {
            (&twoa + &b) * (&twoa - &b) + &two()
        } else {
            (&twoa + &b) * (&twoa - &b) - &two()
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

    let start = Instant::now();

    let mut memo = fibo_init();
    let fib_n = fibo(n, &mut memo);

    let dur = start.elapsed();
    println!("fibo Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else {
        let fib_n = fib_n.to_string();
        let l = fib_n.len();
        println!("F({}) = {}...{}", n, &fib_n[0..20], &fib_n[l - 20..l - 1]);
    }

    let start = Instant::now();

    let (mut a, mut b) = (ubig!(0), ubig!(1));
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
    } else {
        let fib_n = fib_n.to_string();
        let l = fib_n.len();
        println!("F({}) = {}...{}", n, &fib_n[0..20], &fib_n[l - 20..l - 1]);
    }

    let start = Instant::now();

    let (mut a, mut b) = (ubig!(0), ubig!(1));
    fibo_new_work(n, &mut a, &mut b);
    let fib_n = a;

    let start = Instant::now();

    let mut b = ubig!(1);
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
    } else {
        let fib_n = fib_n.to_string();
        let l = fib_n.len();
        println!("F({}) = {}...{}", n, &fib_n[0..20], &fib_n[l - 20..l - 1]);
    }
}
