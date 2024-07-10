/*[toml]
[dependencies]
syn = { version = "2.0.69", features = ["extra-traits", "full", "visit"] }
quote = "1.0.36"
*/

use quote::quote;
use syn::{parse_quote, parse_str, Expr, File, Item, Stmt};

let expr: Expr = parse_quote! {
    {
        // fn fib_fn(n: usize) -> impl Iterator<Item = (IBig, IBig)> {
        //     successors(Some((IBig::from(0), IBig::from(1))), |(a, b)| Some((b.clone(), (a + b).into())))
        // .take(n + 1)
        a += 1
    }
};

println!("expr={expr:#?}");
