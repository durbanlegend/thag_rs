/*[toml]
[dependencies]
strum = { version = "0.27", features = ["derive", "phf"] }
# The 'thag-auto' keyword automatically resolves dependencies based on your environment:
# - Default: Uses crates.io (no environment variables needed)
# - Development: Set THAG_DEV_PATH=/absolute/path/to/thag_rs (e.g. $PWD not .)
# - Git: Set THAG_GIT_REF=main (or other branch) to use git repository instead of crates.io
# Note: Run with 'thag script.rs' not 'cargo build' to enable thag-auto processing
thag_proc_macros = { version = "0.2, thag-auto" }
*/

/// Try generating category enum.
/// Testing the `category_enum` proc macro for use with `demo/gen_readme.rs` and `demo/filter_demos.rs`/
//# Purpose: Test the proof of concept and potentially the implementation.
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

    let variant = Category::from_str("learning");
    println!("variant={variant:#?}");

    let all_cats = all_categories();
    println!("all_cats={all_cats:#?}");
}
