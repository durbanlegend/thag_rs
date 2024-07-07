/*[toml]
[dependencies]
quote = "1.0.36"
syn = { version = "2.0.69", features = ["full"] }
*/

use quote::quote;
use std::io::Read;
use syn::{self, Expr};

fn main() {
    loop {
        println!("Enter an expression (e.g., 2 + 3): ");
        let mut input = Vec::<u8>::new();
        std::io::stdin()
            .read_to_end(&mut input)
            .expect("Failed to read input");

        let input = match std::str::from_utf8(&input) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        // Parse the expression string into a syntax tree
        let expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(&input.trim());

        match expr {
            Ok(expr) => {
                // Generate Rust code for the expression
                let rust_code = quote!(println!("result={}", #expr););

                eprintln!("rust_code={rust_code}");
            }
            Err(err) => println!("Error parsing expression: {}", err),
        }
    }
}
