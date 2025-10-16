/*[toml]
[dependencies]
serde = { version = "1.0.219", features = ["derive"] }
syn = { version = "2", features = ["extra-traits", "full", "parsing"] }
*/

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
                    // if let Some(first_segment) = use_tree_path.ident {
                    let crate_name = use_tree_path.ident.to_string();
                    if crate_name != "crate" {
                        // Filter out "crate" entries
                        dependencies.push(crate_name);
                    }
                    // }
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
    let source_code = r#"
        extern crate serde;
        use std::io;
        use crate::my_module::{MyStruct, my_function};
        use crate::my_other_module as other;
        use proc_macro::TokenStream;
        #[macro_use(lazy_static)]
        extern crate lazy_static;
        use owo_colors::colors::xterm::{AltoBlue, Alto, AltoBeige};
        use crate; // "crate" entry
        use std; // Duplicate entry

        fn main() {
            let _ = serde_json::to_string(&my_variable);
            let _ = other::MyStruct::new();
        }
    "#;

    let dependencies = extract_dependencies(source_code);
    println!("External dependencies: {:?}", dependencies);
}
