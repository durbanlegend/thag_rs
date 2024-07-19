/*[toml]
[dependencies]
dashu = "0.4.2"
*/

/// Very fast recursive calculation of an individual Fibonacci number
/// using the matrix squaring technique.
/// This example is by courtesy of Gemini AI. For F100,000 this is the
/// fastest individual calculation, 3-4 times faster than the doubling
/// method, and about 10 times faster than the classic iteration. For
/// F1,000,000 to F10,000,000 it's overtaken by the doubling method.
/// These are are not formal benchmarks and your mileage may vary. Besides,
/// these are only demo scripts and come with no guarantees.
///
/// Aside from the imports, this script is interchangeable with `demo/fib_matrix_ibig.rs`
/// and performance on my setup was very similar. However, `dashu` is
/// not confined to integers but also supports floating point and rational
/// numbers.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo a very fast precise computation for large individual Fibonacci numbers.
use dashu::ubig;
use dashu::integer::UBig;
use std::env;
use std::time::Instant;

fn fibonacci_matrix(n: u128) -> UBig {
  if n <= 1 {
    return UBig::from(n);
  }

  let mut a = [[ubig!(1), ubig!(1)], [ubig!(1), ubig!(0)]];
  let mut result = [[ubig!(1), ubig!(0)], [ubig!(0), ubig!(1)]];

  // Efficient exponentiation using repeated squaring
  let mut power = n - 1;
  while power > 0 {
    if power & 1 == 1 {
      result = multiply_matrices(result.clone(), a.clone());
    }
    power >>= 1;
    a = multiply_matrices(a.clone(), a.clone());
  }

  return result[0][0].clone();
}

fn multiply_matrices(a: [[UBig; 2]; 2], b: [[UBig; 2]; 2]) -> [[UBig; 2]; 2] {
  let mut result: [[UBig; 2]; 2] = [[ubig!(0), ubig!(0)], [ubig!(0), ubig!(0)]];
  for i in 0..2 {
    for j in 0..2 {
      for k in 0..2 {
        result[i][j] += a[i][k].clone() * b[k][j].clone();
      }
    }
  }
  return result;
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>, where 0 <= n", args[0]);
    std::process::exit(1);
}

let msg = "Please provide a positive integer";
let n: u128 = args[1].parse().expect(msg);
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
let fib_n = fibonacci_matrix(n);

let dur = start.elapsed();
println!("Done! in {}.{}s", dur.as_secs(), dur.subsec_millis());

if n <= 1000 {
    println!("F({n})={fib_n}");
} else if n >= 1000000 {
    println!("F({n_disp}) ends in ...{}", fib_n % ubig!(1000000000));
} else {
    let fib_n_str = fib_n.to_string();
    let l = fib_n_str.len();
    println!(
        "F({n_disp}) len = {l}, value = {}...{}",
        &fib_n_str[0..20],
        &fib_n_str[l - 20..l - 1]
    );
}
