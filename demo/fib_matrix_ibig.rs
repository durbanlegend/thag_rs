/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// Example of a matrix calculation of an individual Fibonacci number.
/// This example is by courtesy of Gemini AI. For F100,000 this is the
/// fastest individual calculation, 3-4 times faster than the doubling
/// method, and about 10 times faster than the classic iteration. These
/// are not formal benchmarks and your mileage may vary.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo an alternative to the standard computation for Fibonacci numbers.
use ibig::{ubig, UBig};
use std::env;

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
let n: usize = args[1].parse().expect(msg);

// for i in 0..=n {
//   println!("F{} = {}", i, fibonacci_matrix(i.try_into().unwrap()));
// }
println!("F{} = {}", n, fibonacci_matrix(n.try_into().unwrap()));
