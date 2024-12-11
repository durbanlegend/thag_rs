/*[toml]
[dependencies]
# To switch off, use version without features = ["enable_system_time"]
# If on, prepare for function to run twice for some reason!
# firestorm = "0.5.1"
firestorm = { version = "0.5.1", features = ["enable_system_time"] }
lazy_static = "1.4.0"
log = "0.4.22"
regex = "1.10.5"
# thag_proc_macros = { version = "0.1.0", path = "/Users/donf/projects/thag_rs/src/proc_macros" }
thag_proc_macros = { git = "https://github.com/durbanlegend/thag_rs", rev = "ae819fc549a9d6a40d4907f6f981815c25882bfa" }
# thag_rs = "0.1.7"
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", rev = "ae819fc549a9d6a40d4907f6f981815c25882bfa" }
# thag_rs = { path = "/Users/donf/projects/thag_rs" }
*/

/// This is the actual script used to collect demo script metadata and generate
/// demo/README.md.
///
/// Strategy and grunt work thanks to ChatGPT.
//# Purpose: Document demo scripts in a demo/README.md as a guide to the user.
//# Categories: technique, tools
use firestorm::profile_fn;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, read_dir, File},
    io::Write as OtherWrite,
    path::{Path, PathBuf},
};
use thag_rs::{code_utils, lazy_static_var};
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::category_enum;

#[derive(Debug)]
struct ScriptMetadata {
    script: String,
    purpose: Option<String>,
    crates: Vec<String>,
    script_type: Option<String>,
    description: Option<String>,
    categories: Vec<String>,
    sample_args: Option<String>,
}

// Generates all_categories()
category_enum! {}

fn parse_metadata(file_path: &Path) -> Option<ScriptMetadata> {
    profile_fn!(parse_metadata);
    // Lazy static variable from the categories defined in macro category_enum!.
    let valid_categories = lazy_static_var!(Vec<String>, {
        let valid_categories = all_categories();
        // eprintln!("valid_categories={valid_categories:?}");
        valid_categories
    });
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
    let mut sample_args: Option<String> = None;

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
                        // eprintln!("{}: categories={categories:?}", file_path.display());
                        // Check all the categories are valid
                        assert!(
                            categories.iter().all(|cat| valid_categories.contains(cat)),
                            "One or more invalid categories found in {}",
                            file_path.display()
                        );
                    }
                    "sample arguments" => {
                        // Extract content between backticks, if present
                        let value = value.trim();
                        sample_args = if let Some(quoted) = value.strip_prefix('`') {
                            if let Some(args) = quoted.strip_suffix('`') {
                                Some(args.to_string())
                            } else {
                                Some(quoted.to_string())
                            }
                        } else {
                            Some(value.to_string())
                        };
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

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)^\s*(async\s+)?fn\s+main\s*\(\s*\)").unwrap();
    }
    let main_methods = match maybe_syntax_tree {
        Some(ref ast) => code_utils::count_main_methods(ast),
        None => RE.find_iter(&content).count(),
    };

    let script_type = if main_methods >= 1 {
        "Program"
    } else {
        "Snippet"
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
        script_type: Some(script_type.to_string()),
        description: description.cloned(),
        categories,
        sample_args,
    })
}

