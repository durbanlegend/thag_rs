/*[toml]
[dependencies]
anyhow = "1.0.96"
atty = "0.2.14"
crossterm = "0.28.1"
inquire = "0.7.5"
# quote = "1.0.38"
side-by-side-diff = "0.1.2"
tempfile = "3.17.1"
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/src/proc_macros" }
*/

use anyhow::{anyhow, Context, Result};
use crossterm::terminal;
use side_by_side_diff::create_side_by_side_diff;
use std::{
    env::args,
    fs,
    io::{self, Error, ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};
use tempfile::tempdir;
use thag_proc_macros::file_navigator;

file_navigator! {}

#[derive(Debug)]
pub enum CustomError {
    Dyn(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for CustomError {
    fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self::Dyn(err)
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dyn(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CustomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dyn(e) => Some(&**e),
        }
    }
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

enum ScriptMode {
    Stdin,
    File,
    Interactive,
}

fn get_script_mode() -> ScriptMode {
    if atty::isnt(atty::Stream::Stdin) {
        // We're receiving input via pipe
        ScriptMode::Stdin
    } else if args().len() > 1 {
        // We have command line arguments (likely a file path)
        ScriptMode::File
    } else {
        // Interactive mode
        ScriptMode::Interactive
    }
}

fn main() -> Result<()> {
    let input_path = match get_script_mode() {
        ScriptMode::Stdin => {
            eprintln!("This tool cannot be run with stdin input. Please provide a file path or run interactively.");
            std::process::exit(1);
        }
        ScriptMode::File => {
            // Get the file path from args
            let args: Vec<String> = args().collect();
            PathBuf::from(args[1].clone())
        }
        ScriptMode::Interactive => {
            // Use the file selector
            let mut navigator = FileNavigator::new();
            select_file(&mut navigator, Some("rs"), false)
                .map_err(|e| CustomError::Dyn(format!("Failed to select file: {e}",).into()))?
        }
    };
    if !input_path.exists() {
        return Err(anyhow!("File not found: {}", input_path.display()));
    }

    let viewer = {
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

    let unexpanded_source = match viewer {
        ViewerOption::ExpandedOnly => None,
        _ => Some(fs::read_to_string(&input_path).or_else(|err| {
            Err(Error::new(
                ErrorKind::Other,
                format!("Failed to read file: {}", err),
            ))
        })?),
    };
    let expanded_source = match viewer {
        ViewerOption::ExternalViewer => None,
        _ => Some(run_cargo_expand(&input_path).context(
            "Failed to run cargo-expand. Is it installed? (cargo install cargo-expand)",
        )?),
    };

    // Display the expanded code according to the chosen view option
    match viewer {
        ViewerOption::SideBySide => {
            let width = detect_terminal_width();
            let unexpanded_source = &unexpanded_source
                .unwrap()
                .lines()
                .map(|line| line.get(..width as usize).unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            let expanded_source = expanded_source
                .unwrap()
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
                .unwrap()
                .lines()
                .map(|line| line.get(..width as usize).unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            let expanded_source = expanded_source
                .unwrap()
                .lines()
                .map(|line| line.get(..width as usize).unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            display_side_by_side(&unexpanded_source, &expanded_source, width)?;
        }
        ViewerOption::ExpandedOnly => {
            println!("{}", expanded_source.unwrap());
        }
        ViewerOption::UnifiedDiff => {
            let temp_dir = tempdir()?;
            let orig_path = temp_dir.path().join("original.rs");
            let expanded_path = temp_dir.path().join("expanded.rs");

            fs::write(&orig_path, &unexpanded_source.unwrap())?;
            fs::write(&expanded_path, &expanded_source.unwrap())?;

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

            // Get expanded source using cargo-expand
            let expanded_source = run_cargo_expand(&input_path).context(
                "Failed to run cargo-expand. Is it installed? (cargo install cargo-expand)",
            )?;

            let temp_dir = tempdir()?;
            let orig_path = temp_dir.path().join("original.rs");
            let expanded_path = temp_dir.path().join("expanded.rs");

            fs::write(&orig_path, &unexpanded_source.unwrap())?;
            fs::write(&expanded_path, &expanded_source)?;

            let command = if tool == "custom" {
                Text::new("Enter custom diff command (use $ORIG and $EXPANDED for file paths):")
                    .prompt()?
            } else if tool == "sdiff" {
                let width = match terminal::size() {
                    Ok((width, _)) => width as u16,
                    Err(_) => 160, // Default if we can't detect
                };
                format!("sdiff -w {width}")
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
fn run_cargo_expand(input_path: &Path) -> Result<String> {
    let input_path_str = input_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
    // Run cargo-expand
    let mut binding = Command::new("thag");
    let cmd = binding
        .args(["--cargo", input_path_str, "--", "expand"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    eprintln!("Running command {cmd:?}");
    let output = cmd.output()?;

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
fn run_cargo_expand_on_project(project_path: &Path) -> Result<Output> {
    // Run cargo-expand in the project directory
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(["expand"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "cargo-expand failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output)
}
