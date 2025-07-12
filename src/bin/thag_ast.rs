/*[toml]
[dependencies]
syn = { version = "2", features = ["extra-traits", "full", "parsing"] }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
*/

use quote::quote;
/// Tries to convert input to a `syn` abstract syntax tree (`syn::File` or `syn::Expr`).
//# Purpose: Debugging
//# Categories: AST, crates, technique, tools
use std::io::{self, Read};
use thag_rs::{auto_help, help_system::check_help_and_exit};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_ast");
    check_help_and_exit(&help);

    let content = read_stdin().expect("Problem reading input");
    eprintln!("[{content:#?}]");
    match syn::parse_str::<syn::File>(&content) {
        Ok(file) => {
            println!("{file:#?}");
            eprintln!("[{}]", quote!(#file));
        }
        Err(_) => match syn::parse_str::<syn::Expr>(&format!("{{ {content} }}")) {
            Ok(expr) => {
                println!("{expr:#?}");
                eprintln!("[{}]", quote!(#expr));
            }
            Err(err) => return Err(err.into()),
        },
    };
    Ok(())
}
