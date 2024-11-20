/*[toml]
[dependencies]
crossterm = "0.28.1"
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
strum = { version = "0.26.2", features = ["derive"] }
termbg = "0.5.2"
*/

use crossterm::cursor::{MoveToColumn, Show};
use crossterm::ExecutableCommand;
use owo_colors::colors::css::{Black, DarkOrange, Orange};
use owo_colors::colors::{Blue, Cyan, Green, Red, White};
use owo_colors::{OwoColorize, Style};
use std::io::{stdout, Write};
use strum::{Display, EnumIter, IntoEnumIterator};
use termbg::Theme;

/// An early exploration of the idea of adaptive message colouring according to the terminal theme.

//# Purpose: Demo a simple example of adaptive message colouring, and the featured crates.
//# Categories: crates, exploration, technique
pub trait ThemeStyle {
    fn get_style(&self) -> Style;
}

// Enum for light theme styles
#[derive(Clone, Copy, Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum LightStyle {
    Error,
    Warning,
    Emphasis,
    Info,
    Debug,
}

// Enum for dark theme styles
#[derive(Clone, Copy, Display, EnumIter)]
pub enum DarkStyle {
    Error,
    Warning,
    Emphasis,
    Info,
    Debug,
}

impl ThemeStyle for LightStyle {
    // Get the corresponding color style for the message type
    fn get_style(&self) -> Style {
        match *self {
            LightStyle::Error => Style::new().fg::<Red>().bold(),
            LightStyle::Warning => Style::new().fg::<DarkOrange>().bold(),
            LightStyle::Emphasis => Style::new().fg::<Green>().bold(),
            LightStyle::Info => Style::new().fg::<Black>(),
            LightStyle::Debug => Style::new().fg::<Blue>(),
        }
    }
}

impl ThemeStyle for DarkStyle {
    // Get the corresponding color style for the message type
    fn get_style(&self) -> Style {
        match *self {
            DarkStyle::Error => Style::new().fg::<Red>().bold(),
            DarkStyle::Warning => Style::new().fg::<Orange>().bold(),
            DarkStyle::Emphasis => Style::new().fg::<Green>().bold(),
            DarkStyle::Info => Style::new().fg::<White>(),
            DarkStyle::Debug => Style::new().fg::<Cyan>(),
        }
    }
}

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
pub fn clear_screen() {
    // let mut out = stdout();
    // // out.execute(Clear(ClearType::FromCursorUp)).unwrap();
    // out.execute(MoveToColumn(0)).unwrap();
    // out.execute(Show).unwrap();
    // out.flush().unwrap();
}

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let term = termbg::terminal();
    let theme: Result<Theme, termbg::Error> = termbg::theme(timeout);

    println!("  Term : {:?}", term);

    let supports_color: bool = match theme {
        Ok(_theme) => true,
        Err(ref _e) => false,
    };

    // Get the appropriate style based on the theme
    if supports_color {
        match theme.unwrap() {
            Theme::Light => {
                for variant in LightStyle::iter() {
                    let msg = &format!("My light theme {variant} message");
                    println!("{}", msg.style(variant.get_style()));
                }
            }

            Theme::Dark => {
                for variant in DarkStyle::iter() {
                    let msg = &format!("My dark theme {variant} message");
                    println!("{}", msg.style(variant.get_style()));
                }
            }
        }
    } else {
        println!("My warning message - no color support");
    }
}
