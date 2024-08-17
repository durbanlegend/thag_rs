/// Converts embedded manifest format from `thag` to `rust-script`.
//# Purpose: Convenience for any `thag` user who wants to try out `rust-script`.
use std::io::{self, Read};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() {
    let content = read_stdin().expect("Problem reading input");
    let mut is_cargo = false;

    for line in content.lines() {
        if line.trim().starts_with(format!("/{}[toml]", '*').as_str()) {
            // Flag cargo section
            is_cargo = true;
            println!("//! ```cargo");
            continue;
        }
        if line.contains(r#"*/"#) {
            // Flag end of cargo section
            is_cargo = false;
            println!("//! ```");
            continue;
        }
        if is_cargo {
            // Preserve toml
            println!("//! {line}");
        } else {
            // Preserve Rust source
            println!("{line}");
        }
    }
}
