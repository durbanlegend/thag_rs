use ra_ap_syntax::{
    ast::{self, HasModuleItem, HasName, HasVisibility, Item},
    ted::{self, Position},
    AstNode, Edition, Parse, SourceFile, SyntaxKind, SyntaxNode,
};
use std::io::Read;

/// A stand-alone convenience tool to instrument a Rust source program for `thag_rs` profiling.
/// It accepts the source code on stdin and outputs instrumented code to stdout.
/// The instrumentation consists of adding the #[enable_profiling] attribute to `fn main` if
/// present, and the #[profile] attribute to all other functions and methods, as well as import
/// statements for the `thag_rs` profiling.
/// module and proc macro library. It is intended to be lossless, using the `rust-analyzer` crate
/// to preserve the original source code intact with its comments and formatting. However, by using
/// it you accept responsibility for all consequences of instrumentation and profiling.
/// It's recommended to use profiling only in development environments and thoroughly test the
/// instrumented code before deploying it.
/// It's also recommended to do a side-by-side comparison of the original and instrumented code
/// to ensure that the instrumentation did not introduce any unintended changes.
/// Free tools for this purpose include `diff`, `sdiff` git diff, GitHub desktop and BBEdit.

/// This tool attempts to position the injected code sensibly and to avoid duplication of existing
/// `thag_rs` profiling code. It implements default profiling which currently includes both execution
/// time and memory usage, but this is easily tweaked manually by modifying the instrumented code by
/// adding the keyword `profile_type = ["time" | "memory"])` to the `#[enable_profiling]` attribute,
/// e.g.: `#[enable_profiling(profile_type = "time")]`.
///
/// This tool is intended for use with the `thag_rs` command-line tool or compiled into a binary.
/// Run it with the `-qq` flag to suppress unwanted output.
///
/// E.g.
///
/// 1. As a script:
///
/// ```
/// thag tools/profile_instr.rs -qq < demo/colors.rs > demo/colors_instrumented.rs
/// ```
///
/// 2. As a command (compiled with `thag tools/profile_instr.rs -x`)
///
/// ```
/// profile_instr < demo/colors.rs > demo/colors_instrumented.rs
/// ```
///
//# Purpose: Stand-alone tool to instrument any Rust source code for `thag` profiling.
//# Categories: profiling, tools
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin()?;
    let instrumented = instrument_code(&content);
    print!("{}", instrumented);
    Ok(())
}

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
            !item.syntax().children_with_tokens().any(|token| {
                token.kind() == SyntaxKind::COMMENT && token.to_string().starts_with("/*[toml]")
            })
        })
        .take(1)
        .next();
    // eprintln!("item={item:#?}");
    (
        Position::before(item.expect("Could not unwrap item").syntax()),
        true,
    )
}

