/*[toml]
[dependencies]
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
strum = { version = "0.26.2", features = ["derive"] }
termbg = "0.5.0"
*/

use owo_colors::colors::css::{Black, DarkOrange, Orange};
use owo_colors::colors::{Blue, Cyan, Green, Red, White};
use owo_colors::{OwoColorize, Style};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};
use termbg::Theme;

pub trait ThemeStyle {
    fn get_style(&self) -> Style;
}

// Enum for light theme styles
#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum LightStyle {
    Error,
    Warning,
    Emphasis,
    Info,
    Debug,
}

// Enum for dark theme styles
#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
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

#[allow(dead_code)]
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
                    let level: &str = &<LightStyle as Into<&str>>::into(variant);
                    let msg = &format!("My {} message", level);
                    println!("{}", msg.style(variant.get_style()));
                }
            }

            // <Opt as Into<&'static str>>::into(option).to_case(Case::Kebab)
            Theme::Dark => {
                for variant in DarkStyle::iter() {
                    let level: &str =
                        &<DarkStyle as Into<&str>>::into(variant).to_case(Case::Kebab);
                    let msg = &format!("My {} message", level);
                    println!("{}", msg.style(variant.get_style()));
                }
            }
        }
    } else {
        println!("My warning message - no color support");
    }
}
