/*[toml]
[dependencies]
rfd = "0.14.1"
*/

/// Prototype of invoking the Rust formatter programmatically, with the addition of an `rfd`
/// (`Rusty File Dialogs`) cross-platform file chooser to select the file to format. The code
/// for both was AI-generated because I find AI very handy for this kind of grunt work.
//# Purpose: Demo file chooser and calling an external program, in this case the Rust formatter.
//# Categories: crates, technique
use rfd::FileDialog;
use std::error::Error;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    // Check if rustfmt is available
    if Command::new("rustfmt").arg("--version").output().is_ok() {
        let source_file = FileDialog::new()
            .set_title("Pick a .rs file to format")
            .add_filter("rust", &["rs"])
            .set_directory(".")
            .pick_file()
            .ok_or("No file selected")?;

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
