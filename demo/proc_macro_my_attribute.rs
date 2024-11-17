/// Exploring proc macro expansion. Expansion may be enabled via the `enable` feature (default = ["expand"]) in
/// `demo/proc_macros/Cargo.toml` and the expanded macro will be displayed in the compiler output.
//# Purpose: Sample model of a basic attribute proc macro.
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::my_attribute;

#[my_attribute]
fn my_function() {
    let not_in_use = "abc";
    println!("Hello, world!");
}

my_function();
