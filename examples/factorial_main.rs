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
   log = "0.4.21"

   [workspace]

   [[bin]]
   name = "factorial_main"
   path = "/Users/donf/projects/rs-script/.cargo/rs-script/tmp_source.rs"

*/

use rug::Integer;
use std::error::Error;
use std::io;
use std::io::Read;

/// Fast factorial algorithm avoiding inefficient recursion.
/// Unfortunately Windows 11 won't currently run this natively.
/// Supposedly you can run it by installing MSYS2, but I haven't tested this.
fn main() -> Result<(), Box<dyn Error>> {
    let fac = |n: usize| -> Integer {
        if n == 0 {
            Integer::from(0_usize)
        } else {
            (1..=n).map(Integer::from).product()
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

    println!("fac({n}) = {}", fac(n));
    Ok(())
}
