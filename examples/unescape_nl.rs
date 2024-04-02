/// Unescape \n markers in a string to convert the wall of text to readable lines.
/// See unescape_nl2.rs for a Regex version that seems to work
#[inline]
pub(crate) fn reassemble<'a>(iter: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    iter.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers in a string to convert the wall of text to readable lines.
#[inline]
pub(crate) fn disentangle(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

use std::{
    io::{self, Read},
    option::Iter,
};

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
