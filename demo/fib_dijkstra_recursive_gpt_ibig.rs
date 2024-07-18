/*[toml]
[dependencies]
ibig = "0.3.6"
*/

use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn add_tuples(
    (a1, b1, c1): (UBig, UBig, UBig),
    (a2, b2, c2): (UBig, UBig, UBig),
) -> (UBig, UBig, UBig) {
    let v = &a1 * &b2 + &b1 * &c2;
    let w = &b1 * &b2 + &c1 * &c2;
    (w.clone() - v.clone(), v, w)
}

fn dijkstra_fib(n: usize) -> UBig {
    let mut r = (ubig!(1), ubig!(1), ubig!(2));
    let mut p = (ubig!(0), ubig!(1), ubig!(1));

    let mut k = n;

    while k > 0 {
        if k % 2 == 1 {
            r = add_tuples(r, p.clone());
        }
        p = add_tuples(p.clone(), p);
        k /= 2;
    }

    r.0
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().expect("Please provide a valid number");

    let start = Instant::now();
    let fib_n = dijkstra_fib(n - 1);

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
