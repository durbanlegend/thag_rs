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
                let pos = find_best_import_position(&tree);
                ted::insert(pos, &import_node);
                let newline = ast::make::tokens::single_newline();
                ted::insert(Position::after(&import_node), newline);
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
                let indent = function
                    .syntax()
                    .prev_sibling_or_token()
                    .and_then(|t| {
                        if t.kind() == SyntaxKind::WHITESPACE {
                            Some(t.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                let attr_node = parse_attr(&format!("{}{}", indent, attr_text))
                    .expect("Failed to parse attribute");
                ted::insert(Position::before(function.syntax()), &attr_node);

                let ws_token = ast::make::tokens::whitespace(&format!("\n{}", indent));
                ted::insert(Position::after(&attr_node), ws_token);
            }
        }
    }

    // Remove extra blank lines and normalize spacing
    let result = tree.syntax().to_string();
    result
        .trim_start()
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
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
