#![crate_type = "bin"]

use std::error::Error;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cargo_toml = File::create("Cargo.toml")?;
    cargo_toml.write_all(
        r#"
  [package]
  name = "factorial_main"
  version = "0.0.1"
  edition = "2021"

  [dependencies]
  rug = { version = "1.24.0", features = ["integer"] }

  [[bin]]
  name = "factorial_main"
  path = "/Users/donf/projects/build_run/.cargo/build_run/tmp_source.rs"
  "#
        .as_bytes(),
    )?;

    Ok(())
}
