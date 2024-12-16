/*[toml]
[dependencies]
ibig = "0.3.6"
*/

/// https://github.com/tczajka/bigint-benchmark-rs (MIT licence)

// Using matrix exponentiation: [[1,0],[1,1]]^n = [[fib(n-1), fib(n)], [(fib(n), fib(n+1)]]
//
// If follows that:
// fib(2n) = fib(n) * (2 * fib(n+1) - fib(n))
// fib(2n+1) = fib(n)^2 + fib(n+1)^2
// fib(2n+2) = fib(2n) + fib(2n+1)

use ibig::{ubig, UBig};
use std::{
    env,
    fmt::Display,
    ops::{Add, Div, Mul, Sub}, time::Instant,
};

pub(crate) trait Number
where
    Self: Sized,
    Self: From<u32>,
    Self: Display,
    Self: Add<Self, Output = Self>,
    Self: for<'a> Add<&'a Self, Output = Self>,
    Self: Sub<Self, Output = Self>,
    Self: for<'a> Sub<&'a Self, Output = Self>,
    Self: Mul<Self, Output = Self>,
    Self: for<'a> Mul<&'a Self, Output = Self>,
    Self: Div<Self, Output = Self>,
    Self: for<'a> Div<&'a Self, Output = Self>,
{
    fn pow(&self, exp: u32) -> Self;
    fn to_hex(&self) -> String;
    fn mul_ref(&self, rhs: &Self) -> Self;
}

impl Number for ibig::UBig {
    fn pow(&self, exp: u32) -> Self {
        self.pow(exp as usize)
    }

    fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    fn mul_ref(&self, rhs: &Self) -> Self {
        self * rhs
    }
}

/// Fibonacci(n) in decimal
pub(crate) fn calculate_decimal<T: Number>(n: u32) -> String {
    calculate::<T>(n).to_string()
}

/// Fibonacci(n) in hexadecimal
pub(crate) fn calculate_hex<T: Number>(n: u32) -> String {
    calculate::<T>(n).to_hex()
}

fn calculate<T: Number>(n: u32) -> T {
    let (a, b) = fib::<T>(n / 2);
    if n % 2 == 0 {
        (T::from(2) * b - &a) * a
    } else {
        a.mul_ref(&a) + b.mul_ref(&b)
    }
}

// (fib(n), fib(n+1))
fn fib<T: Number>(n: u32) -> (T, T) {
    if n == 0 {
        (T::from(0), T::from(1))
    } else {
        let (a, b) = fib::<T>(n / 2);
        let new_b = a.mul_ref(&a) + b.mul_ref(&b);
        let new_a = (T::from(2) * b - &a) * a;
        if n % 2 == 0 {
            (new_a, new_b)
        } else {
            let new_c = new_a + &new_b;
            (new_b, new_c)
        }
    }
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <n>, where 0 <= n", args[0]);
    std::process::exit(1);
}

let msg = "Please provide a positive integer";
let n: u32 = args[1].parse().expect(msg);
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
let fib_n: UBig = calculate::<UBig>(n);

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
        fib_n % (ubig!(10).pow(20))
    );
}
