/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] } # For optional theming of `inquire`
*/

/// Early prototype of a front-end prompt for `thag`.
//# Purpose: Ultimately, to provide a prompt-driven front-end to the `thag` command.
//# Categories: prototype, thag_front_ends, tools
use inquire::{set_global_render_config, MultiSelect};
// For optional theming of `inquire`
use std::process::Command;
use thag_styling::{file_navigator, themed_inquire_config, Styleable};

file_navigator! {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️  Prompted Thag");
    println!("================================\n");

    // For optional theming of `inquire`
    set_global_render_config(themed_inquire_config());

    let mut navigator = FileNavigator::new();

    // Step 1: Select a file
    println!("Step 1: Select a script file");
    let selected_file = match select_file(&mut navigator, Some("rs"), false) {
        Ok(path) => path,
        Err(_) => {
            println!("No file selected. Exiting.");
            return Ok(());
        }
    };

    println!("Selected file: {}\n", selected_file.display());

    // let script_path = Text::new("Script path:")
    //     .with_help_message("Path to your Rust script")
    //     .prompt()?;

    let options = MultiSelect::new(
        "Select options:",
        vec![
            "Check only (-c)",
            "Expand (-X)",
            "Verbose (-v)",
            // ... other options
        ],
    )
    .prompt()?;

    let mut cmd = Command::new("thag");
    for opt in options {
        cmd.arg(match opt {
            "Check only (-c)" => "-c",
            "Expand (-X)" => "-X",
            "Verbose (-v)" => "-v",
            // ... other mappings
            _ => continue,
        });
    }
    cmd.arg(selected_file);

    let mut cmd_str = format!("{cmd:?}");
    cmd_str.retain(|c| c != '"');
    println!("Running: {}", cmd_str.info());
    cmd.status()?;

    Ok(())
}
