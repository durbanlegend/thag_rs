/// This script generates documentation for proc macros defined in demo/proc_macros/lib.rs
/// and creates a comprehensive README.md file for the proc macros directory.
///
/// It extracts proc macro definitions, their documentation, and links them to their
/// corresponding example files in the demo/ directory.
//# Purpose: Generate README.md documentation for proc macros with examples and usage
//# Categories: technique, tools, proc_macros
// Simple case conversion without external dependencies
use std::{
    env,
    fs::{self, File},
    io::Write as OtherWrite,
    path::Path,
};
use syn::{parse_file, Attribute, Item, ItemFn, Lit, Meta, MetaNameValue};
use thag_rs::{auto_help, cvprtln, help_system::check_help_and_exit, Role, V};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProcMacroInfo {
    name: String,
    macro_type: ProcMacroType,
    doc_comment: String,
    attributes: Vec<String>,
    example_file: Option<String>,
    signature: String,
}

#[derive(Debug, Clone)]
enum ProcMacroType {
    Derive,
    Attribute,
    FunctionLike,
}

// impl ProcMacroType {
//     fn as_str(&self) -> &'static str {
//         match self {
//             ProcMacroType::Derive => "Derive Macro",
//             ProcMacroType::Attribute => "Attribute Macro",
//             ProcMacroType::FunctionLike => "Function-like Macro",
//         }
//     }
// }

fn extract_doc_comments(attrs: &[Attribute]) -> String {
    let mut doc_lines = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(MetaNameValue {
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }),
                ..
            }) = &attr.meta
            {
                let content = lit_str.value();
                // Remove leading space if present
                let content = content.strip_prefix(' ').unwrap_or(&content);
                doc_lines.push(content.to_string());
            }
        }
    }

    doc_lines.join("\n")
}

fn extract_proc_macro_attributes(attrs: &[Attribute]) -> Vec<String> {
    let mut proc_attrs = Vec::new();

    for attr in attrs {
        let path = attr.path();
        if path.is_ident("proc_macro_derive")
            || path.is_ident("proc_macro_attribute")
            || path.is_ident("proc_macro")
        {
            proc_attrs.push(format!("{}", quote::quote!(#attr)));
        }
    }

    proc_attrs
}

fn determine_macro_type_and_name(item: &ItemFn) -> Option<(String, ProcMacroType)> {
    for attr in &item.attrs {
        let path = attr.path();

        if path.is_ident("proc_macro_derive") {
            // Extract derive macro name from attribute
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = &meta_list.tokens;
                let token_str = tokens.to_string();
                // Parse the first identifier before any comma or attribute list
                if let Some(name) = token_str
                    .split(',')
                    .next()
                    .and_then(|s| s.split('(').next())
                {
                    return Some((name.trim().to_string(), ProcMacroType::Derive));
                }
            }
        } else if path.is_ident("proc_macro_attribute") {
            return Some((item.sig.ident.to_string(), ProcMacroType::Attribute));
        } else if path.is_ident("proc_macro") {
            return Some((item.sig.ident.to_string(), ProcMacroType::FunctionLike));
        }
    }

    None
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 && prev_was_lower {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
        prev_was_lower = c.is_lowercase();
    }

    result
}

fn find_example_file(macro_name: &str, demo_dir: &Path) -> Option<String> {
    // Convert macro name to expected file pattern
    let snake_case_name = to_snake_case(macro_name);
    let expected_patterns = vec![
        format!("proc_macro_{}.rs", snake_case_name),
        format!("proc_macro_{}.rs", macro_name.to_lowercase()),
    ];

    for pattern in expected_patterns {
        let file_path = demo_dir.join(&pattern);
        if file_path.exists() {
            return Some(pattern);
        }
    }

    None
}

fn parse_proc_macros(
    lib_path: &Path,
    demo_dir: &Path,
) -> Result<Vec<ProcMacroInfo>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(lib_path)?;
    let syntax_tree = parse_file(&content)?;

    let mut macros = Vec::new();

    for item in syntax_tree.items {
        if let Item::Fn(item_fn) = item {
            if let Some((name, macro_type)) = determine_macro_type_and_name(&item_fn) {
                let doc_comment = extract_doc_comments(&item_fn.attrs);
                let attributes = extract_proc_macro_attributes(&item_fn.attrs);
                let example_file = find_example_file(&name, demo_dir);
                let signature = format!("pub fn {}(...) -> TokenStream", item_fn.sig.ident);

                macros.push(ProcMacroInfo {
                    name,
                    macro_type,
                    doc_comment,
                    attributes,
                    example_file,
                    signature,
                });
            }
        }
    }

    // Sort by name for consistent output
    macros.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(macros)
}

fn extract_example_metadata(example_path: &Path) -> (Option<String>, Option<String>) {
    let content = match fs::read_to_string(example_path) {
        Ok(c) => c,
        Err(_) => return (None, None),
    };

    let mut purpose = None;
    let mut description = Vec::new();

    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("//# Purpose:") {
            purpose = Some(stripped.trim().to_string());
        } else if line.starts_with("///") {
            let doc_line = line[3..].trim();
            if !doc_line.is_empty() {
                description.push(doc_line.to_string());
            }
        }
    }

    let description = if description.is_empty() {
        None
    } else {
        Some(description.join(" "))
    };

    (purpose, description)
}

