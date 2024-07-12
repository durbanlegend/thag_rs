use std::env;

// Some simple CLI args requirements...
let n = if let Some(arg) = env::args().nth(1) {
    // usize::from(arg)
    if let Ok(n) = arg.parse::<usize>() {
        n
    } else {
        println!("Usage: {} <n>", env::args().nth(0).unwrap());
        std::process::exit(1);
    }
} else {
    println!("Usage: {} <n>", env::args().nth(0).unwrap());
    std::process::exit(1);
};

let sqrt_5 = f64::from(5.0).sqrt();
let phi = (f64::from(1.0) + sqrt_5) / 2.0_f64;
let psi = (f64::from(1.0) - sqrt_5) / 2.0_f64;
// println!("sqrt_5={}, phi={:?}, psi={:?}", sqrt_5, phi, psi);

for i in 0..=n {
    let f = i as f64;
    println!("F{i} = {}", ((phi.powf(f) - psi.powf(f)) / sqrt_5).round());
}
