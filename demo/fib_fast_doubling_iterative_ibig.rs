/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast recursive calculation of an individual Fibonacci number
/// using the fast doubling technique.
/// https://www.geeksforgeeks.org/fast-doubling-method-to-find-the-nth-fibonacci-number/
use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn fast_doubling(n: usize, res: &mut [UBig; 2]) {
    if n == 0 {
        res[0] = ubig!(0);
        res[1] = ubig!(1);
        return;
    }

    let a = &res[0];
    let b = &res[1];

    let mut c = 2 * b - a;
    if c < ubig!(0) {
        c = c + ubig!(1);
    }
    c = a * &c;

    let d = a * a + b * b;

    // eprintln!("n={n}, a={a}, b={b}, c={c}, d={d}");

    if n % 2 == 0 {
        res[0] = c;
        res[1] = d;
    } else {
        res[0] = d.clone();
        res[1] = c + d;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>, where 0 <= n", args[0]);
        std::process::exit(1);
    }

    let msg = "Please provide a positive integer";
    let mut n: usize = args[1].parse().expect(msg);

    let start = Instant::now();
    let mut res = [ubig!(0), ubig!(1)];
    let mut chain = Vec::<usize>::new();
    while n > 0 {
        chain.push(n);
        n = n / 2;
    }

    chain.sort();
    for i in chain.iter() {
        fast_doubling(*i, &mut res);
    }

    // println!("F({}) = {}", n, res[0]);

    let dur = start.elapsed();
    println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    let fib_n = res[0].to_string();

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else {
        let fib_n = fib_n.to_string();
        let l = fib_n.len();
        println!("F({}) = {}...{}", n, &fib_n[0..20], &fib_n[l - 20..l - 1]);
    }
}
