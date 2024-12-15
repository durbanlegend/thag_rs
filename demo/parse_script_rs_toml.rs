/// Prototype of extracting Cargo manifest metadata from source code using
/// basic line-by-line comparison as opposed to a regular expression. I eventually
/// decided to use a regular expression as I found it less problematic (see
/// `demo/regex_capture_toml.rs`).
//# Purpose: Prototype
//# Categories: prototype, technique
fn separate_rust_and_toml(source_code: &str) -> (String, String) {
    let mut rust_code = String::new();
    let mut toml_metadata = String::new();
    let mut is_metadata_block = false;

    for line in source_code.lines() {
        // Check if the line contains the start of the metadata block
        let ltrim = line.trim();
        // Using the dodge of interpolating the toml literal here so as not to
        // break the script runner when it parses the source code for /*[t0ml].
        if ltrim.starts_with(&format!("/*[{}]", "toml")) {
            is_metadata_block = true;
            continue;
        }

        // Check if the line contains the end of the metadata block
        if ltrim == "*/" {
            is_metadata_block = false;
            continue;
        }

        // Add the line to the appropriate string based on the metadata block status
        if is_metadata_block {
            toml_metadata.push_str(line);
            toml_metadata.push('\n');
        } else {
            rust_code.push_str(line);
            rust_code.push('\n');
        }
    }

    // Trim trailing whitespace from both strings
    rust_code = rust_code.trim().to_string();
    toml_metadata = toml_metadata.trim().to_string();

    (rust_code, toml_metadata)
}

fn main() {
    // Using the same interpolation dodge here.
    let source_code = format!(
        r##"// Some comments
/*[{}]
[dependencies]
syn = {{ version = "2.0.90", features = ["extra-traits"] }}
*/
// More comments or start of Rust code

// Rust code continues here
fn main() {{
    println!("Hello, world!");
}}
"##,
        "toml"
    );

    let (rust_code, toml_metadata) = separate_rust_and_toml(&source_code);

    println!("Rust code:\n{}", rust_code);
    println!("\nTOML metadata:\n{}", toml_metadata);
}
