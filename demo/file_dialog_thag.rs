/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
*/

/// Demo of invoking the Rust formatter programmatically, using `thag_proc_macros`
/// cross-platform file chooser to select the file to format.
/// Compare with `demo/file_dialog_gui.rs`, which uses the platform's native gui.
//# Purpose: Demo file chooser and calling an external program, in this case the Rust formatter.
//# Categories: crates, technique
use inquire;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use thag_proc_macros::file_navigator;

file_navigator! {}

fn main() -> Result<(), Box<dyn Error>> {
    let mut navigator = FileNavigator::new();

    // Check if rustfmt is available
    if Command::new("rustfmt").arg("--version").output().is_ok() {
        let source_file = match select_file(&mut navigator, Some("rs"), false) {
            Ok(path) => path,
            Err(_) => {
                println!("No file selected. Exiting.");
                return Ok(());
            }
        };

        println!("Selected file: {}\n", source_file.display());

        // Run rustfmt on the source file
        let mut command = Command::new("rustfmt");
        command.arg("--edition");
        command.arg("2021");
        command.arg(&source_file);
        let output = command.output().expect("Failed to run rustfmt");

        if output.status.success() {
            println!("Successfully formatted {source_file:#?} with rustfmt.");
        } else {
            eprintln!(
                "Failed to format {source_file:#?} with rustfmt:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        eprintln!("`rustfmt` not found. Please install it to use this script.");
    }
    Ok(())
}
