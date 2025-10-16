/*[toml]
[dependencies]
regex = "1.10.5"
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
# thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features=["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features=["full_profiling"] }
*/

use regex::Regex;
use std::io::{self, BufRead};

use thag_profiler::{enable_profiling, end, profile, profiled};

#[profiled]
fn print_line(line: &str, re: &Regex) {
    println!(
        "{}",
        if let Some(m) = re.find(&line) {
            let num: i32 = m.as_str().parse().unwrap();
            let incremented = num + 10;
            let new_line = line.replace(m.as_str(), &incremented.to_string());
            new_line
        } else {
            line.to_string()
        }
    );
}

#[enable_profiling]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    profile!(init_regex);
    let re = Regex::new(r"\d+").unwrap();
    end!(init_regex);
    // Read from stdin and execute main loop for each line
    // let mut i = 0;
    profile!(stand_in);
    let stdin = io::stdin();
    end!(stand_in);
    profile!(loop_de_loop);
    for line in stdin.lock().lines() {
        let line = line?;
        print_line(&line, &re);
        // i += 1;
    }
    end!(loop_de_loop);
    profile!(farewell);
    println!("Processing complete");
    end!(farewell);
    Ok(())
}