fn generate_readme(
    macros: &[ProcMacroInfo],
    output_path: &Path,
    demo_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;

    // Write header
    writeln!(file, "# Procedural Macros Documentation\n")?;
    writeln!(file, "This directory contains a collection of procedural macros demonstrating various techniques and patterns for writing proc macros in Rust.\n")?;

    // Write overview
    writeln!(file, "## Overview\n")?;
    writeln!(file, "The procedural macros in this crate showcase:")?;
    writeln!(
        file,
        "\n- **Derive macros**: Generate implementations for traits automatically"
    )?;
    writeln!(
        file,
        "\n- **Attribute macros**: Transform or augment code with custom attributes"
    )?;
    writeln!(
        file,
        "\n- **Function-like macros**: Generate code using function-like syntax\n"
    )?;

    // Group macros by type
    let mut derive_macros = Vec::new();
    let mut attribute_macros = Vec::new();
    let mut function_like_macros = Vec::new();

    for macro_info in macros {
        match macro_info.macro_type {
            ProcMacroType::Derive => derive_macros.push(macro_info),
            ProcMacroType::Attribute => attribute_macros.push(macro_info),
            ProcMacroType::FunctionLike => function_like_macros.push(macro_info),
        }
    }

    // Write each section
    write_macro_section(&mut file, "Derive Macros", &derive_macros, demo_dir)?;
    write_macro_section(&mut file, "Attribute Macros", &attribute_macros, demo_dir)?;
    write_macro_section(
        &mut file,
        "Function-like Macros",
        &function_like_macros,
        demo_dir,
    )?;

    // Write usage section
    writeln!(file, "## Usage\n")?;
    writeln!(file, "To use these macros in your project:\n")?;
    writeln!(file, "```toml")?;
    writeln!(file, "[dependencies]")?;
    writeln!(
        file,
        "thag_demo_proc_macros = {{ path = \"demo/proc_macros\" }}"
    )?;
    writeln!(file, "```\n")?;

    writeln!(file, "Or when using `thag_rs`:\n")?;
    writeln!(file, "```rust")?;
    writeln!(
        file,
        "// \"thag_demo_proc_macros\" is automatically resolved"
    )?;
    writeln!(file, "use thag_demo_proc_macros::{{YourMacro}};")?;
    writeln!(file, "```\n")?;

    // Write development section
    writeln!(file, "## Development\n")?;
    writeln!(file, "### Building")?;
    writeln!(file, "```bash")?;
    writeln!(file, "cd demo/proc_macros")?;
    writeln!(file, "cargo build")?;
    writeln!(file, "```\n")?;

    writeln!(file, "### Testing")?;
    writeln!(file, "```bash")?;
    writeln!(file, "cargo test")?;
    writeln!(file, "```\n")?;

    writeln!(file, "### Macro Expansion")?;
    writeln!(
        file,
        "Many macros support the `expand` feature to show generated code:"
    )?;
    writeln!(file, "```bash")?;
    writeln!(file, "cargo build --features expand")?;
    writeln!(file, "```\n")?;

    Ok(())
}

