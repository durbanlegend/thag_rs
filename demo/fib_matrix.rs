use std::env;

fn fibonacci_matrix(n: u32) -> u32 {
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

fn multiply_matrices(a: [[u32; 2]; 2], b: [[u32; 2]; 2]) -> [[u32; 2]; 2] {
  let mut result: [[u32; 2]; 2] = [[0; 2]; 2];
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
    eprintln!("Usage: {} <n>", args[0]);
    std::process::exit(1);
}
let n: usize = args[1].parse().expect("Please provide a valid number");

for i in 0..=n {
  println!("F{} = {}", i, fibonacci_matrix(i.try_into().unwrap()));
}
