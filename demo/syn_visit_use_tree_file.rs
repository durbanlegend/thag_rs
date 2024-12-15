/*[toml]
[dependencies]
quote = "1.0.37"
syn = { version = "2.0.90", features = ["extra-traits", "full", "parsing", "visit", "visit-mut"] }
*/

/// Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
/// Rust source program passed to the script. Specifically the combination of fn `visit_use_tree`
/// to process the nodes representing `use` statements and fn `visit_file` to initiate the tree
/// traversal. This version expects the script contents to consist of a full-fledged Rust program.
//# Purpose: Develop improved algorithm for `thag_rs` that accepts imports of the form `use <crate>;` instead of requiring `use <crate>::...`.
//# Categories: AST, crates, prototype, technique
//# Sample arguments: `-- demo/syn_visit_use_tree_file.rs`
use std::{env, fs, path::PathBuf};

fn main() {
    use ::syn::{visit::*, *};

    let mut args = env::args_os();
    let _ = args.next(); // executable name

    let filepath = match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => panic!("Couldn't find filepath arg"),
    };
    let content = fs::read_to_string(&filepath).expect("Error reading file");
    let code: File = parse_file(&content).unwrap();

    // println!("ast={code:#?}");

    struct FindCrates;
    impl<'ast> Visit<'ast> for FindCrates {
        fn visit_use_tree(&mut self, node: &'ast syn::UseTree) {
            // println!("{node:#?}");
            match node {
                UseTree::Path(use_path) => {
                    println!("Path ident={:#?}", use_path.ident.to_string())
                }
                UseTree::Name(use_name) => {
                    println!("Name ident={:#?}", use_name.ident.to_string())
                }
                _ => (),
            }
        }
    }

    FindCrates.visit_file(&code);
}
