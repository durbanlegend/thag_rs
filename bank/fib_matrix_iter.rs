/// This turned out to be the basic iteration in disguise, so a bit lame. >>>
/// Example of a matrix calculation of any given number in the Fibonacci sequence
/// using a matrix calculation. This example is by courtesy of Gemini AI.
///
/// See https://en.wikipedia.org/wiki/Fibonacci_sequence.
/// F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.
///
//# Purpose: Demo an alternative to iterative computation for Fibonacci numbers.
use std::env;

fn fibonacci_iterative_matrix(n: usize) -> Vec<u128> {
    if n == 0 {
        return vec![];
    }

    let mut result: Vec<u128> = vec![0; n + 1];
    result[0] = 0;
    result[1] = 1;

    // Loop from index 2 to n (inclusive)
    for i in 2..=n {
        // Access previous two elements using indexing
        let prev1 = result[i - 1];
        let prev2 = result[i - 2];

        // Update current element using matrix multiplication principle
        result[i] = prev1 + prev2;
    }

    // Return the calculated Fibonacci series
    result
}

fn main() {
    // let n = 10;

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>, where 0 <= n <= 128", args[0]);
        std::process::exit(1);
    }

    let msg = "Please provide a valid integer between 0 and 186";
    let n: usize = args[1].parse().expect(msg);
    if n > 186 {
        println!("{msg}");
        std::process::exit(1);
    }

    let fibonacci_series = fibonacci_iterative_matrix(n);

    println!("Fibonacci Series (0 to {} ):", n);
    for (i, value) in fibonacci_series.iter().enumerate() {
        println!("F({}): {}", i, value);
    }
}
