#[cfg(feature = "clipboard")]
use arboard::Clipboard;

#[cfg_attr(not(feature = "clipboard"), allow(clippy::unnecessary_wraps))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "clipboard"))]
    {
        eprintln!("Error: thag_copy requires the 'clipboard' feature to be enabled");
        eprintln!("Please run with: cargo run --bin thag_copy --features clipboard");
    }

    #[cfg(feature = "clipboard")]
    {
        // Read all input from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        // Don't copy empty input
        if input.trim().is_empty() {
            eprintln!("No input provided to copy to clipboard");
            std::process::exit(1);
        }

        // Copy to clipboard
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(&input)?;

        println!("Text copied to clipboard successfully");
    }

    Ok(())
}
