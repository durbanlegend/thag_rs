/*[toml]
[dependencies]
syn = { version = "2.0.87", features = ["extra-traits", "full", "visit"] }*/

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
