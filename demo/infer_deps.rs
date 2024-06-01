use std::fs;
use std::io::{self, BufRead};

fn main() {
    // Replace "your_source_file.rs" with the path to your Rust source file
    if let Ok(file) = fs::File::open("demo/has_main.rs") {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(trimmed_line) = line.split("//").next() {
                    if let Some(use_index) = trimmed_line.find("use ") {
                        // Check if "use" is not within a string literal
                        if !is_inside_string_literal(&trimmed_line[..use_index]) {
                            println!("Found use statement: {}", trimmed_line.trim());
                        }
                    }
                }
            }
        }
    } else {
        println!("Failed to open file");
    }
}

fn is_inside_string_literal(line: &str) -> bool {
    let mut in_string_literal = false;
    let mut prev_char: char = ' ';

    for c in line.chars() {
        if c == '"' && prev_char != '\\' {
            in_string_literal = !in_string_literal;
        }
        prev_char = c;
    }

    in_string_literal
}
