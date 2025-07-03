/// Exploring proc macro expansion. Expansion may be enabled via the `enable` feature (default = ["expand"]) in
/// `demo/proc_macros/Cargo.toml` and the expanded macro will be displayed in the compiler output.
//# Purpose: Sample model of a basic function-like proc macro.
//# Categories: proc_macros, technique

// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::function_like_basic;
fn main() {
    function_like_basic!(42);

    println!("VALUE={VALUE}");
}
