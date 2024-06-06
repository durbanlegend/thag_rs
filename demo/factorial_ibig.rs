/*[toml]
[dependencies]
ibig = "0.3.6"
*/

use ibig::{ubig, UBig};
use std::io::Read;
use std::iter::successors;

// Closures could just as well be methods
let fac1 = |n: usize| -> UBig {
    if n == 0 {
        ubig!(0)
    } else {
        (1..=n).fold(ubig!(1), |acc: UBig, i: usize| acc * UBig::from(i))
    }
};

// Can't substitute this in initial values (which hardly matter anyway)
// without getting further down a deferencing rabbit hole and ending up cloning.
let ubig_1 = ubig!(1);

// Using successors is possible, but turns out pretty inscrutable
let fac2 = |n: usize| -> UBig {
    successors(Some((ubig!(1), ubig!(1))), |(a, b)| {
        Some(((*&a + &ubig_1), (*&a + &ubig_1) * b))
    })
        .take(n)
        .last()
        .unwrap()
        .1
};

println!("Enter a positive integer to calculate its factorial");
println!(
    "Type lines of text at the prompt and hit Ctrl-{} on a new line when done",
    if cfg!(windows) { 'Z' } else { 'D' }
);

let mut buffer = String::new();
io::stdin().lock().read_to_string(&mut buffer)?;

let n: usize = buffer
    .trim_end()
    .parse()
    .expect("Can't parse input into a positive integer");

let fac1_n = fac1(n);

assert_eq!(fac1_n, fac2(n));
println!("factorial({n}) = {:#?}", fac1_n);
