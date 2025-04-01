use ra_ap_syntax::{
    ast::{self, HasAttrs, HasName, HasVisibility},
    ted::{self, Position},
    AstNode, Edition, Parse, SourceFile, SyntaxKind, SyntaxNode,
};
use std::env;
use std::io::Read;

/// A stand-alone convenience tool to instrument a Rust source program for `thag_profiler` profiling.
/// It accepts the source code on stdin and outputs instrumented code to stdout.
/// The instrumentation consists of adding the #[thag_profiler::enable_profiling] attribute to `fn main` if
/// present, and the #[thag_profiler::profiled] attribute to all functions and methods.
/// module and proc macro library. It is intended to be lossless, using the `rust-analyzer` crate
/// to preserve the original source code intact with its comments and formatting. However, by using
/// it you accept responsibility for all consequences of instrumentation and profiling.
/// It's recommended to use profiling only in development environments and thoroughly test the
/// instrumented code before deploying it.
/// It's also recommended to do a side-by-side comparison of the original and instrumented code
/// to ensure that the instrumentation did not introduce any unintended changes.
/// Free tools for this purpose include `diff`, `sdiff` git diff, GitHub desktop and BBEdit.

/// This tool attempts to position the injected code sensibly and to avoid duplication of existing
/// `thag_profiler` profiling code. It implements default profiling which currently includes both execution
/// time and memory usage, but this is easily tweaked manually by modifying the instrumented code by
/// adding the keyword `profile_type = ["time" | "memory"])` to the `#[enable_profiling]` attribute,
/// e.g.: `#[enable_profiling(profile_type = "time")]`.
///
/// This tool requires a single argument: a positive integer, being the Rust edition number of
/// the source code being instrumented (2015, 2018, 2021, 2024).
///
/// E.g.
///
/// ```
/// thag-instrument 2021 < demo/colors.rs > demo/colors_instrumented.rs
/// ```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1]
        .parse()
        .expect("Please provide a valid number in the set (2015, 2018, 2021, 2024).");

    let edition = match n {
        2015 => Edition::Edition2015,
        2018 => Edition::Edition2018,
        2021 => Edition::Edition2021,
        2024 => Edition::Edition2024,
        _ => panic!("nsupported or invalid Rust edition {n}"),
    };

    let content = read_stdin()?;
    let instrumented = instrument_code(edition, &content);
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

// fn find_best_import_position(tree: &ast::SourceFile) -> (Position, bool) {
//     // Look for the first non-USE node
//     let item = tree
//         .items()
//         .filter(|item| !matches!(item, Item::Use(_)))
//         .filter(|item| {
//             !item.syntax().children_with_tokens().any(|token| {
//                 token.kind() == SyntaxKind::COMMENT && token.to_string().starts_with("/*[toml]")
//             })
//         })
//         .take(1)
//         .next();
//     // eprintln!("item={item:#?}");
//     (
//         Position::before(item.expect("Could not unwrap item").syntax()),
//         true,
//     )
// }

