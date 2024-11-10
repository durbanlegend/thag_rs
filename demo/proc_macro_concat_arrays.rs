#![allow(dead_code)]
/// Published example from `https://github.com/redmcg/const_gen_proc_macro`
//# Purpose: Use proc macros to generate constants at compile time
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::concat_arrays;

fn main() {
    // Concatenating two arrays of `&str`
    const RESULT: &[&str] = concat_arrays!(["Hello", "world"], ["from", "macro"]);
    println!("{:?}", RESULT); // Should output: ["Hello", "world", "from", "macro"]
}
