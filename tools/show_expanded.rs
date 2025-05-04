/*[toml]
[dependencies]
anyhow = "1.0.96"
atty = "0.2.14"
crossterm = "0.29"
inquire = "0.7.5"
# quote = "1.0.38"
side-by-side-diff = "0.1.2"
tempfile = "3.17.1"
thag_proc_macros = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop" }
# thag_proc_macros = { path = "/Users/donf/projects/thag_rs/thag_proc_macros" }
*/

/// Useful front-end for `thag --cargo <script> --expand`, which in turn uses `cargo-expand` to show the macro expansion
/// of a user script. This tool provides a user-friendly interface to select the script to analyse and to view the expanded code,
/// either on its own or side-by-side with the original script using a choice of diff tools.
///
/// # Purpose
/// Display the expanded code of a user script on its own or side-by-side with the original script using a choice of diff tools.
///
/// # Categories
/// diagnosis, technique, thag_front_ends, tools
use anyhow::{anyhow, Context, Result};
use crossterm::terminal;
use side_by_side_diff::create_side_by_side_diff;
use std::{
    env::args,
    fs,
    io::{self, Error, ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tempfile::tempdir;
use thag_proc_macros::{file_navigator, tool_errors};

tool_errors! {}
file_navigator! {}

/// Available viewing options for expanded code
enum ViewerOption {
    SideBySide,
    SideBySideCustomWidth,
    ExpandedOnly,
    UnifiedDiff,
    ExternalViewer,
    Exit,
}

impl std::fmt::Display for ViewerOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SideBySide => write!(f, "Side-by-side view (auto width)"),
            Self::SideBySideCustomWidth => write!(f, "Side-by-side view (custom width)"),
            Self::ExpandedOnly => write!(f, "Expanded code only"),
            Self::UnifiedDiff => write!(f, "Unified diff"),
            Self::ExternalViewer => write!(f, "External diff tool"),
            Self::Exit => write!(f, "Exit"),
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
    // Directly call expand_script
    expand_script()
}

#[allow(clippy::too_many_lines)]
fn expand_script() -> Result<()> {
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
                .map_err(|e| ToolError::ThreadSafe(format!("Failed to select file: {e}",).into()))?
        }
    };
    if !input_path.exists() {
        return Err(anyhow!("File not found: {}", input_path.display()));
    }

    // Load source files - do this once
    let unexpanded_source = fs::read_to_string(&input_path)
        .map_err(|err| Error::new(ErrorKind::Other, format!("Failed to read file: {err}")))?;

    // Get expanded source using cargo-expand - this can be slow, so only do it once
    let start = std::time::Instant::now();
    let expanded_source = run_cargo_expand(&input_path)
        .context("Failed to run cargo-expand. Is it installed? (cargo install cargo-expand)")?;
    let expand_duration = start.elapsed();

    println!("Macro expansion completed in {expand_duration:.2?}");

    // Create temporary directory and files once
    let temp_dir = tempdir()?;
    let orig_path = temp_dir.path().join("original.rs");
    let expanded_path = temp_dir.path().join("expanded.rs");

    fs::write(&orig_path, &unexpanded_source)?;
    fs::write(&expanded_path, &expanded_source)?;

    // Loop for viewing options
    loop {
        // Let user choose viewing option
        let options = vec![
            ViewerOption::SideBySide,
            ViewerOption::SideBySideCustomWidth,
            ViewerOption::ExpandedOnly,
            ViewerOption::UnifiedDiff,
            ViewerOption::ExternalViewer,
            ViewerOption::Exit,
        ];

        let selection = Select::new("How would you like to view the expanded code?", options)
            .with_help_message(
                "Choose a viewing option for the expanded macros (press Esc to exit)",
            )
            .prompt_skippable()?;

        // If user pressed Esc, exit
        let Some(viewer) = selection else {
            println!("Exiting...");
            return Ok(());
        };

        // Display the expanded code according to the chosen view option
        match viewer {
            ViewerOption::SideBySide => {
                let width = detect_terminal_width_split();
                let unexpanded_truncated = unexpanded_source
                    .lines()
                    .map(|line| line.get(..width as usize).unwrap_or(line))
                    .collect::<Vec<_>>()
                    .join("\n");
                let expanded_truncated = expanded_source
                    .lines()
                    .map(|line| line.get(..width as usize).unwrap_or(line))
                    .collect::<Vec<_>>()
                    .join("\n");
                display_side_by_side(&unexpanded_truncated, &expanded_truncated, width);

                // After viewing, wait for user input
                println!("\nPress Enter to continue...");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
            }
            ViewerOption::SideBySideCustomWidth => {
                let width_input = Text::new("Enter width for each column:")
                    .with_default(&detect_terminal_width_split().to_string())
                    .prompt_skippable()?;

                // If user pressed Esc, go back to viewer selection
                let Some(width_input) = width_input else {
                    continue;
                };

                let width: u16 = width_input.parse().context("Invalid width")?;
                let unexpanded_truncated = unexpanded_source
                    .lines()
                    .map(|line| line.get(..width as usize).unwrap_or(line))
                    .collect::<Vec<_>>()
                    .join("\n");
                let expanded_truncated = expanded_source
                    .lines()
                    .map(|line| line.get(..width as usize).unwrap_or(line))
                    .collect::<Vec<_>>()
                    .join("\n");
                display_side_by_side(&unexpanded_truncated, &expanded_truncated, width);

                // After viewing, wait for user input
                println!("\nPress Enter to continue...");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
            }
            ViewerOption::ExpandedOnly => {
                println!("{expanded_source}");

                // After viewing, wait for user input
                println!("\nPress Enter to continue...");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
            }
            ViewerOption::UnifiedDiff => {
                let output = Command::new("diff")
                    .arg("-u")
                    .arg(&orig_path)
                    .arg(&expanded_path)
                    .output()?;

                io::stdout().write_all(&output.stdout)?;

                // After viewing, wait for user input
                println!("\nPress Enter to continue...");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
            }
            ViewerOption::ExternalViewer => {
                let tools = vec!["diff", "sdiff", "git diff", "vimdiff", "code -d", "custom"];

                let tool_selection = Select::new("Select external diff tool:", tools)
                    .with_help_message(
                        "Choose a diff tool to view the files (press Esc to go back)",
                    )
                    .prompt_skippable()?;

                // If user pressed Esc, go back to viewer selection
                let Some(tool) = tool_selection else {
                    continue;
                };

                let command = if tool == "custom" {
                    let input = Text::new(
                        "Enter custom diff command (use $ORIG and $EXPANDED for file paths):",
                    )
                    .prompt_skippable()?;

                    // If user pressed Esc, go back to tool selection
                    match input {
                        Some(cmd) => cmd,
                        None => continue,
                    }
                } else if tool == "sdiff" {
                    let width = detect_terminal_width_full();
                    format!("sdiff -w {width}")
                } else {
                    tool.to_string()
                };

                let orig = "$ORIG";
                let expanded = "$EXPANDED";
                let orig_path_str = orig_path.to_str().unwrap();
                let expanded_path_str = expanded_path.to_str().unwrap();
                let contains_orig = command.contains(orig);
                let contains_expanded = command.contains(expanded);

                if contains_orig != contains_expanded {
                    eprintln!("Error: Command must contain both $ORIG and $EXPANDED or neither");
                    continue;
                }
                let args_present = contains_orig && contains_expanded;

                let parts: Vec<&str> = command.split_whitespace().collect();

                if parts.is_empty() {
                    eprintln!("Error: Empty command");
                    continue;
                }

                let mut cmd = Command::new(parts[0]);
                if parts.len() > 1 {
                    for arg in &parts[1..] {
                        let arg_replaced = if args_present {
                            arg.replace(orig, orig_path_str)
                                .replace(expanded, expanded_path_str)
                        } else {
                            (*arg).to_string()
                        };
                        cmd.arg(arg_replaced);
                    }
                }
                if !args_present {
                    cmd.arg(orig_path_str).arg(expanded_path_str);
                }

                eprintln!("Executing command: {cmd:#?}");

                let status = cmd.status()?;

                // For diff tools, exit code 1 means differences found (which is expected)
                // Only report failure for other exit codes, or for tools that aren't diff-related
                if !status.success()
                    && (status.code() != Some(1)
                        || !(tool == "diff" || tool == "sdiff" || tool == "git diff"))
                {
                    eprintln!("External viewer exited with non-zero status: {status}");
                }

                // Show file paths for user reference
                println!("Original file: {orig_path_str}\nExpanded file: {expanded_path_str}");

                // After viewing, wait for user input
                println!("\nPress Enter to continue...");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
            }
            ViewerOption::Exit => {
                println!("Exiting...");
                return Ok(());
            }
        }
    }
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

    eprintln!(
        "Running command {} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
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
fn display_side_by_side(original: &str, expanded: &str, max_width: u16) {
    let diff = create_side_by_side_diff(original, expanded, max_width.into());
    println!("{diff}");
}

fn detect_terminal_width_full() -> u16 {
    match terminal::size() {
        Ok((width, _)) => width,
        Err(_) => 160, // Default if we can't detect
    }
}

/// Detect terminal width to optimize side-by-side display
fn detect_terminal_width_split() -> u16 {
    match terminal::size() {
        Ok((width, _)) => {
            // Use a bit less than half the terminal width to account for borders and spacing
            (width - 26) / 2
        }
        Err(_) => 80, // Default if we can't detect
    }
}
