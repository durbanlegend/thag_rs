#![allow(dead_code)]
/// Exploring expansion
//# Purpose: explore proc macros
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::my_macro;
fn main() {
    my_macro!(42);

    println!("VALUE={VALUE}");
}
