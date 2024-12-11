/*[toml]
[dependencies]
edit = "0.1.5"
#log = "0.4.22"
regex = "1.10.5"
syn = "2"
# thag_rs = "0.1.7"
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", rev = "ad07d461c3b20c837d901adeb7b46371bf79646f" }
# thag_rs = { path = "/Users/donf/projects/thag_rs" }
# tokio = "1.41.1"
tokio = { version = "1", features = ["full"] }
warp = "0.3.7"
*/

/// Select demo scripts and generate and serve HTML report.
///
/// Strategy and grunt work thanks to ChatGPT.
//# Purpose: Allow user to select scripts by category.
//# Categories: technique, tools
use edit;
// use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
};
use thag_rs::code_utils;
use tokio;
use warp::Filter;

#[tokio::main]
async fn main() {
    let scripts_dir = Path::new("demo");
    let categories = std::env::args().skip(1).collect::<Vec<_>>(); // Command-line args as categories
    if categories.is_empty() {
        eprintln!("Please specify at least one category.");
        return;
    }

    // Collect metadata and filter by categories
    let metadata = collect_all_metadata(scripts_dir)
        .into_iter()
        .filter(|meta| meta.categories.iter().any(|cat| categories.contains(cat)))
        .collect::<Vec<_>>();
    // eprintln!("metadata={metadata:#?}");

    // Generate HTML report
    let html_report = generate_html_report(&categories.join(", "), &metadata);

    // Serve via HTTPS
    let edit_route =
        warp::path("edit")
            .and(warp::path::param::<String>())
            .map(move |script_name: String| {
                let script_path = Path::new(&scripts_dir).join(&script_name);
                if edit::edit_file(&script_path).is_ok() {
                    format!("Editing script: {}", script_name)
                } else {
                    format!("Failed to edit script: {}", script_name)
                }
            });

    let html_route = warp::path::end().map(move || warp::reply::html(html_report.clone()));
    let routes = html_route.or(edit_route);

    println!("Serving web page on http://127.0.0.1:8081");
    // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
}

// Function to generate HTML content
fn generate_html_report(categories_str: &str, metadata_list: &[ScriptMetadata]) -> String {
    let mut html = String::from("<html><head><title>Demo Scripts</title></head><body>");
    html.push_str(&format!(
        "<h1>thag_rs demo scripts matching the categories: {categories_str}</h1><ul>"
    ));

    for meta in metadata_list {
        html.push_str(&format!(
            "<li><h2>{}</h2><p>{}</p><p><strong>Purpose:</strong> {}</p>",
            meta.script,
            meta.description
                .as_ref()
                .unwrap_or(&String::from("No description available")),
            meta.purpose
                .as_ref()
                .unwrap_or(&String::from("No purpose specified")),
        ));

        if !meta.crates.is_empty() {
            html.push_str(&format!(
                "<p><strong>Crates:</strong> {}</p>",
                meta.crates.join(", ")
            ));
        }

        html.push_str(&format!(
            "<p><strong>Categories:</strong> {}</p>",
            meta.categories.join(", ")
        ));

        html.push_str(&format!(
            "<p><a href=\"/edit/{}\">Edit script</a></p></li>",
            meta.script
        ));
    }

    html.push_str("</ul></body></html>");
    html
}

// Collect metadata logic here (reuse from `gen_readme.rs` with filtering changes)

#[derive(Debug)]
#[allow(dead_code)]
struct ScriptMetadata {
    script: String,
    purpose: Option<String>,
    crates: Vec<String>,
    // script_type: Option<String>,
    description: Option<String>,
    categories: Vec<String>, // New field for categories
}

fn parse_metadata(file_path: &Path) -> Option<ScriptMetadata> {
    let mut content = fs::read_to_string(file_path).ok()?;

    content = if content.starts_with("#!") && !(content.starts_with("#![")) {
        let split_once = content.split_once('\n');
        let (_shebang, rust_code) = split_once.expect("Failed to strip shebang");
        // eprintln!(
        //     "Successfully stripped shebang {shebang} from {}",
        //     file_path.display()
        // );
        rust_code.to_string()
    } else {
        content
    };

    let mut metadata = HashMap::new();
    let mut lines = Vec::<String>::new();
    let mut doc = false;
    let mut purpose = false;
    let mut categories = vec!["missing".to_string()]; // Default to "general"

    for line in content.clone().lines() {
        if line.starts_with("//#") {
            let parts: Vec<&str> = line[3..].splitn(2, ':').collect();
            if parts.len() == 2 {
                let keyword = parts[0].trim();
                let value = parts[1].trim().to_string();
                match keyword.to_lowercase().as_str() {
                    "purpose" => {
                        metadata.insert("purpose".to_string(), value);
                        purpose = true;
                    }
                    "categories" => {
                        categories = value.split(',').map(|cat| cat.trim().to_string()).collect();
                    }
                    _ => {}
                }
            }
        } else if line.starts_with("///") || line.starts_with("//:") {
            lines.push(line[3..].to_string() + "\n");
            if !doc {
                doc = true;
            }
        }
    }

    let file_path_str = &file_path.to_string_lossy();

    if !doc || !purpose {
        if !doc {
            println!("{file_path_str} has no docs");
        }
        if !purpose {
            println!("{file_path_str} has no purpose");
        }
    }

    if doc {
        metadata.insert("description".to_string(), lines.join(""));
    }

    let maybe_syntax_tree = code_utils::to_ast(file_path_str, &content);

    let crates = match maybe_syntax_tree {
        Some(ref ast) => code_utils::infer_deps_from_ast(&ast),
        None => code_utils::infer_deps_from_source(&content),
    };

    let script = format!(
        "{}",
        file_path
            .file_name()
            .expect("Error accessing filename")
            .to_string_lossy()
    );

    // eprintln!(
    //     "{script} maybe_syntax_tree.is_some(): {}",
    //     maybe_syntax_tree.is_some()
    // );

    let purpose = metadata.get("purpose");
    let description = metadata.get("description");

    Some(ScriptMetadata {
        script,
        purpose: purpose.cloned(),
        crates,
        // script_type: Some(script_type.to_string()),
        description: description.cloned(),
        categories, // Add categories to metadata
    })
}

fn collect_all_metadata(scripts_dir: &Path) -> Vec<ScriptMetadata> {
    let mut all_metadata = Vec::new();

    let scripts = read_dir(scripts_dir).expect("Error reading scripts");
    let mut scripts = scripts
        .flatten()
        .map(|dir_entry| dir_entry.path())
        .collect::<Vec<PathBuf>>();

    scripts.sort();

    for entry in scripts.iter() {
        let path = entry.as_path();
        // println!("Parsing {:#?}", path.display());

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Some(metadata) = parse_metadata(&path) {
                all_metadata.push(metadata);
            }
        }
    }

    all_metadata.sort_by(|a, b| a.script.partial_cmp(&b.script).unwrap());

    all_metadata
}
