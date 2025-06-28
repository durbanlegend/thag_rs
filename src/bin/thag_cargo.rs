/*[toml]
[dependencies]
atty = "0.2.14"
inquire = "0.7.5"
thag_proc_macros = { version = "0.1, thag-auto" }
thag_rs = { version = "0.2", path = "../..", default-features = false, features = ["core"] }
*/

/// `thag` prompted front-end command to run Cargo commands on scripts.
///
/// Prompts the user to select a Rust script and a cargo command to run against the
/// script's generated project, and invokes `thag` with the --cargo option to run it.
//# Purpose: A user-friendly interface to the `thag` `--cargo` option.
//# Categories: technique, thag_front_ends, tools
use std::{error::Error, path::PathBuf, process::Command};
use thag_proc_macros::{file_navigator, tool_errors};
use thag_rs::{auto_help, help_system::check_help_and_exit};

tool_errors! {}
file_navigator! {}

#[derive(Clone, Debug)]
struct CargoCommand {
    subcommand: String,
    args: Vec<String>,
}

impl CargoCommand {
    fn prompt() -> Result<Self, Box<dyn std::error::Error>> {
        let subcommands = CargoSubcommand::all();

        // Show subcommands with descriptions
        let formatted_commands: Vec<String> = subcommands
            .iter()
            .enumerate()
            .map(|(i, cmd)| format!("{}. {}: {}", i, cmd.name, cmd.description))
            .collect();

        let subcommand = Select::new("Cargo subcommand:", formatted_commands)
            .with_help_message("Select cargo subcommand to run")
            .prompt()?;

        // Extract index from the selection
        let index = subcommand
            .split('.')
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or("Invalid selection")?;

        let selected_cmd = subcommands[index].clone();

        // Show common arguments for selected command
        println!("\nCommon arguments for {}:", selected_cmd.name);
        for (arg, desc) in &selected_cmd.common_args {
            println!("  {arg} - {desc}");
        }

        let args = Text::new("Additional arguments:")
            .with_help_message("Space-separated arguments (press Tab to see common args)")
            .with_autocomplete(move |input: &str| {
                Ok(selected_cmd
                    .common_args
                    .iter()
                    .map(|(arg, _)| *arg)
                    .filter(|arg| arg.starts_with(input))
                    .map(String::from)
                    .collect())
            })
            .prompt()?;

        Ok(Self {
            subcommand: selected_cmd.name,
            args: args.split_whitespace().map(String::from).collect(),
        })
    }
}

#[derive(Clone, Debug)]
struct CargoSubcommand {
    name: String,
    description: &'static str,
    common_args: Vec<(&'static str, &'static str)>, // (arg, description)
}

impl CargoSubcommand {
    fn all() -> Vec<Self> {
        vec![
            Self {
                name: "tree".to_string(),
                description: "Display dependency tree",
                common_args: vec![
                    ("-i", "Invert dependencies"),
                    ("--target", "Filter dependencies by target"),
                    ("--no-default-features", "Exclude default features"),
                    ("--all-features", "Include all features"),
                    ("-p", "Package to inspect"),
                ],
            },
            Self {
                name: "check".to_string(),
                description: "Check compilation without producing binary",
                common_args: vec![
                    ("--all-features", "Enable all features"),
                    ("--no-default-features", "Disable default features"),
                    ("--features", "Space or comma separated list of features"),
                    ("--verbose", "Use verbose output"),
                ],
            },
            Self {
                name: "clippy".to_string(),
                description: "Run clippy lints (Hint: rather run the `thag_clippy` command for better prompts)",
                common_args: vec![
                    ("--all-targets", "Check all targets"),
                    ("--fix", "Automatically apply lint suggestions"),
                    ("--no-deps", "Skip checking dependencies"),
                    ("-W", "Set lint warnings, e.g., -W clippy::pedantic"),
                ],
            },
            Self {
                name: "doc".to_string(),
                description: "Build documentation",
                common_args: vec![
                    ("--no-deps", "Don't build docs for dependencies"),
                    ("--document-private-items", "Document private items"),
                    ("--open", "Open docs in browser after building"),
                ],
            },
            Self {
                name: "expand".to_string(),
                description: "Show result of macro expansion",
                common_args: vec![("--verbose", "Use verbose output")],
            },
            Self {
                name: "test".to_string(),
                description: "Run tests",
                common_args: vec![
                    ("--no-run", "Compile but don't run tests"),
                    ("--test", "Test name to run"),
                    ("--", "Arguments for test binary"),
                ],
            },
        ]
    }
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
    let help = auto_help!("thag_cargo");
    check_help_and_exit(&help);

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
            let mut navigator = FileNavigator::new();
            select_file(&mut navigator, Some("rs"), false)
                .map_err(|e| ToolError::ThreadSafe(format!("Failed to select file: {e}",).into()))?
        }
    };

    println!("\nConfigure cargo command:");
    let cargo_cmd = CargoCommand::prompt()?;

    // Build thag command
    let mut args = vec![
        "--cargo".to_string(),
        script_path.to_string_lossy().into_owned(),
        "--".to_string(),
        cargo_cmd.subcommand,
    ];
    args.extend(cargo_cmd.args);

    println!("\nRunning: thag {}", args.join(" "));
    // println!("Command to run: {cargo_cmd}");

    let status = Command::new("thag").args(&args).status()?;

    if !status.success() {
        return Err("thag command failed".into());
    }

    Ok(())
}
