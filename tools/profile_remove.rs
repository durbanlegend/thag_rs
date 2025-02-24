use ra_ap_syntax::{
    ast::{
        self, edit_in_place::Removable, HasModuleItem, HasName, Item, Use, UseTree, UseTreeList,
    },
    ted::{self, Element, Position},
    AstNode, Direction, Edition, Parse, SourceFile, SyntaxKind, SyntaxNode, SyntaxToken,
};
use std::io::Read;

/// A stand-alone convenience tool to remove `thag_rs` profiling instrumentation from a Rust source
/// program.
/// It accepts the instrumented source code on stdin and outputs uninstrumented code to stdout.
/// The process consists of removing any and all attribute macro and other ("legacy" / prototype)
/// macro invocations of `thag_rs` profiling. It is intended to be lossless, using the `rust-analyzer`
/// crate to preserve the original source code intact with its comments and formatting. However, by
/// using it you accept responsibility for all consequences.
/// It's recommended to use profiling only in development environments and thoroughly test or
/// remove the instrumented code before deploying it.
/// It's also recommended to do a side-by-side comparison of the original and de-instrumented code
/// to ensure that the removal of instrumentation did not introduce any unintended changes.
/// Free tools for this purpose include `diff`, `sdiff` git diff, GitHub desktop and BBEdit.
///
/// This tool is intended for use with the `thag_rs` command-line tool or compiled into a binary.
/// Run it with the `-qq` flag to suppress unwanted output.
///
/// E.g.
///
/// 1. As a script:
///
/// ```
/// thag tools/profile_uninstr.rs -qq < demo/colors_instrumented.rs > demo/colors.rs
/// ```
///
/// 2. As a command (compiled with `thag tools/profile_uninstr.rs -x`)
///
/// ```
/// profile_uninstr < demo/colors_instrumented.rs > demo/colors.rs
/// ```
///
//# Purpose: Stand-alone tool to remove any and all `thag_rs` profiling instrumentation from any Rust source code.
//# Categories: profiling, tools
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin()?;
    let stripped = deinstrument_code(&content);
    print!("{}", stripped);
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

fn deinstrument_code(source: &str) -> String {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    let import = "{enable_profiling, profile, profiling, Profile}";

    // Import statement must be in original format.
    let mut maybe_use_node = None::<Use>;
    let mut maybe_use_tree = None::<UseTree>;

    'outer: for node in tree.syntax().descendants() {
        // eprintln!("Node: {:?}", node);
        if let Some(use_node) = ast::Use::cast(node.clone()) {
            for child in use_node.syntax().children() {
                if let Some(use_tree_node) = ast::UseTree::cast(child.clone()) {
                    if let Some(path) = use_tree_node.path() {
                        eprintln!("Path: {:?}", path);
                        for segment in path.segments() {
                            eprintln!("Segment: {:?}", segment);
                            if let Some(name_ref) = segment.name_ref() {
                                let maybe_ident_token = name_ref.ident_token();
                                if let Some(ident_token) = maybe_ident_token {
                                    eprintln!("Ident Token: {:?}", ident_token);
                                    if ident_token.text().contains("thag_rs") {
                                        if let Some(use_tree_list) = use_tree_node.use_tree_list() {
                                            let var_name = use_tree_list.to_string();
                                            eprintln!("Use Tree List: {var_name}");
                                            if var_name.contains(import) {
                                                eprintln!("Found import: {}", import);
                                                maybe_use_node = Some(use_node);
                                                maybe_use_tree = Some(use_tree_node);
                                                break 'outer;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(use_node) = maybe_use_node {
        let maybe_prev_sibling_or_token = use_node.syntax().prev_sibling_or_token();
        if let Some(prev_sibling_or_token) = maybe_prev_sibling_or_token {
            if prev_sibling_or_token.kind() == SyntaxKind::WHITESPACE {
                eprintln!(
                    "Removing whitespace token with text range {:?}",
                    prev_sibling_or_token.text_range()
                );
                ted::remove(prev_sibling_or_token);
            }
        }
    }

    if let Some(use_tree) = maybe_use_tree {
        use_tree.remove_recursive();
    }

    let functions: Vec<_> = tree
        .syntax()
        .descendants()
        .filter_map(|node| ast::Fn::cast(node))
        .collect();

    for function in functions {
        // Do your modifications here
        let fn_name = function.name().map(|n| n.text().to_string());
        eprintln!("Function name: {:?}", fn_name.as_deref());
        let attr_text = if fn_name.as_deref() == Some("main") {
            "#[enable_profiling]"
        } else {
            "#[profile]"
        };
        let function_syntax: &SyntaxNode = function.syntax();
        for child in function_syntax.descendants_with_tokens() {
            if let Some(child_node) = child.as_node() {
                if let Some(attr) = ast::Attr::cast(child_node.clone()) {
                    let text = attr.to_string();
                    if text.starts_with(attr_text) {
                        if let Some(next_sibling_or_token) = attr.syntax().next_sibling_or_token() {
                            if next_sibling_or_token.kind() == SyntaxKind::WHITESPACE {
                                eprintln!(
                                    "Removing whitespace token with text range {:?}",
                                    next_sibling_or_token.text_range()
                                );
                                ted::remove(next_sibling_or_token);
                            }
                        }
                        eprintln!(
                            "Removing attribute with text range {:?}",
                            attr.syntax().text_range()
                        );
                        ted::remove(child_node);
                    }
                }
            }
        }

        if let Some(body) = function.body() {
            let statements = body.statements();
            let statements: Vec<_> = statements
                .map(|statement| statement.syntax().clone())
                .map(|stmt| stmt.clone())
                .filter_map(|stmt| ast::ExprStmt::cast(stmt.clone()))
                .filter(|stmt| {
                    stmt.syntax()
                        .descendants()
                        .find(|descendant| {
                            descendant.kind() == SyntaxKind::MACRO_CALL
                                && (descendant.text().to_string().starts_with("profile!")
                                    | descendant.text().to_string().starts_with("profile_fn!")
                                    || descendant.text().to_string().starts_with("profile_method!")
                                    || descendant
                                        .text()
                                        .to_string()
                                        .starts_with("profile_section!"))
                        })
                        .is_some()
                })
                .collect();

            eprintln!("statements={:?}", statements);
            for statement in statements {
                if let Some(prev_sibling_or_token) = statement.syntax().prev_sibling_or_token() {
                    if prev_sibling_or_token.kind() == SyntaxKind::WHITESPACE {
                        eprintln!(
                            "Removing whitespace token with text range {:?}",
                            prev_sibling_or_token.text_range()
                        );
                        ted::remove(prev_sibling_or_token);
                    }
                }
                ted::remove(statement.syntax().syntax_element());
            }
        }
    }

    // eprintln!("tree (after)={tree:#?}");
    // Return the result without trimming, to preserve original file start
    tree.syntax().to_string()
}

fn read_stdin() -> std::io::Result<String> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
