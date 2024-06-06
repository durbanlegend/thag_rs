/*[toml]
[dependencies]
ibig = "0.3.6"
*/

use ibig::{ubig, UBig};
use std::error::Error;
use std::io;
use std::io::Read;

/// Fast factorial algorithm avoiding inefficient recursion.
fn main() -> Result<(), Box<dyn Error>> {
    let fac = |n: usize| -> UBig {
        if n == 0 {
            ubig!(0)
        } else {
            (1..=n).fold(ubig!(1), |acc: UBig, i: usize| acc * UBig::from(i))
        }
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

    println!("fac({n}) = {:#?}", fac(n));
    Ok(())
}
