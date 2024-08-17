/*[toml]
[dependencies]
rug = "1.24.1"
*/

/// Very fast recursive calculation of an individual Fibonacci number
/// using the matrix squaring technique.
///
/// Won't work with default Windows 11 because of the `rug` crate, which is a pity because
/// `rug` is a beast due to its access to powerful GNU libraries.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo a very fast precise computation for large individual Fibonacci numbers.

use rug::ops::Pow;
use rug::Integer;
use std::env;
use std::time::Instant;

fn fibonacci_matrix(n: u128) -> Integer {
  if n <= 1 {
    return Integer::from(n);
  }

  let mut a = [[Integer::from(1), Integer::from(1)], [Integer::from(1), Integer::from(0)]];
  let mut result = [[Integer::from(1), Integer::from(0)], [Integer::from(0), Integer::from(1)]];

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

fn multiply_matrices(a: [[Integer; 2]; 2], b: [[Integer; 2]; 2]) -> [[Integer; 2]; 2] {
  let mut result: [[Integer; 2]; 2] = [[Integer::from(0), Integer::from(0)], [Integer::from(0), Integer::from(0)]];
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
