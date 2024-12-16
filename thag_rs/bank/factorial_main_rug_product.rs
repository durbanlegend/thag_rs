/*[toml]

   [package]
   name = "factorial_main"
   version = "0.0.1"
   edition = "2021"

   [dependencies]
   rug = { version = "1.24.0", features = ["integer"] }
   # TODO out: the following 3 dependencies are just to test retrieving multiple
   #           dependencies from code and are not needed by this script at all.
   serde = { version = "1.0", features = ["derive"] }
   env_logger = "0.11.3"
   log = "0.4.22"

   [workspace]

   [[bin]]
   name = "factorial_main"

*/

use rug::Integer;
use std::env;
use std::error::Error;
// use std::io;
// use std::io::Read;

/// Fast factorial algorithm avoiding recursion.
/// Unfortunately Windows 11 won't currently run this natively.
/// Supposedly you can run it by installing MSYS2, but I haven't tested this.
/// On Linux you may need to install the m4 package.
//# Purpose: Demo fast factorial using `rug` crate and `std::iter::Product` trait.
fn main() -> Result<(), Box<dyn Error>> {
    let fac = |n: u128| -> Integer {
        if n == 0 {
            Integer::from(0_usize)
        } else {
            (1..=n).map(Integer::from).product()
        }
    };

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>, where 0 <= n", args[0]);
        std::process::exit(1);
    }

    let n: u128 = args[1]
        .parse()
        .expect("Please provide a valid integer > 0s");

    println!("fac({n}) = {}", fac(n));
    Ok(())
}
