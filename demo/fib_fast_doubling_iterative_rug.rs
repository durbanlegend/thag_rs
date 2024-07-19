/*[toml]
[dependencies]
rug = "1.24.1"
*/

use rug::ops::Pow;
/// Very fast recursive calculation of an individual Fibonacci number
/// using the fast doubling technique.
/// https://www.geeksforgeeks.org/fast-doubling-method-to-find-the-nth-fibonacci-number/
use rug::{Complete, Integer};
use std::env;
use std::time::Instant;

fn fast_doubling(n: usize, res: &mut [Integer; 2]) {
    if n == 0 {
        res[0] = Integer::from(0);
        res[1] = Integer::from(1);
        return;
    }

    let a = &res[0];
    let b = &res[1];

    let mut c = Integer::from(2) * b - a;
    if c < Integer::from(0) {
        c += Integer::from(1);
    }
    c = (a * &c).into();

    let d = a.pow(2).complete() + b.pow(2).complete();

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
    let n: usize = args[1].parse().expect(msg);
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
    let mut res = [Integer::from(0), Integer::from(1)];
    let mut chain = Vec::<usize>::new();
    let mut temp_n = n;

    while temp_n > 0 {
        chain.push(temp_n);
        temp_n /= 2;
    }

    chain.sort();
    for i in chain.iter() {
        fast_doubling(*i, &mut res);
    }

    let dur = start.elapsed();
    println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    let fib_n = &res[0];

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
