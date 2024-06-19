/*[toml]
[dependencies]
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
*/

/// Proof of concept of GPT-written color_println macro.
use owo_colors::colors::css::Red;
use owo_colors::{OwoColorize, Style};

#[macro_export]
macro_rules! color_println {
    ($style:expr, $($arg:tt)*) => {{
        let binding = format!("{}", format_args!($($arg)*));
        let styled_text = binding.style($style);
        println!("{}", styled_text);
    }};
}

fn main() {
    let my_content = "colorized";

    // Define the style
    let my_style = Style::new().fg::<Red>().bold();

    // Use the color_println! macro
    color_println!(my_style, "My {} message", my_content);
}
