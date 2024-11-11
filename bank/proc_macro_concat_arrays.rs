#![allow(dead_code)]
/// Published example from `https://github.com/redmcg/const_gen_proc_macro`
//# Purpose: Use proc macros to generate constants at compile time
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::string_array_macro;

// In shared_module.rs

// #[macro_export]
// macro_rules! first_array {
//     () => {
//         &["Hello", "world"]
//     };
// }

// fn main() {
//     // Concatenate `first_array` with a second array
//     const RESULT: &[&str] = concat_arrays!(first_array, ["from", "macro"]);
//     println!("{:?}", RESULT); // Expected output: ["Hello", "world", "from", "macro"]
// }

const RESULT: &[&str] = string_array_macro! {
    ["hello", "world"],
    ["from", "macro"]
};
