/// Prototype of extracting Cargo manifest metadata from source code by locating
/// the start and end of the toml block. I eventually decided to use a regular
/// expression as I found it less problematic (see `demo/regex_capture_toml.rs`).
//# Purpose: Prototype
fn extract_metadata(source_code: &str) -> Option<String> {
    // Using the dodge of interpolating the toml literal here so as not to
    // break the script runner when it parses the source code for /*[t0ml].
    let start_tag = &format!("/*[{}]", "toml");
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
    // Using the same interpolation dodge here.
    let source_code = format!(
        r##"// Some comments
/*[{}]
[dependencies]
syn = {{ version = "2.0.82", features = ["extra-traits"] }}
*/
// More comments or start of Rust code
"##,
        "toml"
    );

    if let Some(metadata) = extract_metadata(&source_code) {
        println!("Metadata block:\n{}", metadata);
    } else {
        println!("Metadata block not found");
    }
}
