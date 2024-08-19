/// Converts embedded manifest format from `rust-script` to `thag`.
///
/// E.g. `cat <path_to_rust_script_file> | thag -qq demo/thag_from_rust_script.rs | thag -s [-- [options] [args] ...]`
///
/// Place any command-line options and/or arguments for the script at the end after a -- as shown.
///
//# Purpose: Convenience for any `rust-script` user who wants to try out `thag`.
use std::io::{self, Read, Write};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

// Tolerate a broken pipe caused by e.g. piping to `head`.
// See https://github.com/BurntSushi/advent-of-code/issues/17
fn safe_println(line: &str) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if let Err(e) = writeln!(handle, "{}", line) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            // eprintln!("Broken pipe error: {}", e);
            return Ok(());
        } else {
            return Err(e);
        }
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let content = read_stdin().expect("Problem reading input");
    let mut is_cargo = false;

    for line in content.lines() {
        if line.trim().starts_with("//!") {
            if line.contains("```cargo") {
                // Flag cargo section
                is_cargo = true;
                safe_println("/*[toml]")?;
                // writeln!(io::stdout(), "{:?}", "/*[toml]".as_bytes());
                continue;
            }
            if line.contains("```") {
                // Flag end of cargo section
                is_cargo = false;
                // writeln!(io::stdout(), "{}/", '*')?;
                safe_println(&format!("{}/", '*'))?;
                continue;
            }
            if !is_cargo {
                // Drop all non-cargo "//!" lines.
                continue;
            }
            // Preserve toml
            let line = line.trim_start_matches("//!").trim_start();
            safe_println(&line)?;
        } else {
            safe_println(&line)?;
        }
    }
    Ok(())
}
