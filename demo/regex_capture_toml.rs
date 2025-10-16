use regex::Regex;

/// Prototype of extracting Cargo manifest metadata from source code using
/// a regular expression. I ended up choosing this approach as being less
/// problematic than line-by-line parsing (see `demo/parse_script_rs_toml.rs`)
/// See also `demo/regex_capture_toml.rs`.
//# Purpose: Prototype, technique
//# Categories: prototype
fn extract_toml_block(input: &str) -> Option<String> {
    let re = Regex::new(r##"(?s)/\*\[toml\](.*?)\*/"##).unwrap();
    // eprintln!("{}", re.as_str());
    re.captures(input)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

// Using the dodge of interpolating the toml literal here so as to prevent a
// false positive when `thag` parses the source code looking for the toml block.
fn main() {
    let input = format!(
        r#"/*[{}]
[dependencies]
puffin = "0.19.0"
*/"#,
        "toml"
    );

    if let Some(toml_content) = extract_toml_block(&input) {
        println!("{}", toml_content);
    } else {
        println!("No TOML block found.");
    }
}
