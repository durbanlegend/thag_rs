//!
//!    [package]
//!    name = "factorial_main"
//!    version = "0.0.1"
//!    edition = "2021"
//!
//!    [dependencies]
//!    # rug = { version = "1.24.0", features = ["integer"] }
//!    # TODO out: the following 3 dependencies are just to test retrieving multiple
//!    #           dependencies from code and are not needed by this script at all.
//!    serde = { version = "1.0", features = ["derive"] }
//!    env_logger = "0.11.3"
//!    log = "0.4.21"
//!
//!    [workspace]
//!
//!    [[bin]]
//!    name = "factorial_main"
//!    path = "/Users/donf/projects/build_run/.cargo/build_run/tmp_source.rs"
//!

use rug::Integer;
use std::error::Error;
use std::io;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
    let fac = |n: usize| -> Integer {
        if n == 0 {
            Integer::from(0_usize)
        } else {
            (1..=n).map(Integer::from).product()
        }
    };

    println!("Type lines of text at the prompt and hit Ctrl-D when done");

    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer)?;

    let n: usize = buffer
        .trim_end()
        .parse()
        .expect("Can't parse input into a positive integer");

    println!("fac({n}) = {}", fac(n));
    Ok(())
}
