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

/// A version of println that prints an entire message in colour or otherwise styled.
/// Format: `color_println`!(<style>, "Lorem ipsum dolor {} amet", <content>);
#[macro_export]
macro_rules! color_println {
    ($style:expr, $($arg:tt)*) => {{
        let binding = format!("{}", format_args!($($arg)*));
        let styled_text = binding.style($style);
        println!("{}", styled_text);
    }};
}

pub trait ThemeStyle {
    fn get_style(&self, theme: Theme) -> Style;
    fn to_string(&self) -> String;
}

// Enum for light and dark theme styles
#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
pub enum YinYangStyle {
    Error,
    Warning,
    Emphasis,
    Info,
    Debug,
}

impl ThemeStyle for YinYangStyle {
    // Get the corresponding color style for the message type
    fn get_style(&self, theme: Theme) -> Style {
        match theme {
            Theme::Light => match *self {
                YinYangStyle::Error => Style::new().fg::<Red>().bold(),
                YinYangStyle::Warning => Style::new().fg::<DarkOrange>().bold(),
                YinYangStyle::Emphasis => Style::new().fg::<Green>().bold(),
                YinYangStyle::Info => Style::new().fg::<Black>(),
                YinYangStyle::Debug => Style::new().fg::<Blue>(),
            },
            Theme::Dark => match *self {
                YinYangStyle::Error => Style::new().fg::<Red>().bold(),
                YinYangStyle::Warning => Style::new().fg::<Orange>().bold(),
                YinYangStyle::Emphasis => Style::new().fg::<Green>().bold(),
                YinYangStyle::Info => Style::new().fg::<White>(),
                YinYangStyle::Debug => Style::new().fg::<Cyan>(),
            },
        }
    }

    fn to_string(&self) -> String {
        match *self {
            YinYangStyle::Error => String::from("error"),
            YinYangStyle::Warning => String::from("warning"),
            YinYangStyle::Emphasis => String::from("emphasis"),
            YinYangStyle::Info => String::from("info"),
            YinYangStyle::Debug => String::from("debug"),
        }
    }
}

#[allow(dead_code)]
fn main() {
    let term = termbg::terminal();

    let theme = get_theme();

    println!("  Term : {:?}", term);

    // Get the appropriate style based on the theme
    for variant in YinYangStyle::iter() {
        let level: &str = &variant.to_string();
        let borrowed_theme = theme.as_ref();
        if let Ok(theme_ref) = borrowed_theme {
            color_println!(
                variant.get_style(*theme_ref),
                "{}",
                format!("My {theme_ref:?} style {} message", level)
            );
        } else {
            println!("My warning message - no color support");
        }
    }
}

fn get_theme() -> Result<Theme, termbg::Error> {
    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let theme: Result<Theme, termbg::Error> = termbg::theme(timeout);
    theme
}
