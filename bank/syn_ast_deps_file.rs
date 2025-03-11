/*[toml]
[dependencies]
serde = { version = "1.0.219", features = ["derive"] }
syn = { version = "2", features = ["extra-traits", "full", "parsing"] }
*/

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use syn::{Item, UseTree};

fn extract_dependencies(source_code: &str) -> Vec<String> {
    let syntax_tree = syn::parse_file(source_code).expect("Failed to parse Rust source code");
    let mut dependencies = Vec::new();

    for item in syntax_tree.items {
        match item {
            Item::ExternCrate(extern_crate) => {
                dependencies.push(extern_crate.ident.to_string());
            }
            Item::Use(use_item) => {
                if let UseTree::Path(use_tree_path) = use_item.tree {
                    let crate_name = use_tree_path.ident.to_string();
                    if crate_name != "crate" {
                        // Filter out "crate" entries
                        dependencies.push(crate_name);
                    }
                }
            }
            Item::Macro(macro_item) => {
                if let Some(macro_path) = macro_item.mac.path.get_ident() {
                    dependencies.push(macro_path.to_string());
                }
            }
            _ => {}
        }
    }

    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    dependencies
}

fn main() {
    let mut args = env::args_os();
    let _ = args.next(); // executable name

    let filepath = match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => {
            println!("No args entered");
            process::exit(1);
        }
    };

    let source_code = fs::read_to_string(&filepath).unwrap();
    let dependencies = extract_dependencies(&source_code);
    println!("External dependencies: {:?}", dependencies);
}
