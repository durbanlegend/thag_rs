/// Very fast recursive calculation of an individual Fibonacci number
/// using the matrix squaring technique.
/// This example is by courtesy of Gemini AI. See big-number versions
/// `demo/fib_matrix_dashu.rs` and `demo/fib_matrix_ibig.rs`.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo an alternative to the standard computation for Fibonacci numbers.
//# Categories: educational, math, recreational, technique
use std::env;

fn fibonacci_matrix(n: u128) -> u128 {
  if n <= 1 {
    return n;
  }

  let mut a = [[1, 1], [1, 0]];
  let mut result = [[1, 0], [0, 1]];

  // Efficient exponentiation using repeated squaring
  let mut power = n - 1;
  while power > 0 {
    if power & 1 == 1 {
      result = multiply_matrices(result.clone(), a);
    }
    power >>= 1;
    a = multiply_matrices(a.clone(), a);
  }

  return result[0][0];
}

fn multiply_matrices(a: [[u128; 2]; 2], b: [[u128; 2]; 2]) -> [[u128; 2]; 2] {
  let mut result: [[u128; 2]; 2] = [[0; 2]; 2];
  for i in 0..2 {
    for j in 0..2 {
      for k in 0..2 {
        result[i][j] += a[i][k] * b[k][j];
      }
    }
  }
  return result;
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>, where 0 <= n <= 128", args[0]);
    std::process::exit(1);
}

let msg = "Please provide a valid integer between 0 and 128";
let n: usize = args[1].parse().expect(msg);
if n > 128 {
    println!("{msg}");
    std::process::exit(1);
}

// for i in 0..=n {
//   println!("F{} = {}", i, fibonacci_matrix(i.try_into().unwrap()));
// }
println!("F{} = {}", n, fibonacci_matrix(n.try_into().unwrap()));
