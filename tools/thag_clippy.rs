/*[toml]
[dependencies]
atty = "0.2.14"
colored = "2.1.0"
inquire = "0.7.5"
*/
/// `thag` prompted front-end command to run `clippy` on scripts. It is recommended to compile this to an executable with -x.
/// Prompts the user to select a Rust script and one or more Clippy lints to run against the script's generated project, and
/// and invokes `thag` with the --cargo option to run it.
//# Purpose: A user-friendly interface to the `thag` `--cargo` option specifically for running `cargo clippy` on a script.
//# Categories: technique, thag_front_ends, tools
use colored::Colorize;
use inquire::{Confirm, MultiSelect, Select};
use std::{env, error::Error, path::PathBuf, process::Command};

#[derive(Debug, Clone)] // Added Clone
struct ClippyLintGroup {
    name: &'static str,
    description: &'static str,
    level: LintLevel, // New: for coloring and grouping
}

#[derive(Debug, Clone)]
enum LintLevel {
    Basic,      // cargo, correctness
    Style,      // style, complexity
    Extra,      // nursery, pedantic
    Strict,     // restriction, suspicious
    Deprecated, // deprecated
}

impl LintLevel {
    const fn color(&self) -> colored::Color {
        match self {
            Self::Basic => colored::Color::Green,
            Self::Style => colored::Color::Cyan,
            Self::Extra => colored::Color::Yellow,
            Self::Strict => colored::Color::Red,
            Self::Deprecated => colored::Color::BrightBlack,
        }
    }
}

impl ClippyLintGroup {
    const fn new(name: &'static str, description: &'static str, level: LintLevel) -> Self {
        Self {
            name,
            description,
            level,
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::new(
                "cargo",
                "Checks for common mistakes when using Cargo",
                LintLevel::Basic,
            ),
            Self::new(
                "complexity",
                "Checks for code that might be too complex",
                LintLevel::Style,
            ),
            Self::new(
                "correctness",
                "Checks for common programming mistakes",
                LintLevel::Basic,
            ),
            Self::new(
                "nursery",
                "New lints that are still under development",
                LintLevel::Extra,
            ),
            Self::new(
                "pedantic",
                "Stricter checks for code quality",
                LintLevel::Extra,
            ),
            Self::new(
                "perf",
                "Checks that impact runtime performance",
                LintLevel::Style,
            ),
            Self::new("restriction", "Highly restrictive lints", LintLevel::Strict),
            Self::new("style", "Checks for common style issues", LintLevel::Style),
            Self::new(
                "suspicious",
                "Checks for suspicious code constructs",
                LintLevel::Strict,
            ),
            Self::new(
                "deprecated",
                "Previously deprecated lints",
                LintLevel::Deprecated,
            ),
        ]
    }

    fn to_arg(&self) -> String {
        format!("clippy::{}", self.name)
    }

    fn format_for_display(&self) -> String {
        format!(
            "{}: {}",
            self.name.color(self.level.color()).bold(),
            self.description
        )
    }
}

struct FileNavigator {
    current_dir: PathBuf,
    history: Vec<PathBuf>,
}

impl FileNavigator {
    fn new() -> Self {
        Self {
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            history: Vec::new(),
        }
    }

