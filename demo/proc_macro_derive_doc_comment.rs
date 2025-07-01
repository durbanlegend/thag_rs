#![allow(dead_code)]
/// Exploring exposing doc comments at runtime.
/// Example from https://www.reddit.com/r/rust/comments/pv5v3x/looking_for_a_minimal_example_on_how_to_parse_doc/
//# Purpose: explore proc macros
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveDocComment;

#[derive(Debug, DeriveDocComment)]
enum MyEnum {
    /// Doc comment A
    A,
    /// Doc comment B
    B,
}

fn main() {
    let my_enum = MyEnum::A;
    println!(
        "my_enum = {my_enum:#?}, my_enum.doc_comment()=[{}]",
        my_enum.doc_comment()
    );
}
