use ra_ap_syntax::{
    ast::{self, HasName},
    ted::{self, Position},
    AstNode, Edition, Parse, SourceFile, SyntaxKind,
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

fn find_best_import_position(tree: &ast::SourceFile) -> Position {
    let mut last_use = None;
    for node in tree.syntax().children() {
        if node.kind() == SyntaxKind::USE {
            last_use = Some(node.clone());
        }
    }
    if let Some(last) = last_use {
        Position::after(last)
    } else {
        Position::first_child_of(tree.syntax())
    }
}

fn instrument_code(source: &str) -> String {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    let imports = [
        "use thag_proc_macros::enable_profiling;",
        "use thag_rs::profiling;",
    ];
    for import_text in imports.iter() {
        if !source.contains(import_text) {
            if let Some(import_node) = parse_attr(import_text) {
                let newline = ast::make::tokens::single_newline();
                let pos = find_best_import_position(&tree);
                ted::insert(pos, newline);
                let pos = find_best_import_position(&tree);
                ted::insert(pos, &import_node);
                // Single newline after each import
                // ted::insert(Position::after(&import_node), newline);
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
                        eprintln!("t: {t:?}");
                        if t.kind() == SyntaxKind::WHITESPACE {
                            let s = t.to_string();
                            let new_indent = s
                                .rmatch_indices('\n')
                                .next()
                                .map_or(s.clone(), |(i, _)| (&s[i..]).to_string());
                            eprintln!("new_indent: [{new_indent}]");
                            Some(new_indent)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                eprintln!("Indentation: {}, value: [{}]", indent.len(), indent);
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
