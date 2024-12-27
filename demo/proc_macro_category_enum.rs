/*[toml]
[dependencies]
strum = { version = "0.26.3", features = ["derive", "phf"] }
*/
/// Try generating category enum.
/// Testing the `category_enum` proc macro for use with `demo/gen_readme.rs` and `demo/filter_demos.rs`/
//# Purpose: Test the proof of concept and potentially the implementation.
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_proc_macros::category_enum;

macro_rules! lazy_static_var {
    ($type:ty, $init_fn:expr, deref) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        *GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    ($type:ty, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
}

fn main() {
    category_enum! {
        #[expand_macro]
    };

    let variant = Category::from_str("educational");
    println!("variant={variant:#?}");

    let all_cats = all_categories();
    println!("all_cats={all_cats:#?}");
}
