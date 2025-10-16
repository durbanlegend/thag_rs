/*[toml]
[dependencies]
strum = { version = "0.27", features = ["derive", "phf"] }
# The `thag` command uses the `thag-auto` keyword here to resolve dependencies automatically based on your environment:
# - Default: Uses crates.io (no environment variables needed)
# - Development: Set THAG_DEV_PATH=/absolute/path/to/thag_rs (e.g. $PWD not .)
# - Git: Set THAG_GIT_REF=main (or other branch) to use git repository instead of crates.io
# E.g. from `thag_rs` project dir: `THAG_DEV_PATH=$PWD thag demo/proc_macro_category_enum.rs`
thag_proc_macros = { version = "0.2, thag-auto" }
*/

/// Try generating category enum.
/// Testing the `category_enum` proc macro for use with `demo/gen_readme.rs` and `demo/filter_demos.rs`/
//# Purpose: Test the proof of concept and potentially the implementation.
use thag_proc_macros::category_enum;

fn main() {
    category_enum! {};

    let variant = Category::from_str("learning");
    println!("variant={variant:#?}");

    let all_cats = all_categories();
    println!("all_cats={all_cats:#?}");
}
