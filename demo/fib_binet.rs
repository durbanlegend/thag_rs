/*[toml]
[dependencies]
num-traits = "0.2.19"
*/

use num_traits::Float;
use std::env;

fn fib_closed<T: Float>(n: T) -> T {
    let sqrt_5 = T::from(5.0).unwrap().sqrt();
    let phi = (T::from(1.0).unwrap() + sqrt_5) / T::from(2.0).unwrap();
    let psi = (T::from(1.0).unwrap() - sqrt_5) / T::from(2.0).unwrap();

    (phi.powf(n) - psi.powf(n)) / sqrt_5
}

// Some simple CLI args requirements...
let n = if let Some(arg) = env::args().nth(1) {
    // usize::from(arg)
    if let Ok(n) = arg.parse::<usize>() {
        n
    } else {
        println!("Usage: {} <n>", env::args().nth(0).unwrap());
        return Ok(());
    }
}
else {
    println!("Usage: {} <n>", env::args().nth(0).unwrap());
    return Ok(());
};

println!("F{n} = {}", fib_closed(n as f64) as i64);
// fn main() {
//     let seq: Vec<f64> = (0..30).map(|i| fib_closed(i as f64)).collect();
//     for fib in seq {
//         println!("{}", fib as i64);
//     }
// }
