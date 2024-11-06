/*[toml]
[dependencies]
quote = "1.0.37"
syn = { version = "2.0.87", features = ["full"] }
*/

/// Prototype of a simple partial expression evaluator. It solicits a Rust expression and embeds
/// it in a `println!` statement for use in generated code.
///
/// E.g.:
/// ```
/// Enter an expression (e.g., 2 + 3):
/// 5 + 8
/// rust_code=println ! ("result={}" , 5 + 8) ;
/// ```
/// Fun fact: you can paste the output into any of the `expr`, `edit`, `repl` or `stdin`
/// modes of `thag_rs`, or even into a .rs file, and it will print out the value of the
/// expression (in this case the number 13). Or you can do the same with the input (5 + 8)
/// and it will do the same because `thag_rs` will detect and evaluate an expression in
/// essentially the same way as this script does.
//# Purpose: demo expression evaluation (excluding compilation and execution) using the `syn` and `quote` crates.
use quote::quote;
use std::io::Read;
use syn::{self, Expr};

fn main() {
    loop {
        println!("Enter an expression (e.g., 2 + 3)");
        println!("Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done",);
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
