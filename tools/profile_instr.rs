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
                let pos = find_best_import_position(&tree);
                ted::insert(pos, import_node);
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
                if let Some(attr_node) = parse_attr(attr_text) {
                    let indent = function.syntax().first_token().map_or("".to_string(), |t| {
                        t.prev_sibling_or_token()
                            .filter(|p| p.kind() == SyntaxKind::WHITESPACE)
                            .map(|p| p.to_string())
                            .unwrap_or("".to_string())
                    });
                    let formatted_attr = format!("{}{}\n", indent, attr_text);
                    if let Some(formatted_node) = parse_attr(&formatted_attr) {
                        ted::insert(Position::before(function.syntax()), formatted_node);
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
