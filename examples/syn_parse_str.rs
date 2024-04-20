//! [dependencies]
//! syn = { version = "2.0.60", features = ["extra-traits"] }

use syn::{Expr, Type};

// let t: Type = syn::parse_str("std::collections::HashMap<String, Value>")?;
// let t: Type = syn::parse_str("3 + 5")?;
let code = "assert_eq!(u8::max_value(), 255)";
let expr = syn::parse_str::<Expr>(code)?;
 println!("{:#?}", expr);

// println!("type={t:?}");

