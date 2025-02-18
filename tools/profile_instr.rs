use ra_ap_syntax::{
    ast::{self, make::tokens::single_newline, HasName},
    ted::{self, Position},
    AstNode, Edition,
    NodeOrToken::Token,
    Parse, SourceFile, SyntaxKind,
};
use std::io::Read;

fn parse_attr(attr: &str) -> Option<ra_ap_syntax::SyntaxNode> {
    let parse: Parse<ast::SourceFile> = SourceFile::parse(attr, Edition::Edition2021);
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

fn instrument_code(source: &str) -> String {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    let import_text_1 = "use thag_rs::profiling;";
    let import_text_2 = "use thag_proc_macros::enable_profiling;";
    if !source.contains(import_text_1) {
        if let Some((import_node_1, import_node_2)) =
            (parse_attr(import_text_1), parse_attr(import_text_2))
        {
            let (pos, insert_nl) = find_best_import_position(&tree);
            if insert_nl {
                ted::insert(pos, single_newline())
            };
            let (pos, _insert_nl) = find_best_import_position(&tree);
            ted::insert(pos, import_node_2);
            let (pos, _insert_nl) = find_best_import_position(&tree);
            ted::insert(pos, single_newline());
            let (pos, _insert_nl) = find_best_import_position(&tree);
            ted::insert(pos, import_node_1);
            let (pos, _insert_nl) = find_best_import_position(&tree);
            ted::insert(pos, single_newline());
        }
    }

    for node in tree.syntax().descendants() {
        if let Some(function) = ast::Fn::cast(node.clone()) {
            let fn_name = function.name().map(|n| n.text().to_string());
            let attr_text = if fn_name.as_deref() == Some("main") {
                "#[enable_profiling]"
            } else {
                "#[profile]"
            };

            if !function
                .syntax()
                .prev_sibling_or_token()
                .map_or(false, |t| t.to_string().starts_with("#["))
            {
                if let Some(attr_node) = parse_attr(attr_text) {
                    if let Some(whitespace) = function
                        .syntax()
                        .prev_sibling_or_token()
                        .filter(|t| t.kind() == SyntaxKind::WHITESPACE)
                    {
                        let indent = whitespace.to_string();
                        let formatted_attr = format!("{}{}", indent, attr_text);
                        if let Some(formatted_node) = parse_attr(&formatted_attr) {
                            ted::insert(Position::before(function.syntax()), formatted_node);
                            ted::insert(
                                Position::before(function.syntax()),
                                ast::make::tokens::whitespace("\n"),
                            );
                        }
                    } else {
                        ted::insert(Position::before(function.syntax()), attr_node);
                        ted::insert(
                            Position::before(function.syntax()),
                            ast::make::tokens::whitespace("\n"),
                        );
                    }
                }
            }
        }
    }

    tree.syntax().to_string()
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
