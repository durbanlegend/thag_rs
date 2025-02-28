/*[toml]
[dependencies]
syn = { version = "2", features = ["extra-traits", "full", "parsing"] }
*/

/// Tries to convert input to a `syn` abstract syntax tree (`syn::File` or `syn::Expr`).
//# Purpose: Debugging
//# Categories: AST, crates, technique, tools
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
    eprintln!("[{:#?}]", content);
    match syn::parse_str::<syn::File>(&content) {
        Ok(file) => println!("{file:#?}"),
        Err(_) => match syn::parse_str::<syn::Expr>(&format!("{{ {content} }}")) {
            Ok(expr) => println!("{expr:#?}"),
            Err(err) => return Err(err.into()),
        },
    };
    Ok(())
}
