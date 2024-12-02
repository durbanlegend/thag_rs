/*[toml]
[dependencies]
syn = { version = "2.0.90", features = ["extra-traits", "full", "visit"] }
quote = "1.0.37"
*/

use syn::{parse_quote, Stmt};

fn main() {
    // let name = quote!(v);
    // let ty = quote!(u8);

    let stmt: Stmt = parse_quote! {
        // let #name: #ty = Default::default();
        const MAPPINGS: [(i32, &str, &str); 7] = [
            (10, "Key bindings", "Description"),
            (20, "q, Esc", "Close the file dialog"),
            (30, "j, ↓", "Move down in the file list"),
            (40, "k, ↑", "Move up in the file list"),
            (50, "Enter", "Select the current item"),
            (60, "u", "Move one directory up"),
            (70, "I", "Toggle showing hidden files"),
        ];
    };

    println!("{:#?}", stmt);
}
