use owo_colors::{OwoColorize, Style};

// Define the trait for getting the message style
trait GetMessageStyle {
    fn get_style(&self) -> Style;
}

// Define enums for different color support and terminal themes
#[derive(Debug)]
enum Ansi16Theme {
    Light,
    Dark,
}

#[derive(Debug)]
enum Xterm256Theme {
    Light,
    Dark,
}

// Implement GetMessageStyle trait for different enums
impl GetMessageStyle for Ansi16Theme {
    fn get_style(&self) -> Style {
        // Define styles for ANSI 16 color support and terminal themes
        match self {
            Ansi16Theme::Light => Style::new(), // Add appropriate styles for light theme...
            Ansi16Theme::Dark => Style::new(),  // Add appropriate styles for dark theme...
        }
    }
}

impl GetMessageStyle for Xterm256Theme {
    fn get_style(&self) -> Style {
        // Define styles for Xterm 256 color support and terminal themes
        match self {
            Xterm256Theme::Light => Style::new(), // Add appropriate styles for light theme...
            Xterm256Theme::Dark => Style::new(),  // Add appropriate styles for dark theme...
        }
    }
}

// Function to select the correct enum based on color support and theme
fn select_message_style(color_support: &str, theme: &str) -> Box<dyn GetMessageStyle> {
    match color_support {
        "ansi16" => match theme {
            "light" => Box::new(Ansi16Theme::Light),
            "dark" => Box::new(Ansi16Theme::Dark),
            _ => panic!("Invalid theme"),
        },
        "xterm256" => match theme {
            "light" => Box::new(Xterm256Theme::Light),
            "dark" => Box::new(Xterm256Theme::Dark),
            _ => panic!("Invalid theme"),
        },
        _ => panic!("Invalid color support"),
    }
}

fn main() {
    // Example usage
    let color_support = "ansi16"; // Example: retrieved from runtime
    let theme = "dark"; // Example: retrieved from runtime

    // Select the correct enum based on color support and theme
    let message_style = select_message_style(color_support, theme);

    // Get the style for the selected enum and use it
    let style = message_style.get_style();
    println!("{}", owo_colors::Style::style(&style, "Message text"));
}
