/*[toml]
[dependencies]
syn = { version = "2.0.87", features = ["extra-traits", "full", "visit"] }
*/

use syn::{parse_str, Expr, Stmt};

/// Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
/// to detect whether a snippet returns a value that we should print out, or whether
/// it does its own printing.
///
/// Part 1: After some back and forth with ChatGPT suggesting solutions it finally generates essentially this.
//# Purpose: Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.
//# Categories: AST, technique
fn main() {
    let snippet = r#"
        use inline_colorization::{color_red, color_reset, style_reset, style_underline};
        println!("Lets the user {color_red}colorize{color_reset} and {style_underline}style the output{style_reset} text using inline variables");
    "#;

    if should_wrap_with_println(snippet) {
        println!("Option A: Wrap with println!");
    } else {
        println!("Option B: Do not wrap with println!");
    }
}

fn should_wrap_with_println(snippet: &str) -> bool {
    // Parse the snippet into an expression
    match parse_str::<Expr>(snippet) {
        Ok(expr) => {
            // Check if the expression returns a non-unit value
            !returns_unit(&expr)
        }
        Err(_) => {
            // If parsing as an expression fails, parse as statements
            match syn::parse_file(&format!("fn main() {{ {} }}", snippet)) {
                Ok(file) => {
                    // Check the last statement in the function block
                    if let Some(last_stmt) = file.items.iter().find_map(|item| {
                        if let syn::Item::Fn(func) = item {
                            func.block.stmts.last()
                        } else {
                            None
                        }
                    }) {
                        !is_println_macro(last_stmt)
                    } else {
                        true // Default to wrapping if we can't determine
                    }
                }
                Err(_) => true, // Default to wrapping if parsing fails
            }
        }
    }
}

fn returns_unit(expr: &Expr) -> bool {
    // Check if the expression returns a unit value
    matches!(expr, Expr::Tuple(tuple) if tuple.elems.is_empty())
}

fn is_println_macro(stmt: &Stmt) -> bool {
    match stmt {
        // Check if the statement is a macro
        Stmt::Macro(mac_stmt) => {
            if mac_stmt.mac.path.is_ident("println") {
                return true;
            }
        }
        // Check if the statement is an expression with an optional semi-colon
        Stmt::Expr(expr, _) => {
            if let Expr::Macro(expr_macro) = expr {
                if expr_macro.mac.path.is_ident("println") {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}
