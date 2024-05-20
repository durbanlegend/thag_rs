let source_toml = r##"
//!
//!    [package]
//!    name = "factorial_main"
//!    version = "0.0.1"
//!    edition = "2021"
//!
//!    [dependencies]
//!    rug = { version = "1.24.0", features = ["integer"] }
//!
//!    [workspace]
//!
//!    [[bin]]
//!    name = "factorial_main"
//!    path = "/Users/donf/projects/rs-script/.cargo/rs-script/tmp_source.rs"
//!

use rug::Integer;
use std::error::Error;
use std::io;
use std::io::Read;

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
"##;

use std::fmt::Write;

println!("Rust raw source ={source_toml}");

let pat: &[_] = &['/', '/', '!'];
let toml_str = source_toml
    .lines()
    .map(str::trim_start)
    .filter(|&line| line.starts_with("//!"))
    .map(|line| line.trim_start_matches(pat))
    .fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    });

println!("Rust source manifest info ={toml_str}");
