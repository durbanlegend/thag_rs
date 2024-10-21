/*[toml]
[dependencies]
serde = "1.0.204"
serde_json = "1.0.132"
*/

/// Demo of using deserializing JSON with the featured crates.
/// This version prompts for JSON input.
//# Purpose: Demo featured crates.
use serde::de::Deserialize;
use serde_json::Value;

// Prompt for and read Rust source code from stdin.
pub fn read_stdin() -> Result<String, std::io::Error> {
    println!("Enter or paste lines of JSON at the prompt and press Ctrl-D on a new line when done");
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().lock().read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}

let buffer = read_stdin()?;

println!(
    "{:#?}",
    serde_json::from_str::<Value>(
        &buffer
    )
    .unwrap()
);
