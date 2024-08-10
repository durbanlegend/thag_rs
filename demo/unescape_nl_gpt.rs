/*[toml]
[dependencies]
lazy_static = "1.4.0"
regex = "1.10.4"
*/

use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;

/// Prompt for and read Rust source code from stdin. Significantly modified from original
/// GPT-generated version.
/// Caveat: I'm not sure that this is foolproof. Note also that a compiled Rust string
/// literal represents the newline character differently from one that is input at
/// runtime.
//# Purpose: Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.
pub fn read_stdin() -> Result<String, std::io::Error> {
    println!("Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().lock().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn normalize_newlines(input: &str) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(\\r\\n|\\r|\\n)").unwrap();
        static ref RE2: Regex = Regex::new(r#"(\\")"#).unwrap();
    }
    let lf = std::str::from_utf8(&[10_u8]).unwrap();
    let s = RE1.replace_all(input, lf);
    // Remove backslash escapes from double quotes.
    let dq = r#"""#;

    RE2.replace_all(&s, dq).to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Type text wall at the prompt and hit Ctrl-D on a new line when done");

    let input = read_stdin()?;

    let normalized = normalize_newlines(&input);
    // println!("input={input}");
    println!("\n\n{}", normalized);
    Ok(())
}
