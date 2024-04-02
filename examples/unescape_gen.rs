use regex::Regex;
use std::io::{self, Read};

/// Remove leading and trailing double quotes and unescape embedded quotes
/// from a string. Intended for cleaning up old history logs (examples/y) after
/// using unescape_nl2.rs.
#[inline]
pub(crate) fn disentangle(text_wall: &str) -> String {
    use std::fmt::Write;
    // let re = Regex::new(r"(?m)^(runner [\-]{1,2}(?:add|doc|crates)(?:[\\]n|$))").unwrap();
    text_wall
        .lines()
        .map(|b| b.trim_matches('"'))
        .map(|b| b.replace('\\', ""))
        // .filter(|b| !re.is_match(b))
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
