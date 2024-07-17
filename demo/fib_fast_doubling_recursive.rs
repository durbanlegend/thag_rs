/*[toml]
[dependencies]
ibig = "0.3.6"
*/

use ibig::{ubig, UBig};
use std::env;

fn fast_doubling(n: usize, res: &mut [UBig; 2]) {
    if n == 0 {
        res[0] = ubig!(0);
        res[1] = ubig!(1);
        return;
    }

    fast_doubling(n / 2, res);

    let a = &res[0];
    let b = &res[1];

    let mut c = 2 * b - a;
    if c < ubig!(0) {
        c = c + ubig!(1);
    }
    c = a * &c;

    let d = a * a + b * b;

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

    let mut res = [ubig!(0), ubig!(0)];
    fast_doubling(n, &mut res);
    // println!("F({}) = {}", n, res[0]);
}
