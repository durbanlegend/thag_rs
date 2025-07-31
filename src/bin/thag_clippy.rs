/*[toml]
[dependencies]
atty = "0.2.14"
colored = "2.1.0"
inquire = "0.7.5"
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
*/
/// `thag` prompted front-end command to run `clippy` on scripts.
///
/// Prompts the user to select a Rust script and one or more Clippy lints to run against the
/// script's generated project, and invokes `thag` with the --cargo option to run it.
//# Purpose: A user-friendly interface to the `thag` `--cargo` option specifically for running `cargo clippy` on a script.
//# Categories: technique, thag_front_ends, tools
//# Usage: thag_clippy [script_path] or thag_clippy (interactive mode)
use colored::Colorize;
use inquire::{set_global_render_config, Confirm, MultiSelect};
use std::{env, error::Error, path::PathBuf, process::Command};
use thag_proc_macros::file_navigator;
use thag_rs::{
    auto_help, cvprtln, help_system::check_help_and_exit, styling::themed_inquire_config, Role, V,
};

file_navigator! {}

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

fn select_script() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut navigator = FileNavigator::new();

    loop {
        let items = navigator.list_items(Some("rs"), false, false);

        let selection = Select::new(
            &format!("Current dir: {}", navigator.current_dir.display()),
            items,
        )
        .with_help_message("↑↓ navigate, Enter select")
        .with_page_size(20)
        .prompt()?;

        if let NavigationResult::SelectionComplete(script_path) =
            navigator.navigate(&selection, false)
        {
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
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_clippy");
    check_help_and_exit(&help);

    set_global_render_config(themed_inquire_config());

    let script_path = match get_script_mode() {
        ScriptMode::Stdin => {
            cvprtln!(Role::Error, V::QQ, "This tool cannot be run with stdin input. Please provide a file path or run interactively.");
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

    cvprtln!(Role::Heading1, V::QQ, "\nSelect lint groups to apply:");
    match select_lint_groups() {
        Ok(selected_groups) => {
            if selected_groups.is_empty() {
                cvprtln!(
                    Role::Warning,
                    V::QQ,
                    "{}",
                    "\nNo lint groups selected. Using default Clippy checks."
                );
                cvprtln!(Role::Heading3, V::QQ, "\n{}", "Running command:".bold());
                cvprtln!(
                    Role::Code,
                    V::QQ,
                    "thag --cargo {} -- clippy",
                    script_path.display()
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

                cvprtln!(
                    Role::Heading3,
                    V::QQ,
                    "\n{}",
                    "Selected lint groups:".bold()
                );
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
                cvprtln!(Role::Heading3, V::QQ, "\n{}", "Command to run:".bold());
                cvprtln!(Role::Code, V::QQ, "{command}");

                let script_path = script_path.display().to_string();
                // Execute the command
                let mut thag_args = vec!["--cargo", &script_path, "--", "clippy", "--"];
                thag_args.extend(warn_flags.iter().map(String::as_str));

                let status = Command::new("thag").args(&thag_args).status()?;

                if !status.success() {
                    cvprtln!(Role::Error, V::QQ, "Clippy check failed");
                    return Err("Clippy check failed".into());
                }
            }
        }
        Err(e) => {
            cvprtln!(Role::Error, V::QQ, "Error selecting lint groups: {e}");
            return Err(e);
        }
    }

    Ok(())
}
