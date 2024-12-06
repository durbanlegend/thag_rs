/*[toml]
[dependencies]
inquire = "0.7.5"
*/

/// Early prototype of prompting front-end for `thag`.
//# Purpose: Ultimately, to provide a prompt-driven fromt-end to the `thag` command.
//# Categories: prototype, tools
use inquire::{MultiSelect, Text};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let script_path = Text::new("Script path:")
        .with_help_message("Path to your Rust script")
        .prompt()?;

    let options = MultiSelect::new(
        "Select options:",
        vec![
            "Check only (-c)",
            "Edit (-e)",
            "Verbose (-v)",
            // ... other options
        ],
    )
    .prompt()?;

    let mut cmd = Command::new("thag");
    for opt in options {
        cmd.arg(match opt {
            "Check only (-c)" => "-c",
            "Edit (-e)" => "-e",
            "Verbose (-v)" => "-v",
            // ... other mappings
            _ => continue,
        });
    }
    cmd.arg(script_path);

    println!("Running: {:?}", cmd);
    cmd.status()?;

    Ok(())
}
