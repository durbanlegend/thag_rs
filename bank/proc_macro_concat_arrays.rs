#![allow(dead_code)]
/// Published example from `https://github.com/redmcg/const_gen_proc_macro`
//# Purpose: Use proc macros to generate constants at compile time
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::string_array_macro;

struct Foo {
    first: Vec<String>,
    second: Vec<String>,
}

fn get_foo() -> Foo {
    Foo {
        first: vec!["hello".to_string(), "world".to_string()],
        second: vec!["from".to_string(), "macro".to_string()],
    }
}
fn main() {
    string_array_macro! {
    get_foo.merge(); }
}
