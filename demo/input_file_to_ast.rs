/*[toml]
[dependencies]
syn = { version = "2.0.90", features = ["extra-traits", "full", "parsing"] }
*/

/// Tries to convert input to a `syn` abstract syntax tree (syn::File).
//# Purpose: Debugging
//# Categories: AST, crates, technique
use std::io::{self, Read};
use syn;

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin().expect("Problem reading input");
    println!("[{:#?}]", content);
    let syntax: syn::File = syn::parse_str(&content)?;
    println!("{:#?}", syntax);
    Ok(())
}
