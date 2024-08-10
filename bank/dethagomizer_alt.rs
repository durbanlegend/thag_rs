/*[toml]
[dependencies]
lazy_static = "1.5.0"
regex = "1.10.5"
*/
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, Read};

/// Unescape \n and \" markers in a string to convert the wall of text to readable lines.
/// This version using regex seems to work where the original approach using .lines() fails.
/// Tip: Regex tested using https://rustexp.lpil.uk/.
//# Purpose: Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.
fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[inline]
pub(crate) fn dethagomize(text_wall: &str) -> String {
    use std::fmt::Write;
    // We extract the non-greedy capturing group named "line" from each capture of the multi-line mode regex..
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?m)(?P<line>.*?)(?:\\r\\n|\\r|\\n|$)").unwrap();
        static ref RE2: Regex = Regex::new(r#"(\\")"#).unwrap();
    }
    // Remove backslash escapes from double quotes.
    let dq = r#"""#;

    RE1.captures_iter(text_wall)
        .map(|c| c.name("line").unwrap().as_str())
        .map(|s| RE2.replace_all(&s, dq).to_string())
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        })
}

fn main() {
    println!("Type text wall at the prompt and hit Ctrl-D on a new line when done");
    let content = read_stdin().expect("Problem reading input");
    println!("\n\nDethagomized:\n\n{}", dethagomize(&content));
}
