/// Partially worked-out solution to colouring and styling messages based on the level of
/// colour support of the current terminal and whether a light or dark theme is currently
/// selected. This was the interim result of a dialog with ChatGPT to figure out a solution
/// that met my needs.
//# Purpose: Demo use of `strum` crate to get enum variant from string, and of AI-generated code.
use owo_colors::{OwoColorize, Style};
use std::str::FromStr;
use strum::{EnumIter, EnumString};

#[derive(Debug, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
enum MessageStyle {
    AnsiLightError,
    AnsiLightWarning,
    // Add more variants for all possible combinations
    // For example: AnsiDarkError, AnsiDarkWarning, XtermLightError, etc.
}

impl MessageStyle {
    fn style(&self) -> Style {
        match self {
            MessageStyle::AnsiLightError => Style::new().red().bold(),
            MessageStyle::AnsiLightWarning => Style::new().yellow(),
            // Match other variants with appropriate styles
        }
    }
}

fn main() {
    // Example: Choose a message style based on runtime conditions
    let color_support = "ansi"; // Example: retrieved from runtime
    let theme = "light"; // Example: retrieved from runtime
    let level = "error"; // Example: retrieved from runtime

    let style_string = format!("{}_{}_{}", color_support, theme, level);
    let message_style = MessageStyle::from_str(&style_string);
    eprintln!(
        "Called from_str on {}_{}_{}, found {message_style:#?}",
        &color_support, &theme, &level,
    );
    if let Ok(message_style) = message_style {
        let style = message_style.style();
        println!("{}", "Message text".style(style));
    } else {
        eprintln!("Invalid message style");
    }
}
