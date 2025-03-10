/*[toml]
[dependencies]
syn = { version = "2.0.90", features = ["full"] }
# thag_proc_macros = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop" }
thag_proc_macros = { version = "0.1.1", path = "/Users/donf/projects/thag_rs/thag_proc_macros" }
toml = "0.8"
*/
/// Exploring proc macro expansion. Expansion may be enabled via the `enable` feature (default = ["expand"]) in
/// `demo/proc_macros/Cargo.toml` and the expanded macro will be displayed in the compiler output.
//# Purpose: Debug a proc macro under development
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_proc_macros::preload_themes;
fn main() {
    preload_themes {}

    // println!("VALUE={VALUE}");
}
