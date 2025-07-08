/// Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
/// Rust source program passed to the script. Specifically the combination of fn `visit_item_extern_crate`
/// to process the nodes representing `extern crate` statements and fn `visit_expr` to start the tree
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
        fn visit_item_extern_crate(&mut self, i: &'ast syn::ItemExternCrate) {
            println!("{:#?}", i);
        }
    }

    FindCrates.visit_expr(&code);
}
