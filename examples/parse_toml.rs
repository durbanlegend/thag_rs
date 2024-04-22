fn extract_metadata(source_code: &str) -> Option<String> {
    let start_tag = "/*[toml]";
    let end_tag = "*/";

    // Find the start and end indices of the metadata block
    let start_index = source_code.find(start_tag)?;
    let end_index = source_code[start_index..]
        .find(end_tag)
        .map(|i| start_index + i + end_tag.len())?;

    // Extract the metadata block as a string
    let metadata = &source_code[start_index + start_tag.len()..end_index - end_tag.len()];

    Some(metadata.trim().to_string())
}

fn main() {
    let source_code = r#"
        // Some comments
        /*[toml]
        [dependencies]
        syn = { version = "2.0.60", features = ["extra-traits"] }
        */
        // More comments or start of Rust code
    "#;

    if let Some(metadata) = extract_metadata(source_code) {
        println!("Metadata block:\n{}", metadata);
    } else {
        println!("Metadata block not found");
    }
}
