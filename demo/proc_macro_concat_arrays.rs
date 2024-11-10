#![allow(dead_code)]
/// Published example from `https://github.com/redmcg/const_gen_proc_macro`
//# Purpose: Use proc macros to generate constants at compile time
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::concat_arrays;

// let base = vec!["a".to_string(), "b".to_string()];
// let additions = vec!["c".to_string()];

// vec_concat! {
//     const MAPPINGS: &[String] = base.concat(additions);
// }

// assert_eq!(MAPPINGS,["a".to_string(), "b".to_string(), "c".to_string()]);

// println!("MAPPINGS={MAPPINGS}");

fn main() {
    // Using the macro to concatenate two arrays
    let result = concat_arrays!(["1", "2", "3"], ["4", "5", "6"]);
    println!("{:?}", result); // Output should be: [1, 2, 3, 4, 5, 6]
}