fn write_macro_section(
    file: &mut File,
    section_title: &str,
    macros: &[&ProcMacroInfo],
    demo_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if macros.is_empty() {
        return Ok(());
    }

    writeln!(file, "## {}\n", section_title)?;

    for macro_info in macros {
        writeln!(file, "### `{}`\n", macro_info.name)?;

        // Write documentation
        if !macro_info.doc_comment.is_empty() {
            writeln!(file, "{}\n", macro_info.doc_comment)?;
        }

        // Write example file link and info if available
        if let Some(example_file) = &macro_info.example_file {
            let example_path = demo_dir.parent().unwrap().join(example_file);
            let (purpose, description) = extract_example_metadata(&example_path);

            writeln!(
                file,
                "**Example Usage:** [{}](../{})\n",
                example_file, example_file
            )?;

            if let Some(purpose) = purpose {
                writeln!(file, "**Purpose:** {}\n", purpose)?;
            }

            if let Some(description) = description {
                writeln!(file, "**Description:** {}\n", description)?;
            }

            // Generate run command
            writeln!(file, "**Run Example:**")?;
            writeln!(file, "\n```bash")?;
            writeln!(
                file,
                "thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/{}",
                example_file
            )?;
            writeln!(file, "```\n")?;
        }

        writeln!(file, "---\n")?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first
    let help = auto_help!("thag_gen_proc_macro_readme");
    check_help_and_exit(&help);

    let current_dir = env::current_dir()?;

    // Default paths
    let proc_macros_dir = current_dir.join("demo/proc_macros");
    let lib_path = proc_macros_dir.join("lib.rs");
    let demo_dir = current_dir.join("demo");
    let output_path = proc_macros_dir.join("README.md");

    // Verify paths exist
    if !lib_path.exists() {
        cvprtln!(
            Role::ERR,
            V::N,
            "lib.rs not found at: {}",
            lib_path.display()
        );
        std::process::exit(1);
    }

    if !demo_dir.exists() {
        cvprtln!(
            Role::ERR,
            V::N,
            "demo directory not found at: {}",
            demo_dir.display()
        );
        std::process::exit(1);
    }

    println!("Parsing proc macros from: {}", lib_path.display());
    println!("Looking for examples in: {}", demo_dir.display());

    // Parse proc macros from lib.rs
    let macros = parse_proc_macros(&lib_path, &demo_dir)?;

    if macros.is_empty() {
        cvprtln!(
            Role::WARN,
            V::N,
            "No proc macros found in {}",
            lib_path.display()
        );
        return Ok(());
    }

    println!("Found {} proc macros", macros.len());

    // Generate README
    generate_readme(&macros, &output_path, &demo_dir)?;

    println!("Generated README.md at: {}", output_path.display());

    // Print summary
    println!("\n=== Summary ===");
    println!(
        "Derive macros: {}",
        macros
            .iter()
            .filter(|m| matches!(m.macro_type, ProcMacroType::Derive))
            .count()
    );
    println!(
        "Attribute macros: {}",
        macros
            .iter()
            .filter(|m| matches!(m.macro_type, ProcMacroType::Attribute))
            .count()
    );
    println!(
        "Function-like macros: {}",
        macros
            .iter()
            .filter(|m| matches!(m.macro_type, ProcMacroType::FunctionLike))
            .count()
    );

    Ok(())
}
