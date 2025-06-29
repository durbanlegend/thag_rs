/// An early exploration of message colouring, GPT-generated.
/// This one uses basic Ansi 16 colours. Try it on dark vs light
/// backgrounds to see how some of the colours change.
//# Purpose: May be of use to some. Demo featured crates.
//# Categories: crates, exploration
use crossterm::{
    cursor::{MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use owo_colors::OwoColorize;
use std::io::{stdout, Write};
use termbg::Theme;

pub fn clear_screen() {
    let mut out = stdout();
    // out.execute(Hide).unwrap();
    out.execute(Clear(ClearType::All)).unwrap();
    out.execute(MoveTo(0, 0)).unwrap();
    out.execute(Show).unwrap();
    out.flush().unwrap();
}

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

            (MessageType::Warning, Theme::Dark) => owo_colors::Style::new().magenta().bold(),
            (MessageType::Warning, Theme::Light) => owo_colors::Style::new().yellow().bold(),

            (MessageType::Emphasis, Theme::Dark) => owo_colors::Style::new().cyan().bold(),
            (MessageType::Emphasis, Theme::Light) => owo_colors::Style::new().cyan().bold(),

            (MessageType::OuterPrompt, Theme::Dark) => owo_colors::Style::new().white(),
            (MessageType::OuterPrompt, Theme::Light) => owo_colors::Style::new().blue(),

            (MessageType::InnerPrompt, _) => owo_colors::Style::new().purple(),

            (MessageType::Normal, _) => owo_colors::Style::new().green(),

            (MessageType::Debug, _) => owo_colors::Style::new().cyan(),
        }
    }
}

fn main() {
    // Detect terminal background color theme
    // let theme = get_theme().expect("Failed to detect terminal theme");
    let timeout = std::time::Duration::from_millis(100);

    // debug!("Check terminal background color");
    let theme = termbg::theme(timeout).expect("Failed to detect terminal theme");
    clear_screen();

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
