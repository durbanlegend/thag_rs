/*[toml]
[dependencies]
lazy_static = "1.4.0"
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
termbg = "0.5.0"
*/

use owo_colors::colors::css::Orange;
// use lazy_static::lazy_static;
// use owo_colors::colors::{
//     Black, Blue, Cyan, Green, Magenta, Red, White, Yellow};
use owo_colors::colors::*;
use owo_colors::{OwoColorize, Style};
use termbg::{Error, Theme};

// Enum for light theme styles
// #[derive(Clone, Copy)]
pub enum LightStyle {
    Error,
    Warning,
    Info,
    Debug,
}

// Enum for dark theme styles
// #[derive(Clone, Copy)]
pub enum DarkStyle {
    Error,
    Warning,
    Info,
    Debug,
}

// enum ThemeType {
//     Light,
//     Dark,
// }

impl LightStyle {
    // Get the corresponding color style for the message type
    pub fn style(&self) -> Style {
        match *self {
            LightStyle::Error => {
                let style = Style::new();
                style.fg::<Red>()
            }
            LightStyle::Warning => {
                let style = Style::new();
                style.fg::<Orange>()
            }
            LightStyle::Info => {
                let style = Style::new();
                style.fg::<Black>()
            }
            LightStyle::Debug => {
                let style = Style::new();
                style.fg::<Blue>()
            }
        }
    }
}

impl DarkStyle {
    // Get the corresponding color style for the message type
    pub fn style(&self) -> Style {
        match *self {
            DarkStyle::Error => {
                let style = Style::new();
                style.fg::<Red>()
            }
            DarkStyle::Warning => {
                let style = Style::new();
                style.fg::<Yellow>()
            }
            DarkStyle::Info => {
                let style = Style::new();
                style.fg::<White>()
            }
            DarkStyle::Debug => {
                let style = Style::new();
                style.fg::<Cyan>()
            }
        }
    }
}

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let term = termbg::terminal();
    // let rgb = termbg::rgb(timeout);
    let theme: Result<Theme, Error> = termbg::theme(timeout);

    println!("  Term : {:?}", term);

    let supports_color: bool = match theme {
        Ok(_theme) => true,
        Err(ref _e) => false,
    };

    // Get the appropriate style based on the theme
    if supports_color {
        let style = match theme {
            Ok(Theme::Light) => Some(LightStyle::Warning.style()),
            Ok(Theme::Dark) => Some(DarkStyle::Warning.style()),
            _Error => None,
        };

        // Print a message with the determined style
        let style = style.unwrap();
        println!("{} ({style:?})", "My warning message".style(style));
    } else {
        println!("{}", "My warning message - no color support");
    }
}
