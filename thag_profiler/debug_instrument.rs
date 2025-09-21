use ra_ap_syntax::{
    ast::{self, HasModuleItem, HasName, HasVisibility, Item},
    ted::{self, Position},
    AstNode, Edition, Parse, SourceFile, SyntaxKind, SyntaxNode,
};

fn parse_attr(attr: &str) -> Option<ra_ap_syntax::SyntaxNode> {
    let parse: Parse<ast::SourceFile> = SourceFile::parse(attr, Edition::Edition2021);
    parse
        .tree()
        .syntax()
        .first_child()
        .map(|node| node.clone_for_update())
}

fn debug_instrument(source: &str) {
    println!("=== INPUT ===");
    println!("{:?}", source);
    println!("{}", source);

    let parse = SourceFile::parse(source, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    for node in tree.syntax().descendants() {
        if let Some(function) = ast::Fn::cast(node.clone()) {
            if function.const_token().is_some() {
                continue;
            }

            let fn_name = function.name().map(|n| n.text().to_string());
            println!("Found function: {:?}", fn_name);

            let attr_text = "#[profiled]";
            let fn_token = function.fn_token().expect("Function token is None");

            // Get original indentation
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

            println!("Indent: {:?} (len: {})", indent, indent.len());

            if function.body().is_some() {
                // Parse and insert attribute with proper indentation
                let attr_with_indent = format!("{}{}", indent, attr_text);
                println!("Attribute to parse: {:?}", attr_with_indent);

                let attr_node = parse_attr(&attr_with_indent).expect("Failed to parse attribute");
                println!("Parsed attr_node: {:?}", attr_node);
                println!("Parsed attr_node text: {:?}", attr_node.to_string());

                ted::insert(Position::before(&fn_token), &attr_node);

                if indent.len() > 0 {
                    let ws_token = ast::make::tokens::whitespace(&indent);
                    println!("Whitespace token: {:?}", ws_token);
                    ted::insert(Position::before(&fn_token), ws_token);
                }
            }
            break; // Only process first function for debugging
        }
    }

    println!("=== OUTPUT ===");
    let result = tree.syntax().to_string();
    println!("{:?}", result);
    println!("{}", result);

    // Show byte representation
    println!("=== BYTES ===");
    let bytes: Vec<u8> = result.bytes().collect();
    println!("{:?}", bytes);
}

fn main() {
    println!("=== Test 1: Simple function ===");
    debug_instrument("fn foo() {}");

    println!("\n=== Test 2: Indented function ===");
    debug_instrument("impl Foo {\n    fn bar() {}\n}");

    println!("\n=== Test 3: Function with existing attribute ===");
    debug_instrument("#[allow(dead_code)]\nfn main() {}");
}
