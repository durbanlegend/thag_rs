   use arboard::Clipboard;

   fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
       let mut clipboard = Clipboard::new()?;
       clipboard.set_text(text)?;
       Ok(())
   }

   fn main() -> Result<(), arboard::Error> {
       let text_to_copy = "Hello from Rust!";
       copy_to_clipboard(text_to_copy)?;
       println!("Text copied to clipboard.");
       Ok(())
   }
