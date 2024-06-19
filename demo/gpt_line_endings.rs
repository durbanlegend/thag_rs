/*[toml]
[dependencies]
lazy_static = "1.4.0"
regex = "1.10.4"
*/

use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;

/// Prompt for and read Rust source code from stdin.
/// TODO Still haven't got to the bottom of why even dynamic \n
/// may be stubborn or not. Curreently in testing the normalize_newlines
/// method is not even needed.
pub fn read_stdin() -> Result<String, std::io::Error> {
    println!("Enter or paste lines of Rust source code at the prompt and press Ctrl-{} on a new line when done",
        if cfg!(windows) { 'Z' } else { 'D' }
    );
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().lock().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn normalize_newlines(input: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\\r\\n|\\r|\\n)").unwrap();
    }
    let lf = std::str::from_utf8(&[10_u8]).unwrap();
    RE.replace_all(input, lf).to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = read_stdin()?;

    let normalized = normalize_newlines(&input);
    println!("input={input}");
    println!("{}", normalized);
    Ok(())
}
