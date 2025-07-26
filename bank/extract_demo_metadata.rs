/// Extract description and categories from demo file content
/// Being used to debug and test thag_demo main.rs fn extract_demo_metadata
use anyhow::Result;
use std::{fs, path::{Path, PathBuf}};

/// Metadata about a demo file
#[derive(Debug, Clone)]
struct DemoFile {
    name: String,
    path: PathBuf,
    description: String,
    categories: Vec<String>,
    sample_arguments: Option<String>,
    usage_example: Option<String>,
}

fn extract_demo_metadata(path: &Path) -> Result<Option<DemoFile>> {
    let content = fs::read_to_string(path)?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut description = None;
    let mut categories = Vec::new();
    let mut sample_arguments = None;
    let mut usage_example = None;

    let lines: Vec<&str> = content.lines().collect();
    // eprintln!("lines={lines:#?}");
    let mut _in_doc_comment = false;
    let mut _in_block_doc_comment = false;

    for line in lines {
        let trimmed = line.trim();
        eprintln!(r#"trimmed={trimmed}, trimmed.starts_with("///")={}"#, trimmed.starts_with("///"));

        // Look for block doc comments (/** ... */)
        if trimmed.starts_with("/**") {
            _in_block_doc_comment = true;
        } else if _in_block_doc_comment && trimmed.starts_with("*/") {
            _in_block_doc_comment = false;
            _in_doc_comment = true;
        }

        // Look for doc comments (///)
        else if trimmed.starts_with("///") {
            _in_doc_comment = true;
            let comment_text = trimmed.trim_start_matches("///").trim();
            if !comment_text.is_empty() && description.is_none() {
                description = Some(comment_text.to_string());
            }
            // Look for usage examples in doc comments
            eprintln!("comment_text={comment_text}");
            if comment_text.starts_with("E.g.") || comment_text.contains("thag") {
                usage_example = Some(comment_text.to_string());
            }
        }

        else if _in_block_doc_comment {
            if !trimmed.starts_with("/**") {
                let comment_text = trimmed;
                if !comment_text.is_empty() && description.is_none() {
                    description = Some(comment_text.to_string());
                }
                // Look for usage examples in doc comments
                if comment_text.starts_with("E.g.") || comment_text.contains("thag") {
                    usage_example = Some(comment_text.to_string());
                }
            }
        }

        // Look for categories comment (//# Categories:)
        else if trimmed.starts_with("//# Categories:") {
            let cats_text = trimmed.trim_start_matches("//# Categories:").trim();
            categories = cats_text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        // Look for sample arguments comment (//# Sample arguments:)
        else if trimmed.starts_with("//# Sample arguments:") {
            let args_text = trimmed.trim_start_matches("//# Sample arguments:").trim();
            sample_arguments = Some(args_text.to_string());
        }
        // Stop at first non-comment line
        else if _in_doc_comment && !trimmed.starts_with("//") && !trimmed.is_empty() {
            break;
        }
    }

    let final_description = description.unwrap_or_else(|| "No description available".to_string());

    Ok(Some(DemoFile {
        name,
        path: path.to_path_buf(),
        description: final_description,
        categories,
        sample_arguments,
        usage_example,
    }))
}

let args: Vec<String> = env::args().collect();
if args.len() != 2 {
    eprintln!("Usage: {} <path>", args[0]);
    std::process::exit(1);
}

let path_str = args[1].clone();

let path = PathBuf::from(path_str);

println!("{:#?}", extract_demo_metadata(&path));
