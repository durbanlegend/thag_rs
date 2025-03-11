/*[toml]
[dependencies]
quote = "1.0.37"
syn = { version = "2", features = ["extra-traits", "full", "parsing", "visit", "visit-mut"] }
*/

/// Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
/// Rust source expression passed to the script. Specifically the combination of fn `visit_use_path`
/// to process the nodes representing `use` statements and fn `visit_expr` to initiate the tree
/// traversal. This version expects the script contents to consist of a Rust expression.
//# Purpose: Prototype.
//# Categories: AST, crates, prototype, technique
//# Sample arguments: `-- demo/just_a_test_expression.rs`
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
    let code: Expr = parse_str::<Expr>(&content).unwrap();

    struct FindCrates;
    impl<'ast> Visit<'ast> for FindCrates {
        fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
            println!("{:#?}", node);
            println!("{}", node.ident.to_string());
        }
    }

    FindCrates.visit_expr(&code);
}
