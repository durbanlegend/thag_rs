/// Writes text from the system clipboard to `stdout`.
///
/// A cross-platform equivalent to macOS's `pbpaste`. See also `src/bin/thag_copy.rs`.
/// May not work with `pbcopy` in Linux environments owing to the X11 and Wayland requiring the copying
/// app to be open when pasting from the system clipboard. See `arboard` Readme.
//# Purpose: Utility
//# Categories: tools
#[cfg(feature = "clipboard")]
use arboard::Clipboard;

#[cfg(feature = "clipboard")]
use std::io::{self, Write};

#[cfg_attr(not(feature = "clipboard"), allow(clippy::unnecessary_wraps))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "clipboard"))]
    {
        eprintln!("Error: thag_paste requires the 'clipboard' feature to be enabled");
        eprintln!("Please run with: cargo run --bin thag_paste --features clipboard");
        Err("Missing clipboard feature".into())
    }

    #[cfg(feature = "clipboard")]
    {
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
                    eprintln!("Failed to write to stdout: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to read clipboard text: {}", e);
                std::process::exit(1);
            }
        }
        Ok(())
    }
}
