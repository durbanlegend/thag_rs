use inquire::{Select, Text};
use std::{env, path::PathBuf, process::Command};

#[derive(Debug)]
struct FileNavigator {
    current_dir: PathBuf,
    history: Vec<PathBuf>,
}

impl FileNavigator {
    fn new() -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            current_dir,
            history: Vec::new(),
        }
    }

    fn list_items(&self) -> Vec<String> {
        let mut items = vec!["..".to_string()]; // Parent directory

        // Add directories first
        let mut dirs: Vec<_> = self
            .current_dir
            .read_dir()
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        dirs.sort();
        items.extend(dirs.into_iter().map(|d| format!("📁 {}", d)));

        // Then add .rs files
        let mut files: Vec<_> = self
            .current_dir
            .read_dir()
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                    && entry
                        .path()
                        .extension()
                        .map(|ext| ext == "rs")
                        .unwrap_or(false)
            })
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();
        items.extend(files.into_iter().map(|f| format!("📄 {}", f)));

        items
    }

    fn navigate(&mut self, selection: &str) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        if selection == ".." {
            if let Some(parent) = self.current_dir.parent() {
                self.history.push(self.current_dir.clone());
                self.current_dir = parent.to_path_buf();
            }
            Ok(None)
        } else {
            let clean_name = selection.trim_start_matches(['📁', '📄', ' ']);
            let new_path = self.current_dir.join(clean_name);

            if new_path.is_dir() {
                self.history.push(self.current_dir.clone());
                self.current_dir = new_path;
                Ok(None)
            } else {
                Ok(Some(new_path))
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
        .with_help_message("Navigate to a Rust script (↑↓ to move, Enter to select)")
        .prompt()?;

        if let Some(script_path) = navigator.navigate(&selection)? {
            return Ok(script_path);
        }
    }
}

#[derive(Debug)]
struct CargoCommand {
    subcommand: String,
    args: Vec<String>,
}

impl CargoCommand {
    fn prompt() -> Result<Self, Box<dyn std::error::Error>> {
        let subcommands = vec![
            "tree", "check", "build", "doc", "test", "clippy",
            // Add more as needed
        ];

        let subcommand = Select::new("Cargo subcommand:", subcommands.to_vec())
            .with_help_message("Select cargo subcommand to run")
            .prompt()?;

        let args = Text::new("Additional arguments:")
            .with_help_message("e.g., '-i syn' for tree, '--all-features' for check")
            .prompt()?;

        Ok(Self {
            subcommand: subcommand.to_string(),
            args: args.split_whitespace().map(String::from).collect(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Select a Rust script to analyze:");
    let script_path = select_script()?;

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
