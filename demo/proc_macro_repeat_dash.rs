#![allow(dead_code)]
/// Exploring expansion: function-like proc macro.
//# Purpose: explore proc macros
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::repeat_dash;

fn main() {
    repeat_dash!(70);
    println!("DASH_LINE = {DASH_LINE}");
}
