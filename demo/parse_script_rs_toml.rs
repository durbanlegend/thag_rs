fn separate_rust_and_toml(source_code: &str) -> (String, String) {
    let mut rust_code = String::new();
    let mut toml_metadata = String::new();
    let mut is_metadata_block = false;

    for line in source_code.lines() {
        // Check if the line contains the start of the metadata block
        if line.trim().starts_with("/*[toml]") {
            is_metadata_block = true;
            continue;
        }

        // Check if the line contains the end of the metadata block
        if line.trim() == "*/" {
            is_metadata_block = false;
            continue;
        }

        // Check if the line is a TOML comment
        if line.trim().starts_with("//!") {
            toml_metadata.push_str(line.trim_start_matches("//!"));
            toml_metadata.push('\n');
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
    let source_code = r#"
        // Some comments
        /*[toml]
        [dependencies]
        syn = { version = "2.0.60", features = ["extra-traits"] }
        */
        // More comments or start of Rust code

        // Rust code continues here
        fn main() {
            println!("Hello, world!");
        }
    "#;

    let (rust_code, toml_metadata) = separate_rust_and_toml(source_code);

    println!("Rust code:\n{}", rust_code);
    println!("TOML metadata:\n{}", toml_metadata);
}
