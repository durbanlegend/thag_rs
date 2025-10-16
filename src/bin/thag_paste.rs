/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto" }
*/
/// Writes text from the system clipboard to `stdout`.
///
/// A cross-platform equivalent to macOS's `pbpaste`. See also `src/bin/thag_copy.rs`.
/// May not work with `pbcopy` in Linux environments owing to the X11 and Wayland requiring the copying
/// app to be open when pasting from the system clipboard. See `arboard` Readme.
//# Purpose: Utility
//# Categories: tools
use arboard::Clipboard;
use std::io::{self, Write};
use thag_common::{auto_help, help_system::check_help_and_exit};

fn main() {
    // Check for help first
    let help = auto_help!();
    check_help_and_exit(&help);

    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("Failed to access clipboard: {}", e);
            std::process::exit(1);
        }
    };

    match clipboard.get_text() {
        Ok(text) => {
            if let Err(e) = io::stdout().write_all(text.as_bytes()) {
                eprintln!("Failed to write to stdout: {e}");
                std::process::exit(1);
            }
            if !text.ends_with('\n') {
                if let Err(e) = io::stdout().write(b"\n") {
                    eprintln!("Failed to write line feed to stdout: {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read clipboard text: {}", e);
            std::process::exit(1);
        }
    }
}
