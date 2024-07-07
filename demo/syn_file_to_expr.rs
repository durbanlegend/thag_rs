/*[toml]
[dependencies]
syn = { version = "2.0.69", features = ["extra-traits", "full", "visit"] }
quote = "1.0.36"
*/

extern crate syn;

use quote::quote;
use syn::{parse_str, Expr, File, Item, Stmt};

fn main() {
    let code = r#"
        fn main() {
            use inline_colorization::{color_red, color_reset, style_reset, style_underline};
            println!("Lets the user {color_red}colorize{color_reset} and {style_underline}style the output{style_reset} text using inline variables");
        }
    "#;

    let file = parse_str::<File>(code).expect("Unable to parse code");
    if let Some(expr) = extract_expr_from_file(&file) {
        println!("Extracted expression: {:#?}", expr);
    } else {
        println!("No expression extracted");
    }
}

fn extract_expr_from_file(file: &File) -> Option<Expr> {
    // Traverse the file to find the main function and extract expressions from it
    for item in &file.items {
        if let Item::Fn(func) = item {
            if func.sig.ident == "main" {
                let stmts = &func.block.stmts;
                // Collect expressions from the statements
                let exprs: Vec<Expr> = stmts
                    .iter()
                    .filter_map(|stmt| match stmt {
                        Stmt::Expr(expr, _) => Some(expr.clone()),
                        Stmt::Macro(macro_stmt) => {
                            let mac = &macro_stmt.mac;
                            let macro_expr = quote! {
                                #mac
                            };
                            Some(
                                parse_str(&macro_expr.to_string())
                                    .expect("Unable to parse macro expression"),
                            )
                        }
                        _ => None,
                    })
                    .collect();

                // Combine the expressions into a single expression if needed
                if !exprs.is_empty() {
                    let combined_expr = quote! {
                        { #(#exprs);* }
                    };
                    return Some(
                        parse_str(&combined_expr.to_string())
                            .expect("Unable to parse combined expression"),
                    );
                }
            }
        }
    }
    None
}
