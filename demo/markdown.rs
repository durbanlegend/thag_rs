/*[toml]
[dependencies]
thag_common = { version = "1, thag-auto" }
*/

/// Quick basic markdown viewer using the `markdown` and `webbrowser` crates.
/// It makes no pretentions to resolving linked markdown files.
///
//# Purpose: Useful tool and demo.
//# Categories: crates, demo, tools
use std::{env, fs, process};
use thag_styling::{auto_help, help_system::check_help_and_exit, svprtln, Role, V};

fn main() {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    // Get the Markdown file path from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <markdown_file_path>", args[0]);
        process::exit(1);
    }
    let markdown_file_path = &args[1];

    // Read the Markdown file
    let markdown_content = match fs::read_to_string(markdown_file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file {markdown_file_path}: {err}");
            process::exit(1);
        }
    };

    let html_content = markdown::to_html(&markdown_content);

    let md_html = "thag_markdown.html";
    let result = std::fs::write(md_html, &html_content);

    if let Err(_e) = result {
        svprtln!(Role::ERR, V::N, "Error writing markdown to HTML file");

        std::process::exit(1);
    }

    let result = webbrowser::open(md_html);
    if let Err(_e) = result {
        svprtln!(Role::ERR, V::N, "Error opening HTML file in browser");

        std::process::exit(1);
    }
}
