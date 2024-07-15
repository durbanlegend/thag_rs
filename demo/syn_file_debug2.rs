/*[toml]
[dependencies]
syn = { version = "2.0.71", features = ["extra-traits", "full", "visit"] }
*/

use syn::parse_file;

fn main() {
    let code = r#"
    fn foo() -> bool {
        true
    }

    foo()
"#;

    match parse_file(code) {
        Ok(ast) => {
            println!("Parsed AST successfully!");
            // Proceed with further processing of `ast`
        }
        Err(e) => eprintln!("Unable to parse file: {:?}", e),
    }
}
