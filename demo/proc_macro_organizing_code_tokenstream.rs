#![allow(dead_code)]
/// Published example from `https://github.com/tdimitrov/rust-proc-macro-post`
//# Purpose: explore proc macros
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::organizing_code_tokenstream;
fn main() {
    organizing_code_tokenstream!(42);
}
