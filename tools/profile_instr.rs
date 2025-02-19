use ra_ap_syntax::{
    ast::{self, HasModuleItem, HasName, Item},
    ted::{self, Position},
    AstNode,
    Edition,
    // NodeOrToken::Token,
    Parse,
    SourceFile,
    SyntaxKind,
    SyntaxNode,
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
    // Look for the first non-USE node
    let item = tree
        .items()
        .filter(|item| !matches!(item, Item::Use(_)))
        .filter(|item| {
            !item
                // .expect("REASON")
                .syntax()
                .children_with_tokens()
                .any(|token| {
                    token.kind() == SyntaxKind::COMMENT && token.to_string().starts_with("/*[toml]")
                })
        })
        .take(1)
        .next();
    eprintln!("item={item:#?}");
    (
        Position::before(item.expect("Could not unwrap item").syntax()),
        true,
    )
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
                let (pos, insert_nl) = find_best_import_position(&tree);
                eprintln!(
                    "insert_nl={}, pos={pos:?}, import_text={import_text}",
                    insert_nl
                );
                ted::insert(pos, &import_node);
                if insert_nl {
                    let newline = ast::make::tokens::single_newline();
                    let (pos, _) = find_best_import_position(&tree);
                    ted::insert(pos, newline);
                }
            }
        }
    }
    let newline = ast::make::tokens::single_newline();
    let (pos, _) = find_best_import_position(&tree);
    ted::insert(pos, newline);

    for node in tree.syntax().descendants() {
        if let Some(function) = ast::Fn::cast(node.clone()) {
            let fn_name = function.name().map(|n| n.text().to_string());
            let attr_text = if fn_name.as_deref() == Some("main") {
                "#[enable_profiling]"
            } else {
                "#[profile]"
            };

            let function_syntax: &SyntaxNode = function.syntax();
            let fn_token = function.fn_token().expect("Function token is None");
            eprintln!("fn_token: {fn_token:?}");
            if !function_syntax.descendants_with_tokens().any(|it| {
                let text = it.to_string();
                text.starts_with("#[profile")
                    || text.starts_with("#[enable_profiling")
                    || text.starts_with("profile")
                    || text.starts_with("enable_profiling")
            }) {
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
                ted::insert(Position::before(&fn_token), &attr_node);

                // Add single newline with same indentation
                let ws_token = ast::make::tokens::whitespace(&indent);
                // ted::insert(Position::after(&attr_node), ws_token);
                ted::insert(Position::before(fn_token), ws_token);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicate_imports() {
        let input = r#"
use some_crate::something;
use thag_proc_macros::enable_profiling;

fn foo() {}"#;
        let output = instrument_code(input);
        assert_eq!(
            output
                .matches("use thag_proc_macros::enable_profiling")
                .count(),
            1,
            "should not duplicate existing import"
        );
    }

    #[test]
    fn test_basic_function_instrumentation() {
        let input = "\n\nfn foo() {}";
        let output = instrument_code(input);
        eprintln!("output=[{output}]");
        assert!(output.contains("#[profile]\nfn foo()"));
    }

    #[test]
    fn test_main_function_special_handling() {
        let input = "fn main() {}";
        let output = instrument_code(input);
        assert!(output.contains("#[enable_profiling]\nfn main()"));
    }

    #[test]
    fn test_preserves_indentation() {
        let input = r#"
impl Foo {
    fn bar() {}
}"#;
        let output = instrument_code(input);
        assert!(output.contains("    #[profile]\n    fn bar()"));
    }

    #[test]
    fn test_multiple_attributes() {
        let input = r#"#[allow(dead_code)]
fn main() {}"#;
        let output = instrument_code(input);
        assert!(output.contains("#[enable_profiling]\n#[allow(dead_code)]\nfn main()"));
    }

    #[test]
    fn test_nested_functions() {
        let input = r#"
fn outer() {
    fn inner() {}
}"#;
        let output = instrument_code(input);
        assert!(output.contains("#[profile]\nfn outer()"));
        assert!(output.contains("    #[profile]\n    fn inner()"));
    }

    #[test]
    fn test_impl_block_functions() {
        let input = r#"
impl Foo {
    fn method1(&self) {}
    fn method2(&self) {}
}"#;
        let output = instrument_code(input);
        assert!(output.contains("    #[profile]\n    fn method1"));
        assert!(output.contains("    #[profile]\n    fn method2"));
    }

    #[test]
    fn test_preserves_file_start() {
        let input = "// Copyright notice\n\nfn foo() {}";
        let output = instrument_code(input);
        assert!(output.starts_with("// Copyright notice\n"));
    }

    #[test]
    fn test_trait_impl_functions() {
        let input = r#"
impl SomeTrait for Foo {
    fn required_method(&self) {}
}"#;
        let output = instrument_code(input);
        assert!(output.contains("    #[profile]\n    fn required_method"));
    }

    #[test]
    fn test_async_functions() {
        let input = "async fn async_foo() {}";
        let output = instrument_code(input);
        assert!(output.contains("#[profile]\nasync fn async_foo()"));
    }

    #[test]
    fn test_generic_functions() {
        let input = "fn generic<T: Display>(value: T) {}";
        let output = instrument_code(input);
        assert!(output.contains("#[profile]\nfn generic<T: Display>"));
    }

    #[test]
    fn test_doc_comments_preserved() {
        let input = r#"
/// Doc comment
fn documented() {}"#;
        let output = instrument_code(input);
        assert!(output.contains("/// Doc comment\n#[profile]\nfn documented()"));
    }

    #[test]
    fn test_complex_spacing() {
        let input = r#"
use std::fmt;

// Some comment
fn foo() {}

fn bar() {}"#;
        let output = instrument_code(input);
        // Check that blank lines between functions are preserved
        assert!(output.contains("}\n\n#[profile]\nfn bar()"));
    }
}