    fn list_items(&self) -> Vec<String> {
        let mut items = vec!["..".to_string()]; // Parent directory

        // Add directories
        let mut dirs: Vec<_> = std::fs::read_dir(&self.current_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.')) // Optional: hide hidden dirs
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        dirs.sort();
        items.extend(dirs.into_iter().map(|d| format!("📁 {d}")));

        // Add .rs files
        let mut files: Vec<_> = std::fs::read_dir(&self.current_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                    && entry.path().extension().is_some_and(|ext| ext == "rs")
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
}

fn select_script() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut navigator = FileNavigator::new();

    loop {
        let items = navigator.list_items();

        let selection = Select::new(
            &format!("Current dir: {}", navigator.current_dir.display()),
            items,
        )
        .with_help_message("↑↓ navigate, Enter select")
        .with_page_size(20)
        .prompt()?;

        if let Some(script_path) = navigator.navigate(&selection) {
            if Confirm::new(&format!("Use {}?", script_path.display()))
                .with_default(true)
                .prompt()?
            {
                return Ok(script_path);
            }
        }
    }
}

fn select_lint_groups() -> Result<Vec<ClippyLintGroup>, Box<dyn Error>> {
    let lint_groups = ClippyLintGroup::all();
    let formatted_groups: Vec<String> = lint_groups
        .iter()
        .map(ClippyLintGroup::format_for_display)
        .collect();

    let selections = MultiSelect::new("Select Clippy lint groups:", formatted_groups.clone())
        .with_help_message("Space to select/unselect, Enter to confirm")
        .with_default(&[2, 4]) // Default to correctness and pedantic
        .prompt()?;

    // Get the selected groups
    let selected_groups: Vec<ClippyLintGroup> = lint_groups
        .into_iter()
        .enumerate()
        .filter(|(i, _)| selections.contains(&formatted_groups[*i]))
        .map(|(_, group)| group)
        .collect();

    Ok(selected_groups)
}

fn get_script_mode() -> ScriptMode {
    if atty::isnt(atty::Stream::Stdin) {
        // We're receiving input via pipe
        ScriptMode::Stdin
    } else if std::env::args().len() > 1 {
        // We have command line arguments (likely a file path)
        ScriptMode::File
    } else {
        // Interactive mode
        ScriptMode::Interactive
    }
}

enum ScriptMode {
    Stdin,
    File,
    Interactive,
}

fn main() -> Result<(), Box<dyn Error>> {
    let script_path = match get_script_mode() {
        ScriptMode::Stdin => {
            eprintln!("This tool cannot be run with stdin input. Please provide a file path or run interactively.");
            std::process::exit(1);
        }
        ScriptMode::File => {
            // Get the file path from args
            let args: Vec<String> = std::env::args().collect();
            PathBuf::from(args[1].clone())
        }
        ScriptMode::Interactive => {
            // Use the file selector
            select_script()?
        }
    };

    println!("\n{}", "Select lint groups to apply:".bold());
    match select_lint_groups() {
        Ok(selected_groups) => {
            if selected_groups.is_empty() {
                println!(
                    "{}",
                    "\nNo lint groups selected. Using default Clippy checks.".yellow()
                );
                println!("\n{}", "Running command:".bold());
                println!(
                    "thag --cargo {} -- clippy",
                    script_path.display().to_string().bright_cyan()
                );
            } else {
                // Group selected lints by level
                let mut by_level: Vec<(&str, Vec<&ClippyLintGroup>)> = Vec::new();
                for level in [
                    LintLevel::Basic,
                    LintLevel::Style,
                    LintLevel::Extra,
                    LintLevel::Strict,
                ] {
                    let groups: Vec<_> = selected_groups
                        .iter()
                        .filter(|g| {
                            std::mem::discriminant(&g.level) == std::mem::discriminant(&level)
                        })
                        .collect();
                    if !groups.is_empty() {
                        by_level.push((
                            match level {
                                LintLevel::Basic => "Basic checks",
                                LintLevel::Style => "Style checks",
                                LintLevel::Extra => "Extra checks",
                                LintLevel::Strict => "Strict checks",
                                LintLevel::Deprecated => "Deprecated",
                            },
                            groups,
                        ));
                    }
                }

                println!("\n{}", "Selected lint groups:".bold());
                for (level_name, groups) in &by_level {
                    println!("  {}", level_name.bold());
                    for group in groups {
                        println!("    • {}", group.name.color(group.level.color()));
                    }
                }

                // Construct the warning flags
                let warn_flags: Vec<String> = selected_groups
                    .iter()
                    .map(|group| format!("-W{}", group.to_arg()))
                    .collect();

                // Display the command
                let command = format!(
                    "thag --cargo {} -- clippy -- {}",
                    script_path.display(),
                    warn_flags.join(" ")
                );
                println!("\n{}", "Command to run:".bold());
                println!("{}", command.cyan());

                let script_path = script_path.display().to_string();
                // Execute the command
                let mut thag_args = vec!["--cargo", &script_path, "--", "clippy", "--"];
                thag_args.extend(warn_flags.iter().map(String::as_str));

                let status = Command::new("thag").args(&thag_args).status()?;

                if !status.success() {
                    eprintln!("{}", "Clippy check failed".red());
                    return Err("Clippy check failed".into());
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Error selecting lint groups: {e}").red());
            return Err(e);
        }
    }

    Ok(())
}