fn collect_all_metadata(scripts_dir: &Path) -> Vec<ScriptMetadata> {
    profile_fn!(collect_all_metadata);
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

fn generate_readme(metadata_list: &[ScriptMetadata], output_path: &Path, boilerplate_path: &Path) {
    profile_fn!(generate_readme);
    let mut file = File::create(output_path).unwrap();

    // Read boilerplate content
    let boilerplate = fs::read_to_string(boilerplate_path)
        .unwrap_or_else(|_| "## Running the scripts\n\n...".to_string()); // Fallback content if the file is missing

    // Write boilerplate to README
    writeln!(file, "{}", boilerplate).unwrap();
    writeln!(file, "***\n## Detailed script listing\n").unwrap();

    for metadata in metadata_list {
        writeln!(file, "### Script: {}\n", metadata.script).unwrap();
        write!(
            file,
            "**Description:** {}\n",
            metadata.description.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        writeln!(
            file,
            "**Purpose:** {}\n",
            metadata.purpose.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        let crates = metadata
            .crates
            .iter()
            .map(|v| format!("`{v}`"))
            .collect::<Vec<String>>();
        if !crates.is_empty() {
            writeln!(file, "**Crates:** {}\n", crates.join(", ")).unwrap();
        }
        writeln!(
            file,
            "**Type:** {}\n",
            metadata.script_type.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        writeln!(file, "**Categories:** {}\n", metadata.categories.join(", ")).unwrap(); // Include categories
        writeln!(
            file,
            "**Link:** [{}](https://github.com/durbanlegend/thag_rs/blob/master/demo/{})",
            metadata.script, metadata.script
        )
        .unwrap();

        // let example = Example::new(
        //     "https://github.com/durbanlegend/thag_rs/blob/develop/demo/fib_matrix.rs",
        //     vec!["10".to_string()],
        //     Some("Matrix-based Fibonacci calculation example".to_string()),
        // );
        // writeln!(
        //     file,
        //     "**Run this example:** [{}](https://github.com/durbanlegend/thag_rs/blob/master/demo/{})\n",
        //     metadata.script, metadata.script
        // )
        // .unwrap();
        let run_section = generate_run_section(metadata);
        writeln!(file, "{run_section}").unwrap();
        writeln!(file, "---\n").unwrap();
    }
}

fn generate_run_section(metadata: &ScriptMetadata) -> String {
    let mut md = String::new();
    md.push_str("\n**Run this example:**\n\n");
    md.push_str("```bash\n");

    let base_url = "https://github.com/durbanlegend/thag_rs/blob/master/demo";
    let command = if let Some(args) = &metadata.sample_args {
        format!("thag_url {}/{} {}", base_url, metadata.script, args)
    } else {
        format!("thag_url {}/{}", base_url, metadata.script)
    };

    md.push_str(&command);
    md.push_str("\n```\n");

    md
}

// struct Example {
//     url: String,
//     args: Vec<String>,
//     description: Option<String>,
// }

// impl Example {
//     fn new(url: impl Into<String>, args: Vec<String>, description: Option<String>) -> Self {
//         Self {
//             url: url.into(),
//             args,
//             description,
//         }
//     }

//     fn to_command(&self) -> String {
//         if self.args.is_empty() {
//             format!("thag_url {}", self.url)
//         } else {
//             format!("thag_url {} -- {}", self.url, self.args.join(" "))
//         }
//     }

//     fn generate_markdown(&self) -> String {
//         let mut md = String::new();
//         md.push_str("Run this example:\n\n```bash\n");
//         md.push_str(&self.to_command());
//         md.push_str("\n```\n");
//         if let Some(desc) = &self.description {
//             md.push_str("\n<details>\n<summary>About this example</summary>\n\n");
//             md.push_str(desc);
//             md.push_str("\n</details>\n");
//         }
//         md
//     }
// }

fn main() {
    profile_fn!(main);
    let scripts_dir = Path::new("demo");
    let output_path = Path::new("demo/README.md");
    let boilerplate_path = Path::new("assets/boilerplate.md");

    // Check if firestorm profiling is enabled
    // if firestorm::enabled() {
    //     // Profile the `execute` function
    //     // Use borrow_mut to get a mutable reference
    //     firestorm::bench("./flames/", || {
    //         let all_metadata = collect_all_metadata(scripts_dir);
    //         generate_readme(&all_metadata, output_path, boilerplate_path);
    //     })
    //     .unwrap();
    // } else {
    // Regular execution when profiling is not enabled
    let all_metadata = collect_all_metadata(scripts_dir);
    generate_readme(&all_metadata, output_path, boilerplate_path);
    // }

    println!("demo/README.md generated successfully.");
}
