//# Purpose: Collect script metadata
//# Crates: std::collections, std::fs, std::io, std::path
//# Target: all
//# Type: program

use std::collections::HashMap;
use std::fs::{self, read_dir, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
struct ScriptMetadata {
    script: String,
    purpose: Option<String>,
    crates: Option<String>,
    target: Option<String>,
    script_type: Option<String>,
    description: Option<String>,
}

fn parse_metadata(file_path: &Path) -> Option<ScriptMetadata> {
    let content = fs::read_to_string(file_path).ok()?;
    let mut metadata = HashMap::new();
    let mut lines = Vec::<String>::new();

    for line in content.lines() {
        if line.starts_with("//#") {
            let parts: Vec<&str> = line[3..].splitn(2, ':').collect();
            if parts.len() == 2 {
                metadata.insert(parts[0].trim().to_lowercase(), parts[1].trim().to_string());
            }
        } else if line.starts_with("///") {
            lines.push(line[3..].to_string() + "\n");
        }
    }
    metadata.insert("description".to_string(), lines.join(""));

    let script = file_path
        .file_name()
        .expect("Error accessing filename")
        .to_string_lossy()
        .to_string();

    let purpose = metadata.get("purpose");
    let crates = metadata.get("crates");
    let target = metadata.get("target");
    let script_type = metadata.get("type");
    let description = metadata.get("description");

    Some(ScriptMetadata {
        script,
        purpose: purpose.cloned(),
        crates: crates.cloned(),
        target: target.cloned(),
        script_type: script_type.cloned(),
        description: description.cloned(),
    })
}

fn collect_all_metadata(scripts_dir: &Path) -> Vec<ScriptMetadata> {
    let mut all_metadata = Vec::new();

    for entry in read_dir(scripts_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        println!("Parsing {:#?}", path.display());

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Some(metadata) = parse_metadata(&path) {
                all_metadata.push(metadata);
            }
        }
    }

    all_metadata.sort_by(|a, b| a.script.partial_cmp(&b.script).unwrap());

    all_metadata
}

fn generate_readme(metadata_list: &[ScriptMetadata], output_path: &Path) {
    let mut file = File::create(output_path).unwrap();
    writeln!(file, "# Demo Scripts\n").unwrap();

    for metadata in metadata_list {
        writeln!(file, "## Script: {}\n", metadata.script).unwrap();
        writeln!(
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
        writeln!(
            file,
            "**Crates:** {}\n",
            metadata.crates.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        writeln!(
            file,
            "**Target:** {}\n",
            metadata.target.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        writeln!(
            file,
            "**Type:** {}\n",
            metadata.script_type.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        writeln!(file, "---\n").unwrap();
    }
}

/// Collect demo script metadata and generate as demo/README.md.
/// Strategy and grunt work thanks to ChatGPT.
fn main() {
    let scripts_dir = Path::new("demo");
    let output_path = Path::new("demo/README.md");

    let all_metadata = collect_all_metadata(scripts_dir);
    generate_readme(&all_metadata, output_path);

    println!("demo/README.md generated successfully.");
}
