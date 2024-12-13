/*[toml]
[dependencies]
edit = "0.1.5"
inquire = "0.7.5"
#log = "0.4.22"
regex = "1.10.5"
strum = { version = "0.26.3", features = ["derive"] }
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
use inquire::{MultiSelect, Select, Text};
use std::{
    collections::{BTreeSet, HashMap},
    env::current_dir,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    process::Command,
};
use thag_demo_proc_macros::category_enum;
use thag_rs::{code_utils, lazy_static_var, regex, shared};
use tokio;
use warp::Filter;

category_enum! {} // This will generate the Category enum

#[derive(Debug, Display)]
enum FilterLogic {
    Or,
    And,
}

#[derive(Debug)]
enum OutputFormat {
    Html,
    MarkdownPager,
    MarkdownFile,
}

fn get_user_preferences(
    available_crates: &BTreeSet<String>,
) -> (FilterLogic, Vec<Category>, Vec<String>, OutputFormat) {
    // First get the filter logic
    let logic = Select::new("Select filter logic:", vec!["OR", "AND"])
        .with_starting_cursor(0)
        .prompt()
        .map(|s| match s {
            "AND" => FilterLogic::And,
            _ => FilterLogic::Or,
        })
        .unwrap_or(FilterLogic::Or);

    // Then get category selections
    let categories = Category::iter().collect::<Vec<_>>();
    let category_selections = MultiSelect::new("Select categories:", categories)
        .prompt()
        .unwrap_or_default();

    // Then get crate selections
    let crate_selections = if !available_crates.is_empty() {
        MultiSelect::new(
            "Select crates to filter by:",
            available_crates.iter().cloned().collect::<Vec<_>>(),
        )
        .prompt()
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Finally get output format preference
    let format = Select::new("Select output format:", vec!["HTML", "Markdown"])
        .with_starting_cursor(0)
        .prompt()
        .map(|s| match s {
            "Markdown" => {
                // If Markdown selected, ask for output preference
                Select::new(
                    "How would you like to view the Markdown?",
                    vec!["Display in pager", "Save to file"],
                )
                .with_starting_cursor(0)
                .prompt()
                .map(|s| match s {
                    "Save to file" => OutputFormat::MarkdownFile,
                    _ => OutputFormat::MarkdownPager,
                })
                .unwrap_or(OutputFormat::MarkdownPager)
            }
            _ => OutputFormat::Html,
        })
        .unwrap_or(OutputFormat::Html);

    (logic, category_selections, crate_selections, format)
}

#[tokio::main]
async fn main() {
    let scripts_dir = Path::new("demo");

    // Collect all metadata first
    let all_metadata = collect_all_metadata(scripts_dir);

    // Build unique set of crates
    let available_crates: BTreeSet<String> = all_metadata
        .iter()
        .flat_map(|meta| meta.crates.clone())
        .collect();

    // Get user preferences including output format
    let (logic, categories, selected_crates, format) = get_user_preferences(&available_crates);

    // Convert categories to strings for filtering
    let category_strings: Vec<String> = categories
        .iter()
        .map(|c| c.to_string().to_lowercase())
        .collect();

    // Filter metadata based on both categories and crates
    let metadata = all_metadata
        .into_iter()
        .filter(|meta| match logic {
            FilterLogic::Or => {
                let matches_categories = category_strings.is_empty()
                    || meta
                        .categories
                        .iter()
                        .any(|cat| category_strings.contains(&cat.to_lowercase()));
                let matches_crates = selected_crates.is_empty()
                    || meta.crates.iter().any(|c| selected_crates.contains(c));
                matches_categories || matches_crates
            }
            FilterLogic::And => {
                let matches_categories = category_strings.is_empty()
                    || meta
                        .categories
                        .iter()
                        .any(|cat| category_strings.contains(&cat.to_lowercase()));
                let matches_crates = selected_crates.is_empty()
                    || selected_crates
                        .iter()
                        .all(|selected| meta.crates.contains(selected));
                matches_categories && matches_crates
            }
        })
        .collect::<Vec<_>>();

    // Create filter description for display
    let filter_description = {
        let mut desc = Vec::new();
        let logic_str = format!(" {logic} ");
        if !category_strings.is_empty() {
            desc.push(format!("categories: {}", category_strings.join(&logic_str)));
        }
        if !selected_crates.is_empty() {
            desc.push(format!("crates: {}", selected_crates.join(&logic_str)));
        }
        println!(
            "filter_description={}",
            desc.join(&format!(" {} ", &logic_str))
        );
        desc.join(&format!(" {} ", &logic_str))
    };

    // Handle different output formats
    match format {
        OutputFormat::MarkdownPager => {
            let markdown = output_markdown(&filter_description, &metadata);
            display_in_pager(&markdown);
        }
        OutputFormat::MarkdownFile => {
            let markdown = output_markdown(&filter_description, &metadata);
            let default_name = generate_default_filename(&categories, &selected_crates, &logic);
            match save_markdown_to_file(markdown, default_name) {
                Ok(path) => println!("Markdown file saved successfully to: {}", path.display()),
                Err(e) => eprintln!("Error saving file: {}", e),
            }
        }
        OutputFormat::Html => {
            let html_report = generate_html_report(&filter_description, &metadata);

            let edit_route = warp::path("edit").and(warp::path::param::<String>()).map(
                move |script_name: String| {
                    let script_path = Path::new(&scripts_dir).join(&script_name);
                    if edit::edit_file(&script_path).is_ok() {
                        format!("Editing script: {}", script_name)
                    } else {
                        format!("Failed to edit script: {}", script_name)
                    }
                },
            );

            let html_route = warp::path::end().map(move || warp::reply::html(html_report.clone()));
            let routes = html_route.or(edit_route);

            println!("Serving web page on http://127.0.0.1:8081");
            warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
        }
    }
}

// Function to generate HTML content
fn generate_html_report(categories_str: &str, metadata_list: &[ScriptMetadata]) -> String {
    let mut html = String::from(
        r#"
        <html>
        <head>
            <title>Demo Scripts</title>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/water.css">
            <style>
                body { max-width: 800px; margin: 0 auto; padding: 20px; }
                .script-item { margin-bottom: 2em; padding: 1em; border-radius: 5px; }
                .script-item:hover { background: #f5f5f5; }
                .metadata-label { font-weight: bold; color: #555; }
                .edit-link { display: inline-block; padding: 5px 15px;
                            background: #007bff; color: white;
                            text-decoration: none; border-radius: 3px; }
                .edit-link:hover { background: #0056b3; }
                .categories-header {
                    font-size: 1.5em;
                    margin: 20px 0;
                    padding: 10px;
                    background: #f8f9fa;
                    border-radius: 5px;
                }
                .category-highlight {
                    display: inline-block;
                    padding: 2px 8px;
                    margin: 0 2px;
                    background: #e9ecef;
                    border-radius: 3px;
                    font-weight: bold;
                    color: #495057;
                }
            </style>
        </head>
        <body>
    "#
        .to_string(),
    );

    html.push_str("<h1>thag_rs Demo Scripts</h1>");

    // Enhanced categories display
    let highlighted_categories = categories_str
        .split(", ")
        .map(|cat| format!("<span class=\"category-highlight\">{}</span>", cat))
        .collect::<Vec<_>>()
        .join(" ");

    html.push_str(&format!(
        "<div class=\"categories-header\">Matching categories and crates: {}</div>",
        highlighted_categories
    ));

    for meta in metadata_list {
        html.push_str(&format!(
            r#"
            <div class="script-item">
                <h2>{}</h2>
                <p>{}</p>
                <p><span class="metadata-label">Purpose:</span> {}</p>
        "#,
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
                "<p><span class=\"metadata-label\">Crates:</span> {}</p>",
                meta.crates.join(", ")
            ));
        }

        html.push_str(&format!(
            r#"
                <p><span class="metadata-label">Categories:</span> {}</p>
                <p><a href="/edit/{}" class="edit-link">Edit script</a></p>
            </div>
            "#,
            meta.categories.join(", "),
            meta.script
        ));
    }

    html.push_str("</body></html>");
    html
}

fn output_markdown(categories_str: &str, metadata_list: &[ScriptMetadata]) -> String {
    let mut md = String::from("# thag_rs demo scripts\n\n");

    // Enhanced categories display
    md.push_str(&format!("## Matching categories\n\n"));
    for category in categories_str.split(", ") {
        md.push_str(&format!("- **{}**\n", category));
    }
    md.push_str("\n---\n\n");

    for meta in metadata_list {
        md.push_str(&format!("## {}\n\n", meta.script));
        md.push_str(&format!(
            "{}\n\n",
            meta.description
                .as_ref()
                .unwrap_or(&String::from("No description available"))
        ));
        md.push_str(&format!(
            "**Purpose:** {}\n\n",
            meta.purpose
                .as_ref()
                .unwrap_or(&String::from("No purpose specified"))
        ));

        if !meta.crates.is_empty() {
            md.push_str(&format!("**Crates:** {}\n\n", meta.crates.join(", ")));
        }

        md.push_str(&format!(
            "**Categories:** {}\n\n",
            meta.categories.join(", ")
        ));
        md.push_str("---\n\n");
    }

    md
}

fn display_in_pager(content: &str) {
    let mut less = Command::new("less")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn less");

    if let Some(mut stdin) = less.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(content.as_bytes())
            .expect("Failed to write to less");
    }

    less.wait().expect("Failed to wait for less");
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

struct FileNavigator {
    current_dir: PathBuf,
    history: Vec<PathBuf>,
}

impl FileNavigator {
    fn new() -> Self {
        Self {
            current_dir: current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            history: Vec::new(),
        }
    }

    fn list_items(&self) -> Vec<String> {
        let mut items = vec!["*SELECT CURRENT DIRECTORY*".to_string(), "..".to_string()];

        // Add directories
        let mut dirs: Vec<_> = std::fs::read_dir(&self.current_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        dirs.sort();
        items.extend(dirs.into_iter().map(|d| format!("📁 {d}")));

        // Add .md files
        let mut files: Vec<_> = std::fs::read_dir(&self.current_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                    && entry.path().extension().is_some_and(|ext| ext == "md")
            })
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();
        items.extend(files.into_iter().map(|f| format!("📄 {f}")));

        items
    }

    fn navigate(&mut self, selection: &str) -> Option<PathBuf> {
        if selection == ".." {
            if let Some(parent) = self.current_dir.parent() {
                self.history.push(self.current_dir.clone());
                self.current_dir = parent.to_path_buf();
            }
            None
        } else {
            let clean_name = selection.trim_start_matches(['📁', '📄', ' ']);
            let new_path = self.current_dir.join(clean_name);

            if new_path.is_dir() {
                self.history.push(self.current_dir.clone());
                self.current_dir = new_path;
                None
            } else {
                Some(new_path)
            }
        }
    }

    fn current_path(&self) -> &PathBuf {
        &self.current_dir
    }
}

fn generate_default_filename(
    categories: &[Category],
    selected_crates: &[String],
    logic: &FilterLogic,
) -> String {
    let logic_str = match logic {
        FilterLogic::And => "and",
        FilterLogic::Or => "or",
    };

    let parts: Vec<String> = categories
        .iter()
        .map(|cat| {
            cat.to_string()
                .to_lowercase()
                .chars()
                .take(3)
                .collect::<String>()
        })
        .chain(
            selected_crates
                .iter()
                .map(|crate_name| crate_name.chars().take(3).collect::<String>()),
        )
        .collect();

    if parts.is_empty() {
        return "demo_all.md".to_string();
    }

    // parts.sort(); // Optional: sort for consistent ordering
    format!("demo_{}.md", parts.join(&format!("_{}_", logic_str)))
}

fn save_markdown_to_file(content: String, default_name: String) -> std::io::Result<PathBuf> {
    let mut navigator = FileNavigator::new();
    // let mut selected_dir = None;

    println!("Select destination directory (use arrow keys and Enter to navigate):");

    let selected_dir = loop {
        let items = navigator.list_items();
        let selection = Select::new(
            &format!("Current directory: {}", navigator.current_path().display()),
            items,
        )
        .with_help_message("Press Enter to navigate, Space to select current directory")
        .prompt();

        match selection {
            Ok(sel) => {
                if sel == "." || sel == "*SELECT CURRENT DIRECTORY*" {
                    // User selected current directory
                    // selected_dir = Some(navigator.current_path().to_path_buf());
                    break Some(navigator.current_path().to_path_buf());
                } else if let Some(_path) = navigator.navigate(&sel) {
                    // If a file is selected, ignore it and continue navigation
                    continue;
                }
            }
            Err(inquire::error::InquireError::OperationCanceled)
            | Err(inquire::error::InquireError::OperationInterrupted) => {
                // User wants to cancel the whole operation
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Selection cancelled",
                ));
            }
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unexpected error",
                ))
            }
        }
    };

    if let Some(dir) = selected_dir {
        // Get filename
        let filename = Text::new("Enter filename:")
            .with_default(&default_name)
            .prompt()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let full_path = dir.join(filename);
        fs::write(&full_path, content)?;
        Ok(full_path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No directory selected",
        ))
    }
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

    let (crates, _main_methods) = match maybe_syntax_tree {
        Some(ref ast) => {
            let crates_finder = shared::find_crates(&ast);
            let metadata_finder = shared::find_metadata(&ast);
            (
                code_utils::infer_deps_from_ast(&crates_finder, &metadata_finder),
                metadata_finder.main_count,
            )
        }
        None => {
            let re = regex!(r"(?m)^\s*(async\s+)?fn\s+main\s*\(\s*\)");
            (
                code_utils::infer_deps_from_source(&content),
                re.find_iter(&content).count(),
            )
        }
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

mod tests {
    use crate::generate_default_filename;
    use crate::FilterLogic;

    #[test]
    fn test_filename_generation() {
        // Test setup would depend on your Category enum implementation
        let categories = vec![/* your category values */];
        let crates = vec!["tokio".to_string(), "serde".to_string()];
        let logic = FilterLogic::And;

        let filename = generate_default_filename(&categories, &crates, &logic);
        // Assert based on expected output
        assert!(filename.starts_with("demo_"));
        assert!(filename.ends_with(".md"));
        assert!(filename.contains("tok"));
        assert!(filename.contains("ser"));
    }
}
