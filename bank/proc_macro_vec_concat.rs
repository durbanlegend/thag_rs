#![allow(dead_code)]
/// Published example from `https://github.com/redmcg/const_gen_proc_macro`
//# Purpose: Use proc macros to generate constants at compile time
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::string_concat;

let other = "Other";

string_concat! {
    let prefix: Vec<usize> = vec::new("1, 2, 3");
    let variable: Vec<usize> = vec![4, 5];
    const VARIABLE: Vec<usize> = prefix.concat(variable);
}

assert_eq!(VARIABLE, "A Variable");

println!("VARIABLE={VARIABLE}");
