/*[toml]
[dependencies]
convert_case = "0.6.0"
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
serde = { version = "1.0.198", features = ["derive"] }
strum = { version = "0.26.2", features = ["derive"] }
termbg = "0.5.0"
*/

use owo_colors::colors::css::{Black, DarkOrange, Orange};
use owo_colors::colors::*;
use owo_colors::colors::{Blue, Cyan, Green, Red, White};
use owo_colors::{OwoColorize, Style};
// use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};
use termbg::Theme;

pub trait ThemeStyle {
    fn get_style(&self) -> Style;
}

// Enum for light theme styles
#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
pub enum LightStyle {
    Error,
    Warning,
    Info,
    Debug,
}

// Enum for dark theme styles
#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
pub enum DarkStyle {
    Error,
    Warning,
    Info,
    Debug,
}

impl ThemeStyle for LightStyle {
    // Get the corresponding color style for the message type
    fn get_style(&self) -> Style {
        match *self {
            LightStyle::Error => Style::new().fg::<Red>().bold(),
            LightStyle::Warning => Style::new().fg::<DarkBlue>().bold(),
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
            DarkStyle::Info => Style::new().fg::<White>(),
            DarkStyle::Debug => Style::new().fg::<Cyan>(),
        }
    }
}

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let term = termbg::terminal();
    // let rgb = termbg::rgb(timeout);
    let theme: Result<Theme, termbg::Error> = termbg::theme(timeout);

    println!("  Term : {:?}", term);

    let supports_color: bool = match theme {
        Ok(_theme) => true,
        Err(ref _e) => false,
    };

    // Get the appropriate style based on the theme
    if supports_color {
        // let style = match theme {
        //     Ok(Theme::Light) => Some(LightStyle::Warning.style()),
        //     Ok(Theme::Dark) => Some(DarkStyle::Warning.style()),
        //     ref _Error => None,
        // };

        // // Print a message with the determined style
        // let style = style.unwrap();
        // println!("{} ({style:?})", "My warning message".style(style));

        match theme.unwrap() {
            Theme::Light => {
                for variant in LightStyle::iter() {
                    // <Opt as Into<&'static str>>::into(option).to_case(Case::Kebab)
                    {
                        let level: &str =
                            &<LightStyle as Into<&str>>::into(variant).to_case(Case::Kebab);
                        let style = variant.get_style();
                        let msg = &format!("My {level} message");
                        // println!("{}  style {style:?}", msg.style(style));
                        println!("{}", msg.style(style));
                    };
                }
            }

            // <Opt as Into<&'static str>>::into(option).to_case(Case::Kebab)
            Theme::Dark => {
                for variant in DarkStyle::iter() {
                    let level: &str =
                        &<DarkStyle as Into<&str>>::into(variant).to_case(Case::Kebab);
                    let style = variant.get_style();
                    let msg = &format!("My {level} message");
                    // println!("{}  style {style:?}", msg.style(style));
                    println!("{}", msg.style(style));
                }
            }
        }
    } else {
        println!("My warning message - no color support");
    }
}
