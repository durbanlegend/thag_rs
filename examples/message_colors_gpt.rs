/*[toml]
[dependencies]
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
termbg = "0.5.0"
*/

use owo_colors::OwoColorize;
use termbg::Theme;

#[derive(Debug, Clone, Copy)]
enum MessageType {
    Error,
    Warning,
    Emphasis,
    OuterPrompt,
    InnerPrompt,
    Normal,
    Debug,
}

impl MessageType {
    fn color(&self, theme: Theme) -> owo_colors::Style {
        match (self, theme) {
            (MessageType::Error, Theme::Dark) => owo_colors::Style::new().red().bold(),
            (MessageType::Error, Theme::Light) => owo_colors::Style::new().red().bold(),

            (MessageType::Warning, Theme::Dark) => owo_colors::Style::new().yellow(),
            (MessageType::Warning, Theme::Light) => owo_colors::Style::new().yellow(),

            (MessageType::Emphasis, Theme::Dark) => owo_colors::Style::new().cyan().bold(),
            (MessageType::Emphasis, Theme::Light) => owo_colors::Style::new().cyan().bold(),

            (MessageType::OuterPrompt, _) => owo_colors::Style::new().white(),

            (MessageType::InnerPrompt, _) => owo_colors::Style::new().purple(),

            (MessageType::Normal, _) => owo_colors::Style::new().green(),

            (MessageType::Debug, _) => owo_colors::Style::new().cyan().dimmed(),
        }
    }
}

fn main() {
    // Detect terminal background color theme
    // let theme = get_theme().expect("Failed to detect terminal theme");
    let timeout = std::time::Duration::from_millis(100);

    // debug!("Check terminal background color");
    let theme = termbg::theme(timeout).expect("Failed to detect terminal theme");

    // Example: Print messages with different colors based on message type and terminal theme
    println!(
        "{}",
        ("Error message").style(MessageType::Error.color(theme))
    );
    println!(
        "{}",
        ("Warning message").style(MessageType::Warning.color(theme))
    );
    println!(
        "{}",
        ("Emphasis message").style(MessageType::Emphasis.color(theme))
    );
    println!(
        "{}",
        ("OuterPrompt message").style(MessageType::OuterPrompt.color(theme))
    );
    println!(
        "{}",
        ("InnerPrompt message").style(MessageType::InnerPrompt.color(theme))
    );
    println!(
        "{}",
        ("Normal message").style(MessageType::Normal.color(theme))
    );
    println!(
        "{}",
        ("Debug message").style(MessageType::Debug.color(theme))
    );
}
