/*[toml]
[dependencies]
syn = { version = "2", features = ["extra-traits", "full", "parsing"] }
thag_common = { version = "1, thag-auto" }
*/

/// Tries to convert input to a `syn` abstract syntax tree (`syn::File` or `syn::Expr`).
//# Purpose: Debugging
//# Categories: AST, crates, technique, tools
use quote::quote;
use std::io::{self, Read};
use thag_common::{auto_help, help_system::check_help_and_exit};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    let content = read_stdin().expect("Problem reading input");
    eprintln!("[{content:#?}]");
    if let Ok(file) = syn::parse_str::<syn::File>(&content) {
        println!("{file:#?}");
        eprintln!("[{}]", quote!(#file));
    } else {
        let expr = syn::parse_str::<syn::Expr>(&format!("{{ {content} }}"))?;
        println!("{expr:#?}");
        eprintln!("[{}]", quote!(#expr));
    }
    Ok(())
}
