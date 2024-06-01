/*[toml]
[dependencies]
regex = "1.10.4"
*/
use regex::Regex;

fn extract_toml_block(input: &str) -> Option<String> {
    let re = Regex::new(r##"(?s)/\*\[alt\](.*?)\*/"##).unwrap();
    // eprintln!("{}", re.as_str());
    re.captures(input)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn main() {
    let input = r#"/*[alt]
[dependencies]
regex = "1.10.4"
*/"#;

    if let Some(toml_content) = extract_toml_block(input) {
        println!("{}", toml_content);
    } else {
        println!("No TOML block found.");
    }
}
