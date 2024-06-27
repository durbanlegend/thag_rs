/*[toml]
[dependencies]
regex = "1.10.5"
*/

use regex::Regex;
use std::io::{self, Read};

/// Unescape \n markers in a string to convert the wall of text to readable lines.
/// This version using regex seems to work where the original approach using .lines() fails.
/// Tip: Regex tested using https://rustexp.lpil.uk/.
#[inline]
pub(crate) fn disentangle(text_wall: &str) -> String {
    use std::fmt::Write;
    // We extract the non-greedy capturing group named "line" from each capture of the multi-line mode regex..
    let re = Regex::new(r"(?m)(?P<line>.*?)(?:[\\]n|$)").unwrap();
    re.captures_iter(text_wall)
        .map(|c| c.name("line").unwrap().as_str())
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        })
}

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() {
    println!("Type text wall at the prompt and hit Ctrl-D when done");
    let content = read_stdin().expect("Problem reading input");
    println!("Disentangled:\n{}", disentangle(&content));
}
