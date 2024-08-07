/*[toml]
[dependencies]
regex = "1.10.5"
*/
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use regex::Regex;
    let re = Regex::new(r"\d+").unwrap();
    // Read from stdin and execute main loop for each line
    let mut i = 0;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        i += 1;
        println!(
            "{}",
            if let Some(mat) = re.find(&line) {
                let num: i32 = mat.as_str().parse().unwrap();
                let incremented = num + 10;
                let new_line = line.replace(mat.as_str(), &incremented.to_string());
                new_line
            } else {
                line
            }
        );
    }
    println!("Processing complete");
    Ok(())
}
