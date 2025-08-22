/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["ast", "simplelog"] }
*/
use inquire::set_global_render_config;
/// Select demo scripts and generate and serve HTML report.
///
/// Strategy and grunt work thanks to `ChatGPT`.
//# Purpose: Allow user to select scripts by category.
//# Categories: technique, tools
use inquire::MultiSelect;
use std::fmt::Write as _; // import without risk of name clashing
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, read_dir},
    path::{Path, PathBuf},
    process::Command,
};
use thag_proc_macros::{category_enum, file_navigator};
use thag_rs::code_utils::to_ast;
use thag_rs::help_system::check_help_and_exit;
use thag_rs::{ast, auto_help, re, themed_inquire_config};
use warp::Filter;

category_enum! {} // This will generate the Category enum

// Generate the FileNavigator struct, its implementation and the save function.
file_navigator! {}

#[derive(Clone, Debug, Display)]
enum FilterLogic {
    And,
    Or,
    All,
}

impl FilterLogic {
    #[allow(dead_code)]
    const fn prompt_text(&self) -> &'static str {
        match self {
            Self::And => "AND (restrictive filtering)",
            Self::Or => "OR (inclusive filtering)",
            Self::All => "ALL (no filtering)",
        }
    }

    const fn simple_text(&self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
            Self::All => "all",
        }
    }
}

#[derive(Debug)]
struct FilterPreferences {
    category: FilterLogic,
    crate_logic: FilterLogic,
    combination: FilterLogic,
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
    const fn new(logic: FilterLogic, description: &'static str) -> Self {
        Self { logic, description }
    }
}

struct EscapeState {
    pressed: bool,
}

