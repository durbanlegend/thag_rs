#![allow(dead_code)]
/// Published example from crate `expander`
//# Purpose: debug proc macros
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::baz;

#[baz]
struct X<'a> {
    y: &'a [u8; 12],
}

fn main() {
    let x = X { y: b"Hello world!" };
    println!("x.clone()={:#?}", x.clone());
}
