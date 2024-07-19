/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast recursive calculation of an individual Fibonacci number
/// using the fast doubling technique.
use ibig::{ubig, UBig};

/// Recursive function for calculating Fibonacci numbers using the fast doubling method.
fn fast_doubling(n: usize, res: &mut [UBig; 2]) {
    if n == 0 {
        res[0] = ubig!(0);
        res[1] = ubig!(1);
        return;
    }

    let a = &res[0];
    let b = &res[1];

    let mut c = ubig!(2) * b - a;
    if c < ubig!(0) {
        c += ubig!(1);
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
    let args: Vec<String> = std::env::args().collect();
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

    let start = std::time::Instant::now();
    let mut res = [ubig!(0), ubig!(1)];
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

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else if n > 1000000000 {
        println!("F({n}) ends in ...{}", fib_n % ubig!(1000000000));
    } else {
        let fib_n_str = fib_n.to_string();
        let l = fib_n_str.len();
        println!(
            "F({n_disp}) len = {l}, value = {}...{}",
            &fib_n_str[0..20],
            &fib_n_str[l - 20..l - 1]
        );
    }
}
