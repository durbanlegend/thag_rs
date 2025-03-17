/*[toml]
[dependencies]
ra_ap_syntax = "0.0.270"
*/

use ra_ap_syntax::{
    ast::{Fn, HasName, HasVisibility},
    AstNode, Edition, SourceFile,
};
use std::io::Read;

fn read_stdin() -> std::io::Result<String> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin()?;
    let parse = SourceFile::parse(&content, Edition::Edition2021);
    let file = parse.tree();

    for node in file.syntax().descendants() {
        if let Some(function) = Fn::cast(node.clone()) {
            // Don't profile a constant function
            if function.const_token().is_some() {
                continue;
            }

            let fn_name = function.name().map(|n| n.text().to_string());
            let Some("create") = fn_name.as_deref() else {
                continue;
            };

            let fn_token = function.fn_token().expect("Function token is None");
            let maybe_visibility = function.visibility();
            let maybe_async_token = function.async_token();
            let maybe_unsafe_token = function.unsafe_token();
            if let Some(ref unsafe_token) = maybe_unsafe_token {
                eprintln!("unsafe_token: {unsafe_token}");
            }
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
            eprintln!("fn_name={fn_name:?}, target_token: {target_token}");
        }
    }

    Ok(())
}
