/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast recursive calculation of an individual Fibonacci number using the "fast doubling""
/// technique found at
/// https://www.geeksforgeeks.org/fast-doubling-method-to-find-the-nth-fibonacci-number/.
/// Based on the two formulae: F(2n) = F(n)[2F(n+1) – F(n)] and F(2n + 1) = F(n)2+F(n+1)2
use ibig::{ubig, UBig};
use std::env;
use std::time::Instant;

fn fast_doubling(n: usize, results: &(UBig, UBig)) -> (UBig, UBig) {
    let (f_n, f_n_1) = &results; // (F(n), F(n+1))

    let f_2n = f_n * &(2 * f_n_1 - f_n);
    let f_2n_1 = f_n * f_n + f_n_1 * f_n_1; // F(2n + 1)

    // eprintln!("n={n}, f_n={f_n}, f_n_1={f_n_1}, f_2n={f_2n}, f_2n_1={f_2n_1}");

    if n % 2 == 0 {
        (f_2n, f_2n_1)
    } else {
        // temp as sum of references avoids cloning f_2n_1
        let temp = &f_2n + &f_2n_1;
        (f_2n_1, temp)
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
    let mut chain = Vec::<usize>::new();
    let mut temp_n = n;

    while temp_n > 0 {
        chain.push(temp_n);
        temp_n /= 2;
    }

    chain.sort();
    let mut results = (ubig!(0), ubig!(1));
    for i in chain.iter() {
        results = fast_doubling(*i, &results);
    }

    let (fib_n, _) = &results;

    let dur = start.elapsed();
    println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

    let fib_n_str = fib_n.to_string();

    if n <= 1000 {
        println!("F({n})={fib_n}");
    } else if n >= 1000000 {
        println!("F({n_disp}) ends in ...{}", fib_n % ubig!(1000000000));
    } else {
        let l = fib_n_str.len();
        println!(
            "F({}) = {}...{}",
            n_disp,
            &fib_n_str[0..20],
            &fib_n_str[l - 20..l - 1]
        );
    }
}
