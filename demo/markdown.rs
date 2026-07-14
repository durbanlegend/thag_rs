/*[toml]
[dependencies]
thag_common = { version = "1, thag-auto" }
# warp = { version = "0.4", features = ["server"] }
*/

/// Quick markdown viewer.
///
//# Purpose: Useful tool and demo.
//# Categories: demo, tools
use std::env;
use std::fs;
use std::process;
use thag_common::{auto_help, help_system::check_help_and_exit};
use warp::Filter;

#[tokio::main]
async fn main() {
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
    let _ = std::fs::write(md_html, &html_content);
    webbrowser::open(md_html);

    // // Serve the HTML content on a local port
    // let html_filter = warp::any().map(move || warp::reply::html(html_content.clone()));
    // let port = 8080;
    // println!("Serving HTML on http://localhost:{port}");
    // warp::serve(html_filter).run(([127, 0, 0, 1], port)).await;
}
