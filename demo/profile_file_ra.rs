/*[toml]
[dependencies]
ra_ap_syntax = "0.0.261"
*/

use ra_ap_syntax::{
    ast::{self, Fn, Impl, ImplItem},
    ted::{self, Position}, // For tree editing
    AstNode,
    SourceFile,
};
/// Tries to profile a file via injection into its abstract syntax tree.
//# Purpose: Debugging
//# Categories: AST, crates, profiling, technique, tools
use std::io::{self, Read};

fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn has_profile_macro(body: &ast::BlockExpr) -> bool {
    body.syntax().descendants().any(|node| {
        ast::MacroCall::cast(node)
            .map(|call| {
                let name = call
                    .path()
                    .and_then(|p| p.segment())
                    .map(|s| s.name_ref().to_string());
                matches!(name.as_deref(), Some("profile" | "profile_method"))
            })
            .unwrap_or(false)
    })
}

fn instrument_code(source: &str) -> String {
    let parse = SourceFile::parse(source);
    let tree = parse.tree();

    // First, add the imports if they don't exist
    let import_text = "use thag_rs::{profile, profile_method};\n";
    if !source.contains(import_text) {
        // Insert after any existing imports or at the start
        // This is simplified - you might want more sophisticated import handling
        ted::insert(Position::first_child_of(tree.syntax()), import_text);
    }

    // Process standalone functions
    for node in tree.syntax().descendants() {
        if let Some(function) = ast::Fn::cast(node.clone()) {
            if let Some(body) = function.body() {
                if !has_profile_macro(&body) {
                    let fn_name = function.name().map(|n| n.text()).unwrap_or_default();

                    // Special handling for main
                    if fn_name == "main" {
                        let enable_prof = "thag::profiling::enable_profiling(true).expect(\"Failed to enable profiling\");\n    ";
                        ted::insert(Position::first_child_of(body.syntax()), enable_prof);
                    }

                    let profile_stmt = format!("profile!(\"{fn_name}\");\n    ");
                    ted::insert(Position::first_child_of(body.syntax()), &profile_stmt);
                }
            }
        }

        // Process impl methods
        if let Some(impl_block) = ast::Impl::cast(node) {
            let type_name = impl_block
                .self_ty()
                .map(|ty| ty.syntax().text().to_string())
                .unwrap_or_default();

            for impl_item in impl_block.items() {
                if let ImplItem::Fn(method) = impl_item {
                    if let Some(body) = method.body() {
                        if !has_profile_macro(&body) {
                            let method_name = method.name().map(|n| n.text()).unwrap_or_default();
                            let profile_stmt =
                                format!("profile_method!(\"{type_name}::{method_name}\");\n    ");
                            ted::insert(Position::first_child_of(body.syntax()), &profile_stmt);
                        }
                    }
                }
            }
        }
    }

    tree.syntax().to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin()?;
    let instrumented = instrument_code(&content);
    print!("{}", instrumented);
    Ok(())
}
