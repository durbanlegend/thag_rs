/// Parse and display the `rust-analyzer` (not `syn`) format syntax tree of a Rust source file.
///
/// Assumes the input is a valid Rust program and that its Rust edition is 2021
///
//# Purpose: examine a `ra_ap_syntax` syntax tree.
//# Categories: AST, crates, technique
use ra_ap_syntax::ast::{HasModuleItem, Item};
use ra_ap_syntax::{Edition, SourceFile};
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

    println!("file={file:#?}");

    let _item = file
        .items()
        .filter(|item| !matches!(item, Item::Use(_)))
        .take(1)
        .next();
    // println!("item={item:#?}");

    Ok(())
}
