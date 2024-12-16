/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// https://kukuruku.co/post/the-nth-fibonacci-number-in-olog-n/
/// Per Coding Skull
use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn add(a: (UBig, UBig, UBig), b: (UBig, UBig, UBig)) -> (UBig, UBig, UBig) {
    let (a1, b1, c1) = a;
    let (_x, y, z) = b;
    let v = a1 * y.clone() + b1.clone() * z.clone();
    let w = b1 * y + c1 * z;
    (w.clone() - v.clone(), v, w)
}

fn fib2(mut n: u128) -> UBig {
    let mut r = (ubig!(1), ubig!(1), ubig!(2));
    let mut p = (ubig!(0), ubig!(1), ubig!(1));
    while n != 0 {
        if n % 2 != 0 {
            r = add(r, p.clone());
        }
        p = add(p.clone(), p);
        n >>= 1;
    }
    r.0
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: u128 = args[1].parse().expect("Please provide a valid number");

    let start = Instant::now();
    let fib_n = fib2(n - 1);

    let dur = start.elapsed();
    println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else {
        let fib_n = fib_n.to_string();
        let l = fib_n.len();
        println!("F({}) = {}...{}", n, &fib_n[0..20], &fib_n[l - 20..l - 1]);
    }
}
