/// This is an arbitrary expression for use by scripts like `demo/syn_visit_extern_crate_expr.rs`
/// and `demo/syn_visit_use_path_expr.rs`.
/// Don't remove the surrounding braces, because those serve to make it an expression.
//# Purpose: Testing.
//# Categories: testing
{
    // Do not remove: extern crate for demo/syn_visit_extern_crate_expr.rs testing
    extern crate syn;
    // Do not remove: "use"-path for demo/syn_visit_use_path_expr.rs testing
    use ::syn::{visit::*, *};

    println!("Testing testing testing");
    "Let's return something!"
}
