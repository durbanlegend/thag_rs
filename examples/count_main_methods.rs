//! [dependencies]
//! syn = { version = "2.0.60", features = ["full"] }

use syn::{parse_quote, File, Item, ItemFn};

fn count_main_methods(source_code: &str) -> usize {
    // Parse the source code into a syntax tree
    let syntax_tree: File = match syn::parse_str(source_code) {
        Ok(tree) => tree,
        Err(_) => return 0, // Return 0 if parsing fails
    };

    // Count the number of main() methods
    let mut main_method_count = 0;
    for item in syntax_tree.items {
        if let Item::Fn(item_fn) = item {
            // Check if the function is named "main" and has no arguments
            if item_fn.sig.ident == "main" && item_fn.sig.inputs.is_empty() {
                main_method_count += 1;
            }
        }
    }

    main_method_count
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
