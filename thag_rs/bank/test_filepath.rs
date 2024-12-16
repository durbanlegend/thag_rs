/*[toml]
[dependencies]
regex = "1.10.4"
*/
use regex::Regex;
use std::io::{self, Read};

/// Unescape \n markers in a string to convert the wall of text to readable lines.
/// This version using regex seems to work where the original approach using .lines() fails.
/// Tip: I pre-test regular expressions using the very useful https://rustexp.lpil.uk/.
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

fn main() -> std::io::Result<()> {
    let path = "/Users/donf/projects/thag_rs/demo/test_filepath.rs";
    let contents = std::fs::read_to_string(path)?;
    println!("contents={}", disentangle(&contents));

    Ok(())
}
