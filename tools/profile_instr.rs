use ra_ap_syntax::{
    ast::{self, HasName},
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
        // eprintln!("has_toml={has_toml}");
        if has_toml {
            let next_token = first_use.next_sibling_or_token();
            // eprintln!("first_use.next_sibling_or_token()={next_token:?}");
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
            // eprintln!("Returning `(Position::before({first_item:?}), true)`");
            (Position::before(first_item), true)
        } else {
            // eprintln!(
            //     "Returning `(Position::last_child_of({:?}), true)`",
            //     tree.syntax()
            // );
            (Position::last_child_of(tree.syntax()), true)
        }
    }
}

fn instrument_code(source: &str) -> String {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    let imports = [
        "use thag_rs::profiling;",
        "use thag_proc_macros::enable_profiling;",
    ];
    for import_text in imports.iter() {
        if !source.contains(import_text) {
            if let Some(import_node) = parse_attr(import_text) {
                let (pos, _) = find_best_import_position(&tree);
                ted::insert(pos, &import_node);
            }
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
                // Get original indentation.
                // Previous whitespace will include all prior newlines.
                // If there are any, we only want the last one, otherwise we will get
                // as many newlines as there are prior newlines.
                let indent = function
                    .syntax()
                    .prev_sibling_or_token()
                    .and_then(|t| {
                        // eprintln!("t: {t:?}");
                        if t.kind() == SyntaxKind::WHITESPACE {
                            let s = t.to_string();
                            let new_indent = s
                                .rmatch_indices('\n')
                                .next()
                                .map_or(s.clone(), |(i, _)| (&s[i..]).to_string());
                            // eprintln!("new_indent: [{new_indent}]");
                            Some(new_indent)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                // eprintln!("Indentation: {}, value: [{}]", indent.len(), indent);
                // Parse and insert attribute with proper indentation
                let attr_node = parse_attr(&format!("{}{}", indent, attr_text))
                    .expect("Failed to parse attribute");
                ted::insert(Position::before(function.syntax()), &attr_node);

                // Add single newline with same indentation
                let ws_token = ast::make::tokens::whitespace(&indent);
                // ted::insert(Position::after(&attr_node), ws_token);
                ted::insert(Position::before(function.syntax()), ws_token);
            }
        }
    }

    // eprintln!("tree={tree:#?}");
    // Return the result without trimming, to preserve original file start
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
