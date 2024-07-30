/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Very fast recursive calculation of an individual Fibonacci number using the
/// Fibonacci doubling identity. See also `demo/fib_doubling_iterative.rs` and
/// `demo/fib_doubling_iterative_purge.rs` for non-recursive variations.
///
/// I'm sure this is old hat, but I stumbled across an apparent pattern in the
/// Fibonacci sequence:
/// `For m > n: Fm = Fn-1.Fm-n + Fn.Fm-n+1.`
///
/// This has a special case when m = 2n or 2n+1, which not surprisingly turn out
/// to be well-known "doubling identities". The related technique is known as
/// "fast doubling".
///
/// For even indices: `F2n = Fn x (Fn-1 + Fn+1)`.
/// For odd indices: `F2n+1 = Fn^2 + Fn+1^2`.
///
/// This allows us to compute a given Fibonacci number F2n or F2n+1 by recursively
/// or indeed iteratively expressing it in terms of Fn-1, Fn and Fn+1, or any two
/// of these since Fn+1 = Fn-1 + Fn.
///
/// I suggested this to ChatGPT, as well as the idea of pre-computing and storing the
/// first 10 or 100 Fibonacci numbers to save repeated recalculation. ChatGPT went
/// one better by memoizing all computed numbers. As there is a great deal of repetition
/// and fanning out of calls to fib(), the memoization drastically cuts down recursion.
///
//# Purpose: Demo fast efficient Fibonacci with big numbers, limited recursion, and memoization, and a good job by ChatGPT.
use ibig::{ibig, IBig};
use std::collections::HashMap;
use std::time::Instant;

fn fib(n: usize, memo: &mut HashMap<usize, IBig>) -> IBig {
    if let Some(result) = memo.get(&n) {
        // eprintln!("Entered fib but found n={n}");
        return result.clone();
    }

    // eprintln!("Entered fib with new n={n}");
    let k = n / 4;
    let a = fib(k, memo);
    let b = fib(k + 1, memo);
    let a_2 = &a.pow(2);
    let a_3 = a_2 * &a;
    let a_4 = &a_3 * &a;
    let b_2 = &b.pow(2);

    // eprintln!("n={n}, k={k}, n % 4={}, a={a}, b={b}", n % 4);

    let result: IBig = match n % 4 {
        0 => {
            // e = -3a^4 + 8a^3.b - 6a^2.b^2 + 4a.b^3
            let b_3 = b_2 * &b;
            let e = -3 * &a_4 + 8 * &a_3 * &b - 6 * a_2 * b_2 + 4 * &a * &b_3;
            // eprintln!("e={e}");
            e
        }
        1 => {
            // f = 2a^4 - 4a^3.b + 6a^2.b^2 + b^4
            let b_4 = &b_2.pow(2);
            let f: IBig = 2 * &a_4 - 4 * &a_3 * &b + 6 * a_2 * b_2 + b_4;
            // eprintln!("f={f}");
            f
        }
        2 => {
            // g = e + f  = -a^4 + 4a^3.b + 4a.b^3  + b^4
            let b_3 = b_2 * &b;
            let b_4 = &b_3 * &b;
            let g: IBig = -1 * &a_4 + 4 * &a_3 * &b + 4 * &a * &b_3 + &b_4;
            // eprintln!("g={g}");
            g
        }
        3 => {
            // g = e + f  = -a^4 + 4a^3.b + 4a.b^3  + b^4
            let b_3 = b_2 * &b;
            let b_4 = &b_3 * &b;
            let h = &a_4 + 6 * a_2 * b_2 + 4 * &a * &b_3 + 2 * &b_4;
            // eprintln!("h={h}");
            h
        }
        4_usize.. => {
            panic!("Laws of maths now overturned, I'm panicking and I suggest you do too")
        }
    };
    // eprintln!("Made it here!");
    memo.insert(n, result.clone());
    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
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

    // Precompute and store base Fibonacci numbers
    let mut memo: HashMap<usize, IBig> = HashMap::new();
    memo.insert(0, ibig!(0));
    memo.insert(1, ibig!(1));
    memo.insert(2, ibig!(1));
    memo.insert(3, ibig!(2));
    memo.insert(4, ibig!(3));
    memo.insert(5, ibig!(5));
    memo.insert(6, ibig!(8));
    memo.insert(7, ibig!(13));
    memo.insert(8, ibig!(21));
    memo.insert(9, ibig!(34));
    memo.insert(10, ibig!(55));

    let fib_n = fib(n, &mut memo);

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
            fib_n % (ibig!(10).pow(20))
        );
    }
}
