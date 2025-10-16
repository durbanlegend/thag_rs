/*[toml]
[dependencies]
quote = "1.0.37"
syn = { version = "2", features = ["extra-traits", "full"] }
*/

use quote::quote;
use syn::{Expr, Type};

// let t: Type = syn::parse_str("std::collections::HashMap<String, Value>")?;
// let t: Type = syn::parse_str("3 + 5")?;
// println!("{:#?}", t);

// let code = quote!({let other = "world";
// println!("Hello {{other}}!");});
let code = quote!(i32.MAX_VALUE);
println!("code={code:#?}");
let expr = syn::parse_str::<Expr>(code.to_string().as_str())?;
 println!("{:#?}", expr);

// println!("type={t:?}");

use quote::ToTokens;

// Assume you have a `syn::Expr`
// Convert the `syn::Expr` into tokens (a string-like format)
let tokens = expr.to_token_stream().to_string();

// Now you could try running `rustfmt` on the string if you need proper formatting.
println!("tokens={}", tokens);
