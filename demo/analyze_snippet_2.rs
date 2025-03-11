/*[toml]
[dependencies]
syn = { version = "2", features = ["extra-traits", "full", "visit"] }
quote = "1.0.37"
*/

use quote::ToTokens;
use syn::{parse_str, Expr, Stmt};

/// Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
/// to detect whether a snippet returns a value that we should print out, or whether
/// it does its own printing.
///
/// Part 2: ChatGPT responds to feedback with an improved algorithm.
//# Purpose: Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.
//# Categories: AST, technique
fn main() {
    let code = r#"
        for i in 1..=5 {
            println!("{}", i);
        }
    "#;

    match parse_str::<Expr>(code) {
        Ok(expr) => {
            if is_last_stmt_unit(&expr) {
                println!("Option B: unit return type \n{}", wrap_in_main(&expr));
            } else {
                println!(
                    "Option A: non-unit return type\n{}",
                    wrap_in_main_with_println(&expr)
                );
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
