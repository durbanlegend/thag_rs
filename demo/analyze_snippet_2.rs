/*[toml]
[dependencies]
syn = { version = "2.0.68", features = ["extra-traits", "full", "visit"] }
quote = "1.0.36"
*/

use quote::ToTokens;
use std::collections::HashMap;
use syn::{parse_str, Expr, Item, ReturnType, Stmt};

/// Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
/// to detect whether a snippet returns a value that we should print out, or whether
/// it does its own printing.
///
/// Part 2: ChatGPT responds to feedback with an improved algorithm.
//# Purpose: Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.
fn main() {
    let code = r#"{
    fn foo() -> bool {
        true
    }

    foo()
}"#;

    let ast = parse_str::<syn::File>(code).expect("Unable to parse file");
    // let ast = syn::parse_file(code).expect("Unable to parse file");

    let function_map = extract_functions(&ast);

    match parse_str::<Expr>("foo()") {
        Ok(expr) => {
            if is_last_expr_unit(&expr, &function_map) {
                println!("Option B: unit return type \n{}", wrap_in_main(&expr));
            } else {
                println!(
                    "Option A: non-unit return type \n{}",
                    wrap_in_main_with_println(&expr)
                );
            }
        }
        Err(e) => eprintln!("Failed to parse expression: {:?}", e),
    }
}

fn extract_functions(ast: &syn::File) -> HashMap<String, ReturnType> {
    let mut function_map = HashMap::new();

    for item in &ast.items {
        if let Item::Fn(func) = item {
            function_map.insert(func.sig.ident.to_string(), func.sig.output.clone());
        }
    }

    function_map
}

fn is_last_expr_unit(expr: &Expr, function_map: &HashMap<String, ReturnType>) -> bool {
    match expr {
        Expr::ForLoop(_) => true,
        Expr::While(_) => true,
        Expr::Loop(_) => true,
        Expr::If(expr_if) => expr_if.else_branch.is_none(),
        Expr::Block(expr_block) => {
            if let Some(last_stmt) = expr_block.block.stmts.last() {
                match last_stmt {
                    Stmt::Expr(_, None) => false,   // Expression without semicolon
                    Stmt::Expr(_, Some(_)) => true, // Expression with semicolon returns unit
                    Stmt::Macro(m) => m.semi_token.is_some(), // Macro with a semicolon returns unit
                    _ => false,
                }
            } else {
                false
            }
        }
        Expr::Call(expr_call) => {
            if let Expr::Path(path) = &*expr_call.func {
                if let Some(ident) = path.path.get_ident() {
                    if let Some(return_type) = function_map.get(&ident.to_string()) {
                        return match return_type {
                            ReturnType::Default => true,
                            ReturnType::Type(_, ty) => {
                                if let syn::Type::Tuple(tuple) = &**ty {
                                    tuple.elems.is_empty()
                                } else {
                                    false
                                }
                            }
                        };
                    }
                }
            }
            false
        }
        Expr::Closure(expr_closure) => match &expr_closure.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ty) => {
                if let syn::Type::Tuple(tuple) = &**ty {
                    tuple.elems.is_empty()
                } else {
                    false
                }
            }
        },
        _ => false,
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
    println!("{{:?}}", {{
        {expr}
    }});
}}
"#,
        expr = expr.to_token_stream()
    )
}
