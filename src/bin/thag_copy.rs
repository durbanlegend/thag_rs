/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto" }
*/

/// Copies text input from `stdin` to the system clipboard.
///
/// A cross-platform equivalent to macOS's `pbcopy`. See also `src/bin/thag_paste.rs`.
/// May not work in Linux environments owing to the X11 and Wayland requiring the copying
/// app to be open when pasting from the system clipboard. See `arboard` Readme.
//# Purpose: Utility
//# Categories: tools
use arboard::Clipboard;
use thag_common::{auto_help, help_system::check_help_and_exit};

use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    // Read all input from stdin
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    // Don't copy empty input
    if input.trim().is_empty() {
        eprintln!("No input provided to copy to clipboard");
        std::process::exit(1);
    }

    // Copy to clipboard
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(&input)?;

    println!("Text copied to clipboard successfully");

    Ok(())
}
