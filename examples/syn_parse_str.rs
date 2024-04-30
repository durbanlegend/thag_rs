/*[toml]
[dependencies]
quote = "1.0.36"
syn = { version = "2.0.60", features = ["extra-traits", "full"] }
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
