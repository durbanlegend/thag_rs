use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process::Command;

use reqwest::blocking::Client;
use serde_json::json;

/// ChatGPT 4.1 Rust port of https://gist.github.com/joshuamiller/6d58f8bd239df56cabe8
fn main() {
    // Collect command-line arguments
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut clipboard_output = false;

    // Check for the "-c" flag to enable clipboard output
    if args.first().map_or(false, |arg| arg == "-c") {
        clipboard_output = true;
        args.remove(0);
    }

    // Determine input source: file or stdin
    let input = if let Some(filename) = args.first() {
        // Attempt to read from the specified file
        match File::open(filename) {
            Ok(mut file) => {
                let mut contents = String::new();
                if let Err(e) = file.read_to_string(&mut contents) {
                    eprintln!("Error reading file '{}': {}", filename, e);
                    std::process::exit(1);
                }
                contents
            }
            Err(e) => {
                eprintln!("File not found: '{}'. Error: {}", filename, e);
                std::process::exit(1);
            }
        }
    } else {
        // Read from stdin
        let mut contents = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut contents) {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
        if contents.trim().is_empty() {
            eprintln!("No input provided.");
            std::process::exit(1);
        }
        contents
    };

    // Prepare the JSON payload for the GitHub API
    let payload = json!({
        "text": input,
        "mode": "gfm"
    });

    // Send the POST request to GitHub's Markdown API
    let client = Client::new();
    let response = client
        .post("https://api.github.com/markdown")
        .header("User-Agent", "markdown-to-html")
        .json(&payload)
        .send();

    // Handle the response
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.text() {
                    Ok(body) => {
                        // Construct the final HTML output
                        let html_output = format!(
                            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Markdown Output</title>
    <style>
        body {{ font-family: Arial, sans-serif; padding: 2em; }}
        pre {{ background-color: #f6f8fa; padding: 1em; overflow: auto; }}
        code {{ background-color: #f6f8fa; padding: 0.2em 0.4em; }}
    </style>
</head>
<body>
<div id="wrapper">
{}
</div>
</body>
</html>"#,
                            body
                        );

                        // Output to clipboard or stdout
                        if clipboard_output {
                            // Attempt to copy to clipboard using 'pbcopy' (macOS)
                            let mut pbcopy = Command::new("pbcopy")
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                                .expect("Failed to start pbcopy");

                            if let Some(stdin) = pbcopy.stdin.as_mut() {
                                use std::io::Write;
                                if let Err(e) = stdin.write_all(html_output.as_bytes()) {
                                    eprintln!("Failed to write to pbcopy: {}", e);
                                    std::process::exit(1);
                                }
                            }

                            match pbcopy.wait() {
                                Ok(status) if status.success() => {
                                    println!("Result copied to clipboard.");
                                }
                                Ok(status) => {
                                    eprintln!("pbcopy exited with status: {}", status);
                                    std::process::exit(1);
                                }
                                Err(e) => {
                                    eprintln!("Failed to wait for pbcopy: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        } else {
                            // Print to stdout
                            println!("{}", html_output);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read response body: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("GitHub API returned error: {}", resp.status());
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to send request: {}", e);
            std::process::exit(1);
        }
    }
}