fn instrument_code(edition: Edition, source: &str) -> String {
    let parse = SourceFile::parse(source, edition);
    let tree = parse.tree().clone_for_update();

    // let imports = ["use thag_profiler::*;"];

    // for import_text in imports.iter() {
    //     if !source.contains(import_text) {
    //         if let Some(import_node) = parse_attr(import_text) {
    //             let (pos, insert_nl) = find_best_import_position(&tree);
    //             // eprintln!(
    //             //     "insert_nl={}, pos={pos:?}, import_text={import_text}",
    //             //     insert_nl
    //             // );
    //             ted::insert(pos, &import_node);
    //             if insert_nl {
    //                 let newline = ast::make::tokens::single_newline();
    //                 let (pos, _) = find_best_import_position(&tree);
    //                 ted::insert(pos, newline);
    //             }
    //         }
    //     }
    // }
    // let newline = ast::make::tokens::single_newline();
    // let (pos, _) = find_best_import_position(&tree);
    // ted::insert(pos, newline);

    for node in tree.syntax().descendants() {
        if let Some(function) = ast::Fn::cast(node.clone()) {
            // Don't profile a constant function
            if function.const_token().is_some() {
                continue;
            }

            let fn_name = function.name().map(|n| n.text().to_string());
            eprintln!("fn_name={fn_name:?}");
            let attr_texts = if fn_name.as_deref() == Some("main") {
                vec![
                    "#[thag_profiler::profiled]",
                    "#[thag_profiler::enable_profiling]",
                ]
            } else {
                vec!["#[thag_profiler::profiled]"]
            };

            let fn_token = function.fn_token().expect("Function token is None");
            let maybe_visibility = function.visibility();
            let maybe_async_token = function.async_token();
            let maybe_unsafe_token = function.unsafe_token();
            let target_token = if let Some(visibility) = maybe_visibility {
                if let Some(pub_token) = visibility.pub_token() {
                    pub_token
                } else if let Some(async_token) = maybe_async_token {
                    async_token
                } else if let Some(unsafe_token) = maybe_unsafe_token {
                    unsafe_token
                } else {
                    fn_token
                }
            } else if let Some(async_token) = maybe_async_token {
                async_token
            } else if let Some(unsafe_token) = maybe_unsafe_token {
                unsafe_token
            } else {
                fn_token
            };
            // eprintln!(
            //     "target_token: {target_token:?}, function.body().is_some()? {}",
            //     function.body().is_some()
            // );
            let function_syntax: &SyntaxNode = function.syntax();
            if function.body().is_some()
                && !function_syntax.descendants_with_tokens().any(|it| {
                    let text = it.to_string();
                    let filtered_out = text.starts_with("#[profiled")
                        || text.starts_with("#[thag_profiler::profiled")
                        || text.starts_with("#[enable_profiling")
                        || text.starts_with("#[thag_profiler::enable_profiling")
                        || text.starts_with("#[test")
                        || text.starts_with("profile!")
                        || text.starts_with("enable_profiling");
                    filtered_out
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
                        if t.kind() == SyntaxKind::WHITESPACE {
                            let s = t.to_string();
                            let new_indent = s
                                .rmatch_indices('\n')
                                .next()
                                .map_or(s.clone(), |(i, _)| (&s[i..]).to_string());
                            Some(new_indent)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                
                // Get the collection of attributes
                let attrs = function.attrs().collect::<Vec<_>>();
                
                // Determine where to insert our attributes - we need to recreate this for each iteration
                let get_insert_pos = || {
                    if attrs.is_empty() {
                        // No existing attributes, use default position
                        Position::before(&target_token)
                    } else {
                        // Insert before the first attribute
                        Position::before(attrs[0].syntax())
                    }
                };
                
                // Reverse the order so they'll end up in the correct order when inserted
                for attr_text in attr_texts.iter().rev() {
                    // Get a fresh position for each insertion
                    let insert_pos = get_insert_pos();
                    
                    // Parse and insert attribute with proper indentation
                    let attr_node = parse_attr(&format!("{indent}{attr_text}"))
                        .expect("Failed to parse attribute");
                    ted::insert(insert_pos, &attr_node);

                    if indent.len() > 0 {
                        let insert_pos = get_insert_pos();
                        let ws_token = ast::make::tokens::whitespace(&indent);
                        ted::insert(insert_pos, ws_token);
                    }
                }
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

// Run with `cargo test --features="instrument-tool" --bin thag-instrument`
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
    fn test_instrument_no_duplicate_imports() {
        let input = r#"
use some_crate::something;
use thag_proc_macros::enable_profiling;

fn foo() {}"#;
        let output = instrument_code(Edition::Edition2021, input);
        assert_eq!(
            output
                .matches("use thag_proc_macros::enable_profiling")
                .count(),
            1,
            "should not duplicate existing import"
        );
    }

    #[test]
    fn test_instrument_basic_function_instrumentation() {
        let input = "fn foo() {}";
        let output = instrument_code(Edition::Edition2021, input);
        let expected = "use thag_profiler::*; \n\n#[profiled] \nfn foo() {}";
        assert!(
            compare_whitespace(expected, &output),
            "Whitespace mismatch between expected and actual output",
        );
        assert!(output.contains("#[profiled] \nfn foo()"));
    }

    #[test]
    fn test_instrument_main_function_special_handling() {
        let input = "fn main() {}";
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.contains("#[enable_profiling] \nfn main()"));
    }

    #[test]
    fn test_instrument_preserves_indentation() {
        let input = r#"
impl Foo {
    fn bar() {}
}"#;
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.contains("    #[profiled] \n    fn bar()"));
    }

    #[test]
    fn test_instrument_multiple_attributes() {
        let input = r#"#[allow(dead_code)]
fn main() {}"#;
        let output = instrument_code(Edition::Edition2021, input);
        // eprintln!("output=[{output}]");
        let expected =
            "use thag_profiler::*; \n\n#[allow(dead_code)]\n#[enable_profiling] \nfn main() {}";
        assert!(
            compare_whitespace(expected, &output),
            "Whitespace mismatch between expected and actual output",
        );
        assert!(output.contains("#[allow(dead_code)]\n#[enable_profiling] \nfn main()"));
    }

    #[test]
    fn test_instrument_nested_functions() {
        let input = r#"
fn outer() {
    fn inner() {}
}"#;
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.contains("#[profiled] \nfn outer()"));
        assert!(output.contains("    #[profiled] \n    fn inner()"));
    }

    #[test]
    fn test_instrument_impl_block_functions() {
        let input = r#"
impl Foo {
    fn method1(&self) {}
    fn method2(&self) {}
}"#;
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.contains("    #[profiled] \n    fn method1"));
        assert!(output.contains("    #[profiled] \n    fn method2"));
    }

    #[test]
    fn test_instrument_preserves_file_start() {
        let input = "// Copyright notice\n\nfn foo() {}";
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.starts_with("// Copyright notice\n"));
    }

    #[test]
    fn test_instrument_trait_impl_functions() {
        let input = r#"
impl SomeTrait for Foo {
    fn required_method(&self) {}
}"#;
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.contains("    #[profiled] \n    fn required_method"));
    }

    #[test]
    fn test_instrument_async_functions() {
        let input = "async fn async_foo() {}";
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.contains("#[profiled] \nasync fn async_foo()"));
    }

    #[test]
    fn test_instrument_generic_functions() {
        let input = "fn generic<T: Display>(value: T) {}";
        let output = instrument_code(Edition::Edition2021, input);
        assert!(output.contains("#[profiled] \nfn generic<T: Display>"));
    }

    #[test]
    fn test_instrument_doc_comments_preserved() {
        let input = r#"
/// Doc comment
fn documented() {}"#;
        let output = instrument_code(Edition::Edition2021, input);
        // eprintln!("{}", output);
        assert!(output.contains("/// Doc comment\n#[profiled] \nfn documented()"));
    }

    #[test]
    fn test_instrument_complex_spacing() {
        let input = r#"
use std::fmt;

// Some comment
fn foo() {}

fn bar() {}"#;
        let output = instrument_code(Edition::Edition2021, input);
        // Check that blank lines between functions are preserved
        assert!(output.contains("}\n\n#[profiled] \nfn bar()"));
    }
}
