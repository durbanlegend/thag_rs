/// Converts embedded manifest format from `rust-script` to `thag`.
//# Purpose: Convenience for any `rust-script` user who wants to try out `thag`.
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
        if line.trim().starts_with("//!") {
            if line.contains("```cargo") {
                // Flag cargo section
                is_cargo = true;
                println!("/*[toml]");
                continue;
            }
            if line.contains("```") {
                // Flag end of cargo section
                is_cargo = false;
                println!("{}/", '*');
                continue;
            }
            if !is_cargo {
                // Drop all non-cargo "//!" lines.
                continue;
            }
            // Preserve toml
            let line = line.trim_start_matches("//!").trim_start();
            println!("{line}");
        } else {
            // Preserve Rust source
            println!("{line}");
        }
    }
}
