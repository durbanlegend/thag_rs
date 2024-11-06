/*[toml]
[dependencies]
quote = "1.0.37"
syn = { version = "2.0.87", features = ["extra-traits", "full", "parsing", "visit", "visit-mut"] }
*/

/// Prototype that uses the Visitor pattern of the `syn` crate to identify `use` statements that exist
/// for the purpose of renaming a dependency so that we don't go looking for the temporary in the registry.
/// Specifically the combination of fn `visit_use_rename` to process the nodes representing `extern crate`
/// statements and fn `` to initiate the tree traversal. This version expects the script contents
/// to consist of a full-fledged Rust program.
//# Purpose: Demo featured crate.
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

    struct FindCrates;
    impl<'ast> Visit<'ast> for FindCrates {
        fn visit_use_rename(&mut self, node: &'ast syn::UseRename) {
            println!("{}", node.rename);
        }
    }

    FindCrates.visit_file(&code);
}
