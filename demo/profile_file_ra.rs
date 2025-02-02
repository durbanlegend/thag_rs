/*[toml]
[dependencies]
ra_ap_syntax = "0.0.261"
*/

use ra_ap_syntax::{
    ast::{self, make::tokens::single_newline, HasName},
    ted::{self, Position},
    AstNode, Edition,
    NodeOrToken::Token,
    Parse, SourceFile, SyntaxKind,
};
use std::io::Read;

fn parse_stmt(stmt: &str) -> Option<ra_ap_syntax::SyntaxNode> {
    let parse: Parse<ast::SourceFile> = SourceFile::parse(stmt, Edition::Edition2021);
    parse
        .tree()
        .syntax()
        .first_child()
        .map(|node| node.clone_for_update())
}

fn find_best_import_position(tree: &ast::SourceFile) -> (Position, bool) {
    // Look for the first USE node
    if let Some(first_use) = tree
        .syntax()
        .children()
        .find(|node| node.kind() == SyntaxKind::USE)
    {
        // Check if it contains a TOML block comment
        let has_toml = first_use.children_with_tokens().any(|token| {
            token.kind() == SyntaxKind::COMMENT && token.to_string().starts_with("/*[toml]")
        });
        eprintln!("has_toml={has_toml}");
        if has_toml {
            let next_token = first_use.next_sibling_or_token();
            eprintln!("first_use.next_sibling_or_token()={next_token:?}");
            let insert_nl = if let Some(Token(ref token)) = next_token {
                if token.kind() == SyntaxKind::WHITESPACE && token.to_string().starts_with("\n\n") {
                    false
                } else {
                    true
                }
            } else {
                true
            };
            // Insert after the entire USE node and add a blank line
            (Position::after(first_use), insert_nl)
        } else {
            // No TOML block, can insert before the USE node
            (Position::before(first_use), false)
        }
    } else {
        // No USE nodes, find first non-attribute, non-comment item
        if let Some(first_item) = tree.syntax().children().find(|node| {
            !matches!(
                node.kind(),
                SyntaxKind::ATTR | SyntaxKind::COMMENT | SyntaxKind::WHITESPACE
            )
        }) {
            (Position::before(first_item), true)
        } else {
            (Position::last_child_of(tree.syntax()), true)
        }
    }
}

fn insert_profile_in_method_body(body: &ast::BlockExpr, profile_stmt: &str) {
    if let Some(profile_node) = parse_stmt(profile_stmt) {
        // Find the STMT_LIST and its L_CURLY
        if let Some(stmt_list) = body
            .syntax()
            .children()
            .find(|n| n.kind() == SyntaxKind::STMT_LIST)
        {
            if let Some(l_curly) = stmt_list
                .children_with_tokens()
                .find(|t| t.kind() == SyntaxKind::L_CURLY)
            {
                // Insert after the opening brace and its following whitespace
                ted::insert(Position::after(&l_curly), profile_node);
                ted::insert(
                    Position::after(&l_curly),
                    ast::make::tokens::whitespace("    "),
                );
            }
        }
    }
}

fn instrument_code(source: &str) -> String {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    // eprintln!("tree={tree:#?}");

    // Add imports after attributes but before other items
    let import_text = "use thag_rs::{profile, profile_method};";
    if !source.contains(import_text) {
        if let Some(import_node) = parse_stmt(import_text) {
            let (pos, insert_nl) = find_best_import_position(&tree);
            if insert_nl {
                ted::insert(pos, single_newline())
            };
            let (pos, _insert_nl) = find_best_import_position(&tree);
            ted::insert(pos, import_node);
            let (pos, _insert_nl) = find_best_import_position(&tree);
            ted::insert(pos, single_newline());
        }
    }

    // Process standalone functions
    for node in tree.syntax().descendants() {
        if let Some(function) = ast::Fn::cast(node.clone()) {
            if let Some(body) = function.body() {
                if !has_profile_macro(&body) {
                    let fn_name = function
                        .name()
                        .map(|n| n.text().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    // Special handling for main
                    if fn_name == "main" {
                        let enable_prof = "let _ = thag_rs::profiling::enable_profiling(true);";
                        if !source.contains("enable_profiling(") {
                            if let Some(enable_node) = parse_stmt(enable_prof) {
                                if let Some(first_stmt) = body.statements().next() {
                                    ted::insert(Position::before(first_stmt.syntax()), enable_node);
                                    eprintln!(r#"Main: inserting "\n    " before first_stmt"#);
                                    ted::insert(
                                        Position::before(first_stmt.syntax()),
                                        ast::make::tokens::whitespace("\n    "),
                                    );
                                } else {
                                    eprintln!("Main: inserting NL before first_stmt");
                                    ted::insert(
                                        Position::first_child_of(body.syntax()),
                                        single_newline(),
                                    );
                                    ted::insert(
                                        Position::first_child_of(body.syntax()),
                                        enable_node,
                                    );
                                }
                            }
                        }
                    } else {
                        let profile_stmt = format!(r#"profile!("{fn_name}");"#);
                        if let Some(profile_node) = parse_stmt(&profile_stmt) {
                            if let Some(first_stmt) = body.statements().next() {
                                ted::insert(Position::before(first_stmt.syntax()), profile_node);
                                eprintln!(r#"Main: inserting "\n    " before first_stmt"#);
                                ted::insert(
                                    Position::before(first_stmt.syntax()),
                                    ast::make::tokens::whitespace("\n    "),
                                );
                            } else {
                                ted::insert(Position::first_child_of(body.syntax()), profile_node);
                                eprintln!("Inserting NL as first child");
                                ted::insert(
                                    Position::first_child_of(body.syntax()),
                                    single_newline(),
                                );
                            }
                        }
                    }
                }
            }
        }

        // Process impl methods
        if let Some(impl_block) = ast::Impl::cast(node) {
            let type_name = impl_block
                .self_ty()
                .map(|ty| ty.syntax().text().to_string())
                .unwrap_or_default();

            if let Some(items) = impl_block.assoc_item_list() {
                for item in items.assoc_items() {
                    if let ast::AssocItem::Fn(method) = item {
                        if let Some(body) = method.body() {
                            if !has_profile_macro(&body) {
                                let method_name = method
                                    .name()
                                    .map(|n| n.text().to_string())
                                    .unwrap_or_else(|| "unknown".to_string());

                                let profile_stmt =
                                    format!(r#"profile_method!("{type_name}::{method_name}");"#);
                                insert_profile_in_method_body(&body, &profile_stmt);
                            }
                        }
                    }
                }
            }
        }
    }

    // eprintln!("Updated tree.syntax():\n{:#?}", tree.syntax());
    tree.syntax().to_string()
}

fn has_profile_macro(body: &ast::BlockExpr) -> bool {
    body.syntax().descendants().any(|node| {
        ast::MacroCall::cast(node)
            .map(|call| {
                let name = call
                    .path()
                    .and_then(|p| p.segment())
                    .map(|s| s.name_ref().expect("Could not unwrap name_ref").to_string());
                matches!(name.as_deref(), Some("profile" | "profile_method"))
            })
            .unwrap_or(false)
    })
}

fn read_stdin() -> std::io::Result<String> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin()?;
    let instrumented = instrument_code(&content);
    print!("{}", instrumented);
    Ok(())
}
