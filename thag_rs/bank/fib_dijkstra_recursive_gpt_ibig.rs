/*[toml]
[dependencies]
ibig = "0.3.6"
*/

use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn add_tuples((a1, b1, c1): (UBig, UBig, UBig), (b2, c2): (UBig, UBig)) -> (UBig, UBig, UBig) {
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
            r = add_tuples(r, (p.1.clone(), p.2.clone()));
        }
        p = add_tuples(p.clone(), (p.1.clone(), p.2.clone()));
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
    let fib_n = dijkstra_fib(n - 1);

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
}
