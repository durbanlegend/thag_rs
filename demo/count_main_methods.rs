/*[toml]
[dependencies]
syn = { version = "2.0.87", features = ["full", "visit"] }
*/

use syn::visit::Visit;
use syn::Expr;

/// Prototype of a function required by thag_rs to count the main methods
/// in a script to decide if it's a program or a snippet. Uses the `syn`
/// visitor pattern. This is more reliable than a simple source code search
/// which tends to find false positives in string literals and comments.
//# Purpose: Demo prototyping with thag_rs and use of the `syn` visitor pattern to visit nodes of interest
fn count_main_methods(rs_source: &str) -> usize {
    // Parse the source code into a syntax tree
    let mut maybe_ast: Result<Expr, syn::Error> = syn::parse_str::<Expr>(rs_source);
    if maybe_ast.is_err() && !(rs_source.starts_with('{') && rs_source.ends_with('}')) {
        // Try putting the expression in braces.
        let string = format!(r"{{{rs_source}}}");
        let str = string.as_str();
        // vlog!(Verbosity::Normal, "str={str}");

        maybe_ast = syn::parse_str::<Expr>(str);
    }
    let ast: Expr = match maybe_ast {
        Ok(tree) => tree,
        Err(_) => return 0, // Return 0 if parsing fails
    };

    // Count the number of main() methods
    #[derive(Default)]
    struct FindMainFns {
        main_method_count: usize,
    }

    impl<'a> Visit<'a> for FindMainFns {
        fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
            if node.sig.ident == "main" && node.sig.inputs.is_empty() {
                self.main_method_count += 1;
            }
        }
    }

    let mut finder = FindMainFns::default();
    finder.visit_expr(&ast);

    finder.main_method_count
}

fn main() {
    let source_code = r#"
        // Some comments
        fn main() {
            println!("Hello, world!");
        }

        // More Rust code

        fn main() {
            println!("Hello again!");
        }
    "#;

    let main_method_count = count_main_methods(source_code);
    println!("Number of main() methods: {}", main_method_count);
}
