/// Copies text input from `stdin` to the system clipboard.
///
/// A cross-platform equivalent to macOS's `pbcopy`. See also `src/bin/thag_paste.rs`.
/// May not work in Linux environments owing to the X11 and Wayland requiring the copying
/// app to be open when pasting from the system clipboard. See `arboard` Readme.
//# Purpose: Utility
//# Categories: tools
#[cfg(feature = "clipboard")]
use arboard::Clipboard;

#[cfg(feature = "clipboard")]
use std::io::Read;

#[cfg_attr(not(feature = "clipboard"), allow(clippy::unnecessary_wraps))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "clipboard"))]
    {
        eprintln!("Error: thag_copy requires the 'clipboard' feature to be enabled");
        eprintln!("Please run with: cargo run --bin thag_copy --features clipboard");
        Err("Missing clipboard feature".into())
    }

    #[cfg(feature = "clipboard")]
    {
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
}