impl EscapeState {
    const fn new() -> Self {
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

type UserPreferences = (FilterPreferences, Vec<Category>, Vec<String>, OutputFormat);

#[allow(clippy::too_many_lines)]
fn get_user_preferences(
    available_crates: &BTreeSet<String>,
) -> Result<UserPreferences, &'static str> {
    let mut escape_state = EscapeState::new();

    loop {
        // // Reset escape flag at the start of each attempt
        // escape_state.reset();

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
        let categories = if matches!(category_logic, FilterLogic::All) {
            Vec::new()
        } else {
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
                category: category_logic,
                crate_logic,
                combination: combination_logic,
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
    let category_match = match prefs.category {
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
    if matches!(prefs.category, FilterLogic::All) && matches!(prefs.crate_logic, FilterLogic::All) {
        true
    } else {
        // Otherwise use the chosen combination logic
        match prefs.combination {
            FilterLogic::Or => category_match || crate_match,
            FilterLogic::And => category_match && crate_match,
            FilterLogic::All => unreachable!(), // ALL isn't an option for combination_logic
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_find_demos");
    check_help_and_exit(&help);

    set_global_render_config(themed_inquire_config());

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
                eprintln!("{e}");
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
            match save_to_file(markdown, &default_name, Some("md"), false) {
                Ok(path) => println!("Markdown file saved successfully to: {}", path.display()),
                Err(e) => eprintln!("Error saving file: {e}"),
            }
        }
        OutputFormat::Html => {
            let html_report =
                generate_html_report(&categories_desc, &crates_desc, &combination_op, &metadata);
            let edit_route = warp::path("edit").and(warp::path::param::<String>()).map(
                move |script_name: String| {
                    let script_path = Path::new(&scripts_dir).join(&script_name);
                    if edit::edit_file(&script_path).is_ok() {
                        format!("Editing script: {script_name}")
                    } else {
                        format!("Failed to edit script: {script_name}")
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
    let categories_desc = match filter_prefs.category {
        FilterLogic::All => "all".to_string(),
        _ if category_strings.is_empty() => "none".to_string(),
        _ => category_strings.join(&format!(" {} ", filter_prefs.category.simple_text())),
    };

    let crates_desc = match filter_prefs.crate_logic {
        FilterLogic::All => "all".to_string(),
        _ if selected_crates.is_empty() => "none".to_string(),
        _ => selected_crates.join(&format!(" {} ", filter_prefs.crate_logic.simple_text())),
    };

    let combination_op = filter_prefs.combination.simple_text().to_uppercase();

    (categories_desc, crates_desc, combination_op)
}

#[allow(clippy::too_many_lines)]
fn generate_html_report(
    categories_desc: &str,
    crates_desc: &str,
    combination_op: &str,
    metadata_list: &[ScriptMetadata],
) -> String {
    let mut html = r#"
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
    .to_string();

    html.push_str("<h1>thag_rs Demo Scripts</h1>");
    let _ = writeln!(
        html,
        "<h2>Matching categories {combination_op} crates:</h2>"
    );
    let _ = writeln!(
        html,
        r#"
        <div class="filter-description">
            <div class="filter-line">
                <span><b>categories:</b></span>
                <span>{categories_desc}</span>
            </div>
            <div class="combination-op">{combination_op}</div>
            <div class="filter-line">
                <span><b>crates:</b></span>
                <span>{crates_desc}</span>
            </div>
        </div>
    "#
    );

    // Rest of the HTML generation...
    #[allow(clippy::or_fun_call)]
    for meta in metadata_list {
        let _ = writeln!(
            html,
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
        );

        if !meta.crates.is_empty() {
            let _ = writeln!(
                html,
                "<p><span class=\"metadata-label\">Crates:</span> {}</p>",
                meta.crates.join(", ")
            );
        }

        let _ = writeln!(
            html,
            r#"
                <p><span class="metadata-label">Categories:</span> {}</p>
                <p><a href="/edit/{}" class="edit-link">Edit script</a></p>
            </div>
            "#,
            meta.categories.join(", "),
            meta.script
        );
    }

    html.push_str("</body></html>");
    html
}

#[allow(clippy::or_fun_call)]
fn output_markdown(
    categories_desc: &str,
    crates_desc: &str,
    combination_op: &str,
    metadata_list: &[ScriptMetadata],
) -> String {
    let mut md = String::from("# thag_rs Demo Scripts\n\n");
    let _ = writeln!(md, "## Matching categories {combination_op} crates:\n\n");
    let _ = writeln!(md, "**categories:** {categories_desc}\n\n");
    let _ = writeln!(md, "{combination_op}\n\n");
    let _ = writeln!(md, "**crates:**     {crates_desc}\n\n");

    for meta in metadata_list {
        let _ = writeln!(md, "## {}\n\n", meta.script);
        let _ = writeln!(
            md,
            "{}\n\n",
            meta.description
                .as_ref()
                .unwrap_or(&String::from("No description available"))
        );
        let _ = writeln!(
            md,
            "**Purpose:** {}\n\n",
            meta.purpose
                .as_ref()
                .unwrap_or(&String::from("No purpose specified"))
        );

        if !meta.crates.is_empty() {
            let _ = writeln!(md, "**Crates:** {}\n\n", meta.crates.join(", "));
        }

        let _ = writeln!(md, "**Categories:** {}\n\n", meta.categories.join(", "));
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
    let logic_str = filter_prefs.combination.simple_text();
    format!("demo_{}.md", parts.join(&format!("_{logic_str}_")))
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

    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("//#") {
            let parts: Vec<&str> = stripped.splitn(2, ':').collect();
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

    let maybe_syntax_tree = to_ast(file_path_str, &content);

    let (crates, _main_methods) = maybe_syntax_tree.as_ref().map_or_else(
        || {
            let re = re!(r"(?m)^\s*(async\s+)?fn\s+main\s*\(\s*\)");
            (
                ast::infer_deps_from_source(&content),
                re.find_iter(&content).count(),
            )
        },
        |ast| {
            let crates_finder = ast::find_crates(ast);
            let metadata_finder = ast::find_metadata(ast);
            (
                ast::infer_deps_from_ast(&crates_finder, &metadata_finder),
                metadata_finder.main_count,
            )
        },
    );

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

    for entry in &scripts {
        let path = entry.as_path();
        // println!("Parsing {:#?}", path.display());

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Some(metadata) = parse_metadata(path) {
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
