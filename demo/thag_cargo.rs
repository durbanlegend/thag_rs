/*[toml]
[dependencies]
atty = "0.2.14"
inquire = "0.7.5"
rustix = "0.38.42"
tempfile = "3.14.0"
*/
/// `thag` prompted front-end command to run Cargo commands on scripts. It is recommended to compile this to an executable with -x.
/// Prompts the user to select a Rust script and a cargo command to run against the script's generated project, and
/// and invokes `thag` with the --cargo option to run it.
//# Purpose: A user-friendly interface to the `thag` `--cargo` option.
//# Categories: technique, tools
use inquire::{Confirm, Select, Text};
use rustix::path::Arg;
use std::{env, error::Error, path::PathBuf, process::Command};

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
    println!("Select a Rust script to analyze:");

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
                description: "Run clippy lints",
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

    let status = Command::new("thag").args(&args).status()?;

    if !status.success() {
        return Err("thag command failed".into());
    }

    Ok(())
}
