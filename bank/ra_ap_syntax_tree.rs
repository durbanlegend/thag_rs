/*[toml]
[dependencies]
ra_ap_syntax = "0.0.261"
*/

use ra_ap_syntax::{AstNode, Edition, SourceFile};
use std::io::Read;

fn read_stdin() -> std::io::Result<String> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin()?;
    let parse = SourceFile::parse(&content, Edition::Edition2021);
    let tree = parse.tree().clone_for_update();

    eprintln!("tree={tree:#?}");
    Ok(())
}
