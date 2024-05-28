/*[toml]
[dependencies]
lazy_static = "1.4.0"
regex = "1.10.4"
*/

use lazy_static::lazy_static;
use regex::Regex;

fn normalize_newlines(input: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\r\n?").unwrap();
    }
    RE.replace_all(input, "\n").to_string()
}

fn main() {
    let input = "Some text\r\nwith different\rline endings.";
    let normalized = normalize_newlines(&input);
    println!("{}", normalized);
}
