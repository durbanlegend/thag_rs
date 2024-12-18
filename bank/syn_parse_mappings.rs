/*[toml]
[dependencies]
quote = "1.0.37"
serde = { version = "1.0.215", features = ["derive"] }
syn = { version = "2.0.90", features = ["extra-traits", "full", "parsing", "printing"] }
*/

use quote::quote;
use syn::Expr;

const MAPPINGS: [(i32, &str, &str); 7] = [
    (10, "Key bindings", "Description"),
    (20, "q, Esc", "Close the file dialog"),
    (30, "j, ↓", "Move down in the file list"),
    (40, "k, ↑", "Move up in the file list"),
    (50, "Enter", "Select the current item"),
    (60, "u", "Move one directory up"),
    (70, "I", "Toggle showing hidden files"),
];

let mappings_str = r#"{
    const MAPPINGS: [(i32, &str, &str); 7] = [
        (10, "Key bindings", "Description"),
        (20, "q, Esc", "Close the file dialog"),
        (30, "j, ↓", "Move down in the file list"),
        (40, "k, ↑", "Move up in the file list"),
        (50, "Enter", "Select the current item"),
        (60, "u", "Move one directory up"),
        (70, "I", "Toggle showing hidden files"),
    ];
}"#;

fn get_expr() -> syn::ItemConst {
    let expr: syn::ItemConst = syn::parse_quote!(
        const MAPPINGS: [(i32, &str, &str); 7] = [
            (10, "Key bindings", "Description"),
            (20, "q, Esc", "Close the file dialog"),
            (30, "j, ↓", "Move down in the file list"),
            (40, "k, ↑", "Move up in the file list"),
            (50, "Enter", "Select the current item"),
            (60, "u", "Move one directory up"),
            (70, "I", "Toggle showing hidden files"),
        ];
    );
    expr
}

// let expr = syn::parse_str::<Expr>(mappings_quote)?;

println!("expr={:#?}", get_expr());
