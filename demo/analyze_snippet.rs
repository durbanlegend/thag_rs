/*[toml]
[dependencies]
syn = { version = "2.0.68", features = ["extra-traits", "full", "visit"] }
proc-macro2 = "1.0.86"
quote = "1.0.36"
strum = { version = "0.26.3", features = ["derive", "phf"] }
*/

extern crate quote;
extern crate syn;

use quote::ToTokens;
use syn::{parse_str, Expr, Stmt};

fn main() {
    let code = r#"
        for i in 1..=5 {
            println!("{}", i);
        }
    "#;

    match parse_str::<Expr>(code) {
        Ok(expr) => {
            if is_last_stmt_unit(&expr) {
                println!("Option B: \n{}", wrap_in_main(&expr));
            } else {
                println!("Option A: \n{}", wrap_in_main_with_println(&expr));
            }
        }
        Err(e) => eprintln!("Failed to parse expression: {:?}", e),
    }
}

fn is_last_stmt_unit(expr: &Expr) -> bool {
    println!("expr={expr:#?}");
    match expr {
        Expr::ForLoop(_) => true,
        Expr::While(_) => true,
        Expr::Loop(_) => true,
        Expr::If(expr_if) => expr_if.else_branch.is_none(),
        Expr::Block(expr_block) => {
            if let Some(last_stmt) = expr_block.block.stmts.last() {
                match last_stmt {
                    Stmt::Expr(_, None) => {
                        println!("Stmt::Expr(_, None)");
                        false
                    } // Expression without semicolon
                    Stmt::Expr(_, Some(_)) => {
                        println!("Stmt::Expr(_, Some(_))");
                        true
                    } // Expression with semicolon returns unit
                    Stmt::Macro(m) => {
                        let is_some = m.semi_token.is_some();
                        println!("Stmt::Macro({m:#?}), m.semi_token.is_some()={is_some}");
                        is_some
                    } // Macro with a semicolon returns unit
                    _ => {
                        println!("Something else, returning false");
                        false
                    }
                }
            } else {
                println!("Not if let Some(last_stmt) = expr_block.block.stmts.last()");
                false
            }
        }
        _ => {
            println!("Not if let Expr::Block(expr_block) = expr");
            false
        }
    }
}

fn wrap_in_main(expr: &Expr) -> String {
    format!(
        r#"
fn main() {{
    {expr}
}}
"#,
        expr = expr.to_token_stream()
    )
}

fn wrap_in_main_with_println(expr: &Expr) -> String {
    format!(
        r#"
fn main() {{
    println!("{{expr:?}}", {{
        {expr}
    }});
}}
"#,
        expr = expr.to_token_stream()
    )
}