fn instrument_code(source: &str) -> String {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    let imports = ["use thag_rs::{enable_profiling, profile, profiling, Profile};"];

    for import_text in imports.iter() {
        if !source.contains(import_text) {
            if let Some(import_node) = parse_attr(import_text) {
                let (pos, insert_nl) = find_best_import_position(&tree);
                // eprintln!(
                //     "insert_nl={}, pos={pos:?}, import_text={import_text}",
                //     insert_nl
                // );
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

            let fn_token = function.fn_token().expect("Function token is None");
            let maybe_visibility = function.visibility();
            let maybe_async_token = function.async_token();
            let target_token = if let Some(visibility) = maybe_visibility {
                if let Some(pub_token) = visibility.pub_token() {
                    pub_token
                } else if let Some(async_token) = maybe_async_token {
                    async_token
                } else {
                    fn_token
                }
            } else if let Some(async_token) = maybe_async_token {
                async_token
            } else {
                fn_token
            };
            // eprintln!("target_token: {target_token:?}");
            let function_syntax: &SyntaxNode = function.syntax();
            if function.body().is_some()
                && !function_syntax.descendants_with_tokens().any(|it| {
                    let text = it.to_string();
                    text.starts_with("#[profile")
                        || text.starts_with("#[enable_profiling")
                        || text.starts_with("profile")
                        || text.starts_with("enable_profiling")
                })
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
                ted::insert(Position::before(&target_token), &attr_node);

                // Add single newline with same indentation
                let ws_token = ast::make::tokens::whitespace(&indent);
                // ted::insert(Position::after(&attr_node), ws_token);
                ted::insert(Position::before(target_token), ws_token);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_whitespace(expected: &str, actual: &str) -> bool {
        let expected_bytes: Vec<u8> = expected.bytes().collect();
        let actual_bytes: Vec<u8> = actual.bytes().collect();

        if expected_bytes != actual_bytes {
            println!("Expected bytes: {:?}", expected_bytes);
            println!("Actual bytes:   {:?}", actual_bytes);
            println!(
                "Expected str chunks: {:?}",
                expected.split("").collect::<Vec<_>>()
            );
            println!(
                "Actual str chunks:   {:?}",
                actual.split("").collect::<Vec<_>>()
            );
            false
        } else {
            true
        }
    }

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
        let input = "fn foo() {}";
        let output = instrument_code(input);
        let expected = "use thag_rs::profiling; \nuse thag_proc_macros::enable_profiling; \n\n#[profile] \nfn foo() {}";
        assert!(
            compare_whitespace(expected, &output),
            "Whitespace mismatch between expected and actual output",
        );
        assert!(output.contains("#[profile] \nfn foo()"));
    }

    #[test]
    fn test_main_function_special_handling() {
        let input = "fn main() {}";
        let output = instrument_code(input);
        assert!(output.contains("#[enable_profiling] \nfn main()"));
    }

    #[test]
    fn test_preserves_indentation() {
        let input = r#"
impl Foo {
    fn bar() {}
}"#;
        let output = instrument_code(input);
        assert!(output.contains("    #[profile] \n    fn bar()"));
    }

    #[test]
    fn test_multiple_attributes() {
        let input = r#"#[allow(dead_code)]
fn main() {}"#;
        let output = instrument_code(input);
        // eprintln!("output=[{output}]");
        let expected = "use thag_rs::profiling; \nuse thag_proc_macros::enable_profiling; \n\n#[allow(dead_code)]\n#[enable_profiling] \nfn main() {}";
        assert!(
            compare_whitespace(expected, &output),
            "Whitespace mismatch between expected and actual output",
        );
        assert!(output.contains("#[allow(dead_code)]\n#[enable_profiling] \nfn main()"));
    }

    #[test]
    fn test_nested_functions() {
        let input = r#"
fn outer() {
    fn inner() {}
}"#;
        let output = instrument_code(input);
        assert!(output.contains("#[profile] \nfn outer()"));
        assert!(output.contains("    #[profile] \n    fn inner()"));
    }

    #[test]
    fn test_impl_block_functions() {
        let input = r#"
impl Foo {
    fn method1(&self) {}
    fn method2(&self) {}
}"#;
        let output = instrument_code(input);
        assert!(output.contains("    #[profile] \n    fn method1"));
        assert!(output.contains("    #[profile] \n    fn method2"));
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
        assert!(output.contains("    #[profile] \n    fn required_method"));
    }

    #[test]
    fn test_async_functions() {
        let input = "async fn async_foo() {}";
        let output = instrument_code(input);
        assert!(output.contains("#[profile] \nasync fn async_foo()"));
    }

    #[test]
    fn test_generic_functions() {
        let input = "fn generic<T: Display>(value: T) {}";
        let output = instrument_code(input);
        assert!(output.contains("#[profile] \nfn generic<T: Display>"));
    }

    #[test]
    fn test_doc_comments_preserved() {
        let input = r#"
/// Doc comment
fn documented() {}"#;
        let output = instrument_code(input);
        // eprintln!("{}", output);
        assert!(output.contains("/// Doc comment\n#[profile] \nfn documented()"));
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
        assert!(output.contains("}\n\n#[profile] \nfn bar()"));
    }
}
