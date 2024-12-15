/*[toml]
[dependencies]
edit = "0.1.5"
inquire = "0.7.5"
#log = "0.4.22"
regex = "1.10.5"
strum = { version = "0.26.3", features = ["derive"] }
syn = "2"
# thag_rs = "0.1.8"
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "main" }
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

#[derive(Clone, Debug, Display)]
enum FilterLogic {
    And,
    Or,
    All,
}

impl FilterLogic {
    #[allow(dead_code)]
    fn prompt_text(&self) -> &'static str {
        match self {
            FilterLogic::And => "AND (restrictive filtering)",
            FilterLogic::Or => "OR (inclusive filtering)",
            FilterLogic::All => "ALL (no filtering)",
        }
    }

    fn simple_text(&self) -> &'static str {
        match self {
            FilterLogic::And => "and",
            FilterLogic::Or => "or",
            FilterLogic::All => "all",
        }
    }
}

#[derive(Debug)]
struct FilterPreferences {
    category_logic: FilterLogic,
    crate_logic: FilterLogic,
    combination_logic: FilterLogic,
}

#[derive(Debug, Clone)]
struct LogicChoice {
    logic: FilterLogic,
    description: &'static str,
}

impl std::fmt::Display for LogicChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl LogicChoice {
    fn new(logic: FilterLogic, description: &'static str) -> Self {
        Self { logic, description }
    }
}

struct EscapeState {
    pressed: bool,
}

impl EscapeState {
    fn new() -> Self {
        Self { pressed: false }
    }

    fn handle_escape(&mut self) -> Result<(), &'static str> {
        if self.pressed {
            Err("Operation cancelled")
        } else {
            self.pressed = true;
            println!("Press Esc again to exit, or continue to retry");
            Ok(())
        }
    }

    fn reset(&mut self) {
        self.pressed = false;
    }
}

fn get_user_preferences(
    available_crates: &BTreeSet<String>,
) -> Result<(FilterPreferences, Vec<Category>, Vec<String>, OutputFormat), &'static str> {
    let mut escape_state = EscapeState::new();

    loop {
        // Reset escape flag at the start of each attempt
        escape_state.reset();

        // Category logic
        let choices = vec![
            LogicChoice::new(FilterLogic::Or, "OR (inclusive filtering)"),
            LogicChoice::new(FilterLogic::And, "AND (restrictive filtering)"),
            LogicChoice::new(FilterLogic::All, "ALL (no filtering)"),
        ];

        let category_logic = match Select::new("Select category filtering logic:", choices).prompt()
        {
            Ok(choice) => choice.logic,
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => {
                escape_state.handle_escape()?;
                continue;
            }
            Err(_) => return Err("Unexpected error"),
        };

        escape_state.reset();

        // Categories
        let categories = if !matches!(category_logic, FilterLogic::All) {
            match MultiSelect::new("Select categories:", Category::iter().collect::<Vec<_>>())
                .prompt()
            {
                Ok(cats) => cats,
                Err(
                    inquire::error::InquireError::OperationCanceled
                    | inquire::error::InquireError::OperationInterrupted,
                ) => {
                    escape_state.handle_escape()?;
                    continue;
                }
                Err(_) => return Err("Unexpected error"),
            }
        } else {
            Vec::new()
        };

        escape_state.reset();

        // Crate logic
        let crate_choices = vec![
            LogicChoice::new(FilterLogic::Or, "OR (inclusive filtering)"),
            LogicChoice::new(FilterLogic::And, "AND (restrictive filtering)"),
            LogicChoice::new(FilterLogic::All, "ALL (no filtering)"),
        ];

        let crate_logic = match Select::new("Select crate filtering logic:", crate_choices).prompt()
        {
            Ok(choice) => choice.logic,
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => {
                escape_state.handle_escape()?;
                continue;
            }
            Err(_) => return Err("Unexpected error"),
        };

        escape_state.reset();

        // Selected crates
        let selected_crates =
            if !matches!(crate_logic, FilterLogic::All) && !available_crates.is_empty() {
                match MultiSelect::new(
                    "Select crates to filter by:",
                    available_crates.iter().cloned().collect::<Vec<_>>(),
                )
                .prompt()
                {
                    Ok(crates) => crates,
                    Err(
                        inquire::error::InquireError::OperationCanceled
                        | inquire::error::InquireError::OperationInterrupted,
                    ) => {
                        escape_state.handle_escape()?;
                        continue;
                    }
                    Err(_) => return Err("Unexpected error"),
                }
            } else {
                Vec::new()
            };

        escape_state.reset();

        // Combination logic
        let combination_logic = if !matches!(category_logic, FilterLogic::All)
            || !matches!(crate_logic, FilterLogic::All)
        {
            let choices = vec![
                LogicChoice::new(FilterLogic::Or, "OR (inclusive filtering)"),
                LogicChoice::new(FilterLogic::And, "AND (restrictive filtering)"),
            ];

            match Select::new("How should categories and crates be combined?", choices).prompt() {
                Ok(choice) => choice.logic,
                Err(
                    inquire::error::InquireError::OperationCanceled
                    | inquire::error::InquireError::OperationInterrupted,
                ) => {
                    escape_state.handle_escape()?;
                    continue;
                }
                Err(_) => return Err("Unexpected error"),
            }
        } else {
            FilterLogic::Or
        };

        escape_state.reset();

        // Output format
        let format = match Select::new("Select output format:", vec!["HTML", "Markdown"]).prompt() {
            Ok(s) => match s {
                "Markdown" => {
                    match Select::new(
                        "How would you like to view the Markdown?",
                        vec!["Display in pager", "Save to file"],
                    )
                    .prompt()
                    {
                        Ok(s) => match s {
                            "Save to file" => OutputFormat::MarkdownFile,
                            _ => OutputFormat::MarkdownPager,
                        },
                        Err(
                            inquire::error::InquireError::OperationCanceled
                            | inquire::error::InquireError::OperationInterrupted,
                        ) => {
                            escape_state.handle_escape()?;
                            continue;
                        }
                        Err(_) => return Err("Unexpected error"),
                    }
                }
                _ => OutputFormat::Html,
            },
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => {
                escape_state.handle_escape()?;
                continue;
            }
            Err(_) => return Err("Unexpected error"),
        };

        // If we got here, everything succeeded
        return Ok((
            FilterPreferences {
                category_logic,
                crate_logic,
                combination_logic,
            },
            categories,
            selected_crates,
            format,
        ));
    }
}

#[derive(Debug)]
enum OutputFormat {
    Html,
    MarkdownPager,
    MarkdownFile,
}

fn apply_filters(
    meta: &ScriptMetadata,
    prefs: &FilterPreferences,
    category_strings: &[String],
    selected_crates: &[String],
) -> bool {
    let category_match = match prefs.category_logic {
        FilterLogic::All => true, // No category filtering
        FilterLogic::Or => category_strings.iter().any(|cat| {
            meta.categories
                .iter()
                .any(|meta_cat| meta_cat.to_lowercase() == *cat)
        }),
        FilterLogic::And => category_strings.iter().all(|cat| {
            meta.categories
                .iter()
                .any(|meta_cat| meta_cat.to_lowercase() == *cat)
        }),
    };

    let crate_match = match prefs.crate_logic {
        FilterLogic::All => true, // No crate filtering
        FilterLogic::Or => selected_crates.iter().any(|c| meta.crates.contains(c)),
        FilterLogic::And => selected_crates.iter().all(|c| meta.crates.contains(c)),
    };

    // If both are ALL, include everything
    if matches!(prefs.category_logic, FilterLogic::All)
        && matches!(prefs.crate_logic, FilterLogic::All)
    {
        true
    } else {
        // Otherwise use the chosen combination logic
        match prefs.combination_logic {
            FilterLogic::Or => category_match || crate_match,
            FilterLogic::And => category_match && crate_match,
            FilterLogic::All => unreachable!(), // ALL isn't an option for combination_logic
        }
    }
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

    // Get user preferences with error handling
    let (filter_prefs, categories, selected_crates, format) =
        match get_user_preferences(&available_crates) {
            Ok(prefs) => prefs,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

    // Convert categories to strings for filtering
    let category_strings: Vec<String> = categories
        .iter()
        .map(|c| c.to_string().to_lowercase())
        .collect();

    // Filter metadata
    let metadata = all_metadata
        .into_iter()
        .filter(|meta| apply_filters(meta, &filter_prefs, &category_strings, &selected_crates))
        .collect::<Vec<_>>();

    // Create filter description
    let (categories_desc, crates_desc, combination_op) =
        create_filter_description(&category_strings, &selected_crates, &filter_prefs);

    // Handle different output formats
    match format {
        OutputFormat::MarkdownPager => {
            let markdown =
                output_markdown(&categories_desc, &crates_desc, &combination_op, &metadata);
            display_in_pager(&markdown);
        }
        OutputFormat::MarkdownFile => {
            let markdown =
                output_markdown(&categories_desc, &crates_desc, &combination_op, &metadata);
            let default_name =
                generate_default_filename(&categories, &selected_crates, &filter_prefs);
            match save_markdown_to_file(markdown, default_name) {
                Ok(path) => println!("Markdown file saved successfully to: {}", path.display()),
                Err(e) => eprintln!("Error saving file: {}", e),
            }
        }
        OutputFormat::Html => {
            let html_report =
                generate_html_report(&categories_desc, &crates_desc, &combination_op, &metadata);
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

fn create_filter_description(
    category_strings: &[String],
    selected_crates: &[String],
    filter_prefs: &FilterPreferences,
) -> (String, String, String) {
    // Added combination operator
    let categories_desc = match filter_prefs.category_logic {
        FilterLogic::All => "all".to_string(),
        _ if category_strings.is_empty() => "none".to_string(),
        _ => category_strings.join(&format!(" {} ", filter_prefs.category_logic.simple_text())),
    };

    let crates_desc = match filter_prefs.crate_logic {
        FilterLogic::All => "all".to_string(),
        _ if selected_crates.is_empty() => "none".to_string(),
        _ => selected_crates.join(&format!(" {} ", filter_prefs.crate_logic.simple_text())),
    };

    let combination_op = filter_prefs.combination_logic.simple_text().to_uppercase();

    (categories_desc, crates_desc, combination_op)
}

fn generate_html_report(
    categories_desc: &str,
    crates_desc: &str,
    combination_op: &str,
    metadata_list: &[ScriptMetadata],
) -> String {
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
            .filter-description {
                margin: 20px 0;
                font-family: monospace;
            }
            .filter-line {
                display: grid;
                grid-template-columns: 100px 1fr;
                gap: 10px;
                align-items: start;
            }
            .combination-op {
                 font-weight: bold;
                 margin: 10px 0;
                 /*padding-left: 100px;   Match the grid layout of filter-line */
             }
         </style>
     </head>
     <body>
 "#
        .to_string(),
    );

    html.push_str("<h1>thag_rs Demo Scripts</h1>");
    html.push_str(&format!(
        "<h2>Matching categories {} crates:</h2>",
        combination_op
    ));
    html.push_str(&format!(
        r#"
        <div class="filter-description">
            <div class="filter-line">
                <span><b>categories:</b></span>
                <span>{}</span>
            </div>
            <div class="combination-op">{}</div>
            <div class="filter-line">
                <span><b>crates:</b></span>
                <span>{}</span>
            </div>
        </div>
    "#,
        categories_desc, combination_op, crates_desc
    ));

    // Rest of the HTML generation...
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

fn output_markdown(
    categories_desc: &str,
    crates_desc: &str,
    combination_op: &str,
    metadata_list: &[ScriptMetadata],
) -> String {
    let mut md = String::from("# thag_rs Demo Scripts\n\n");
    md.push_str(&format!(
        "## Matching categories {} crates:\n\n",
        combination_op
    ));
    md.push_str(&format!("**categories:** {}\n\n", categories_desc));
    md.push_str(&format!("{}\n\n", combination_op));
    md.push_str(&format!("**crates:**     {}\n\n", crates_desc));

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
    filter_prefs: &FilterPreferences,
) -> String {
    let mut parts: Vec<String> = categories
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

    parts.sort();
    let logic_str = filter_prefs.combination_logic.simple_text();
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

#[cfg(test)]
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
