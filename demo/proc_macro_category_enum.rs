/// Try generating category enum.
/// Testing the `category_enum` proc macro for use with `demo/gen_readme.rs` and `demo/filter_demos.rs`/
//# Purpose: Test the proof of concept and potentially the implementation.
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::category_enum;
fn main() {
    category_enum!();

    let variant = Category::from_str("educational");
    println!("variant={variant:#?}");

    let all_cats = all_categories();
    println!("all_cats={all_cats:#?}");
}
