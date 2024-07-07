/*[toml]
[dependencies]
syn = { version = "2.0.69", features = ["extra-traits", "full", "visit"] }
quote = "1.0.36"
*/

use quote::ToTokens;
use syn::{parse_file, Item};

fn main() {
    let code = r#"
    fn foo() -> bool {
        true
    }
    "#;

    match parse_file(code) {
        Ok(ast) => {
            for item in ast.items {
                if let Item::Fn(func) = item {
                    println!("Function name: {}", func.sig.ident);
                }
            }
        }
        Err(e) => eprintln!("Unable to parse file: {:?}", e),
    }
}
