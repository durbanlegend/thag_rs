/// Converts embedded manifest format from `thag` to `rust-script`.
//# Purpose: Convenience for any `thag` user who wants to try out `rust-script`.
//# Categories: crates, tools
use std::io::{self, Read, Write};

use thag_rs::{auto_help, help_system::check_help_and_exit};
fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer)?;
    Ok(buffer)
}

// Tolerate a broken pipe caused by e.g. piping to `head`.
// See https://github.com/BurntSushi/advent-of-code/issues/17
fn safe_println(line: &str) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if let Err(e) = writeln!(handle, "{line}") {
        if e.kind() == io::ErrorKind::BrokenPipe {
            // eprintln!("Broken pipe error: {}", e);
            return Ok(());
        }
        return Err(e);
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_to_rust_script");
    check_help_and_exit(&help);

    let content = read_stdin().expect("Problem reading input");
    let mut is_cargo = false;

    for line in content.lines() {
        if line.trim().starts_with(format!("/{}[toml]", '*').as_str()) {
            // Flag cargo section
            is_cargo = true;
            safe_println("//! ```cargo")?;
            continue;
        }
        if line.contains(r"*/") {
            // Flag end of cargo section
            is_cargo = false;
            safe_println("//! ```")?;
            continue;
        }
        if is_cargo {
            // Preserve toml
            safe_println(&format!("//! {line}"))?;
        } else {
            // Preserve Rust source
            safe_println(line)?;
        }
    }
    Ok(())
}