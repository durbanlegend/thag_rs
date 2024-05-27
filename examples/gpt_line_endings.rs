/*[toml]
[dependencies]
regex = "1.10.4"
*/

use regex::Regex;

fn normalize_newlines(input: &str) -> String {
    let re = Regex::new(r"\r\n?").unwrap();
    re.replace_all(input, "\n").to_string()
}

fn main() {
    let input = "Some text\r\nwith different\rline endings.";
    let normalized = normalize_newlines(&input);
    println!("{}", normalized);
}
