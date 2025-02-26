use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossterm::terminal;
use inquire::{Select, Text};
use side_by_side_diff::create_side_by_side_diff;
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, Output, Stdio},
};
use tempfile::tempdir;

/// Command-line arguments for the expanded code viewer
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "Show expanded macros with various viewing options"
)]
struct Args {
    /// The input Rust file to expand
    #[clap(name = "FILE")]
    input_file: String,

    /// Skip interactive mode and use the specified viewer
    #[clap(short, long)]
    viewer: Option<String>,

    /// Theme for cargo-expand (when applicable)
    #[clap(short, long, default_value = "gruvbox-dark")]
    theme: String,

    /// Width to use for side-by-side view (auto-detect if not specified)
    #[clap(short, long)]
    width: Option<u16>,
}

/// Available viewing options for expanded code
enum ViewerOption {
    SideBySide,
    SideBySideCustomWidth,
    ExpandedOnly,
    UnifiedDiff,
    ExternalViewer,
}

impl std::fmt::Display for ViewerOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewerOption::SideBySide => write!(f, "Side-by-side view (auto width)"),
            ViewerOption::SideBySideCustomWidth => write!(f, "Side-by-side view (custom width)"),
            ViewerOption::ExpandedOnly => write!(f, "Expanded code only"),
            ViewerOption::UnifiedDiff => write!(f, "Unified diff"),
            ViewerOption::ExternalViewer => write!(f, "External diff tool"),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check if file exists
    let input_path = Path::new(&args.input_file);
    if !input_path.exists() {
        return Err(anyhow!("File not found: {}", args.input_file));
    }

    // Read the original source
    let unexpanded_source = fs::read_to_string(input_path)?;

    // Get expanded source using cargo-expand
    let expanded_source = run_cargo_expand(input_path, &args.theme)
        .context("Failed to run cargo-expand. Is it installed? (cargo install cargo-expand)")?;

    let viewer = if let Some(viewer_name) = args.viewer {
        match viewer_name.as_str() {
            "side-by-side" => ViewerOption::SideBySide,
            "custom-width" => ViewerOption::SideBySideCustomWidth,
            "expanded-only" => ViewerOption::ExpandedOnly,
            "unified-diff" => ViewerOption::UnifiedDiff,
            "external" => ViewerOption::ExternalViewer,
            _ => {
                eprintln!("Unknown viewer: {}. Using side-by-side view.", viewer_name);
                ViewerOption::SideBySide
            }
        }
    } else {
        // Interactive mode - ask user for preferred viewing option
        let options = vec![
            ViewerOption::SideBySide,
            ViewerOption::SideBySideCustomWidth,
            ViewerOption::ExpandedOnly,
            ViewerOption::UnifiedDiff,
            ViewerOption::ExternalViewer,
        ];

        let selection = Select::new("How would you like to view the expanded code?", options)
            .with_help_message("Choose a viewing option for the expanded macros")
            .prompt()?;

        selection
    };

    // Display the expanded code according to the chosen view option
    match viewer {
        ViewerOption::SideBySide => {
            let width = args.width.unwrap_or_else(|| detect_terminal_width());
            let unexpanded_source = &unexpanded_source
                .lines()
                .map(|line| line.get(..width as usize).unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            let expanded_source = expanded_source
                .lines()
                .map(|line| line.get(..width as usize).unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            display_side_by_side(&unexpanded_source, &expanded_source, width)?;
        }
        ViewerOption::SideBySideCustomWidth => {
            let width_input = Text::new("Enter width for each column:")
                .with_default(&detect_terminal_width().to_string())
                .prompt()?;

            let width: u16 = width_input.parse().context("Invalid width")?;
            let unexpanded_source = &unexpanded_source
                .lines()
                .map(|line| line.get(..width as usize).unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            let expanded_source = expanded_source
                .lines()
                .map(|line| line.get(..width as usize).unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            display_side_by_side(&unexpanded_source, &expanded_source, width)?;
        }
        ViewerOption::ExpandedOnly => {
            println!("{}", expanded_source);
        }
        ViewerOption::UnifiedDiff => {
            let temp_dir = tempdir()?;
            let orig_path = temp_dir.path().join("original.rs");
            let expanded_path = temp_dir.path().join("expanded.rs");

            fs::write(&orig_path, &unexpanded_source)?;
            fs::write(&expanded_path, &expanded_source)?;

            let output = Command::new("diff")
                .arg("-u")
                .arg(orig_path)
                .arg(expanded_path)
                .output()?;

            io::stdout().write_all(&output.stdout)?;
        }
        ViewerOption::ExternalViewer => {
            let tool = Select::new(
                "Select external diff tool:",
                vec!["diff", "sdiff", "git diff", "vimdiff", "code -d", "custom"],
            )
            .prompt()?;

            let temp_dir = tempdir()?;
            let orig_path = temp_dir.path().join("original.rs");
            let expanded_path = temp_dir.path().join("expanded.rs");

            fs::write(&orig_path, &unexpanded_source)?;
            fs::write(&expanded_path, &expanded_source)?;

            let command = if tool == "custom" {
                Text::new("Enter custom diff command (use $ORIG and $EXPANDED for file paths):")
                    .prompt()?
            } else {
                tool.to_string()
            };

            let orig = "$ORIG";
            let expanded = "$EXPANDED";
            let orig_path = orig_path.to_str().unwrap();
            let expanded_path = expanded_path.to_str().unwrap();
            let contains_orig = command.contains(orig);
            let contains_expanded = command.contains(expanded);

            if contains_orig != contains_expanded {
                return Err(anyhow!(
                    "Command must contain both $ORIG and $EXPANDED or neither"
                ));
            }
            let args_present = contains_orig && contains_expanded;

            let parts: Vec<&str> = command.split_whitespace().collect();
            // eprintln!("Parts: {:?}", parts);

            if parts.is_empty() {
                return Err(anyhow!("Empty command"));
            }

            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                for arg in &parts[1..] {
                    cmd.arg(arg);
                }
            }
            if !args_present {
                cmd.arg(&orig_path).arg(&expanded_path);
            }

            eprintln!("Executing command: {cmd:#?}");

            let status = cmd.status()?;

            if !status.success() {
                eprintln!("External viewer exited with non-zero status: {}", status);
            }

            // Keep files temporarily for the user
            println!(
                "Original file: {}\nExpanded file: {}",
                orig_path, expanded_path
            );
            println!("These temporary files will be removed when you exit this program.");

            // Wait for user acknowledgment
            println!("Press Enter to continue...");
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;
        }
    }

    Ok(())
}

/// Run cargo-expand on the input file and return the expanded output
fn run_cargo_expand(input_path: &Path, theme: &str) -> Result<String> {
    // Create a temporary directory for the cargo project
    let temp_dir = tempdir()?;
    let project_dir = temp_dir.path();

    // Create minimal project structure
    fs::create_dir(project_dir.join("src"))?;

    // Copy input file to src/main.rs
    let target_path = project_dir.join("src/main.rs");
    fs::copy(input_path, &target_path)?;

    // Create minimal Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "expand-temp"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Run cargo-expand
    let output = Command::new("cargo")
        .current_dir(project_dir)
        .args(["expand", "--theme", theme])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "cargo-expand failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Display original and expanded code side by side
fn display_side_by_side(original: &str, expanded: &str, max_width: u16) -> Result<()> {
    let diff = create_side_by_side_diff(original, expanded, max_width.into());
    println!("{}", diff);
    Ok(())
}

/// Detect terminal width to optimize side-by-side display
fn detect_terminal_width() -> u16 {
    match terminal::size() {
        Ok((width, _)) => {
            // Use a bit less than half the terminal width to account for borders and spacing
            // (width as f32 * 0.45) as u16
            ((width - 26) / 2) as u16
        }
        Err(_) => 80, // Default if we can't detect
    }
}

/// Alternate implementation that runs cargo-expand on an existing Cargo project
fn run_cargo_expand_on_project(project_path: &Path, theme: &str) -> Result<Output> {
    // Run cargo-expand in the project directory
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(["expand", "--theme", theme])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "cargo-expand failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output)
}
