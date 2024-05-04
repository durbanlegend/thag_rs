/*[toml]
[dependencies]
log = "0.4.21"
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
strum = { version = "0.26.2", features = ["derive", "strum_macros", "phf"] }
supports-color= "3.0.0"
termbg = "0.5.0"
*/

use std::{fmt::Display, str::FromStr};

use owo_ansi::{Blue, Cyan, Green, Red, White, Yellow};
use owo_colors::colors::{self as owo_ansi, Magenta};

use owo_ansi::xterm as owo_xterm;
use owo_xterm::{
    Black, BondiBlue, Copperfield, DarkMalibuBlue, DarkPurplePizzazz, DarkViolet, GuardsmanRed,
    LightCaribbeanGreen, LochmaraBlue, Silver,
};

use log::debug;
use owo_colors::Style;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use supports_color::Stream;
use termbg::Theme;

/// A version of println that prints an entire message in colour or otherwise styled.
/// Format: `color_println`!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);
#[macro_export]
macro_rules! color_println {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
        if let Some(style) = $style {
                // Qualified form to avoid imports in calling code.
                println!("{}", owo_colors::Style::style(&style, content));
        } else {
            println!("{}", content);
        }
    }};
}

#[derive(Clone, EnumString, Display, PartialEq)]
/// We include `TrueColor` in Xterm256 as we're not interested in more than 256 colours just for messages.
pub enum ColorSupport {
    Xterm256,
    Ansi16,
    None,
}

#[derive(EnumString, Display, PartialEq)]
pub enum TermTheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, EnumString, Display, PartialEq)]
enum MessageType {
    Error,
    Warning,
    Emphasis,
    OuterPrompt,
    InnerPrompt,
    Normal,
    Debug,
}

pub trait ThemeStyle: Display {
    fn get_style(&self) -> Option<Style>;
}

#[derive(Clone, Debug, Display, EnumIter, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
#[strum(use_phf)]
pub enum MessageStyle {
    Ansi16LightError,
    Ansi16LightWarning,
    Ansi16LightEmphasis,
    Ansi16LightOuterPrompt,
    Ansi16LightInnerPrompt,
    Ansi16LightNormal,
    Ansi16LightDebug,

    Ansi16DarkError,
    Ansi16DarkWarning,
    Ansi16DarkEmphasis,
    Ansi16DarkOuterPrompt,
    Ansi16DarkInnerPrompt,
    Ansi16DarkNormal,
    Ansi16DarkDebug,

    Xterm256LightError,
    Xterm256LightWarning,
    Xterm256LightEmphasis,
    Xterm256LightOuterPrompt,
    Xterm256LightInnerPrompt,
    Xterm256LightNormal,
    Xterm256LightDebug,

    Xterm256DarkError,
    Xterm256DarkWarning,
    Xterm256DarkEmphasis,
    Xterm256DarkOuterPrompt,
    Xterm256DarkInnerPrompt,
    Xterm256DarkNormal,
    Xterm256DarkDebug,
}

impl ThemeStyle for MessageStyle {
    fn get_style(&self) -> Option<Style> {
        let style = match self {
            MessageStyle::Ansi16LightError => Style::new().fg::<Red>().bold(),
            MessageStyle::Ansi16LightWarning => Style::new().fg::<Magenta>().bold(),
            MessageStyle::Ansi16LightEmphasis => Style::new().fg::<Yellow>().bold(),
            MessageStyle::Ansi16LightOuterPrompt => Style::new().fg::<Blue>().bold(),
            MessageStyle::Ansi16LightInnerPrompt => Style::new().fg::<Cyan>().bold(),
            MessageStyle::Ansi16LightNormal => Style::new().fg::<Black>(),
            MessageStyle::Ansi16LightDebug => Style::new().fg::<Cyan>(),
            MessageStyle::Ansi16DarkError => Style::new().fg::<Red>().bold(),
            MessageStyle::Ansi16DarkWarning => Style::new().fg::<Magenta>().bold(),
            MessageStyle::Ansi16DarkEmphasis => Style::new().fg::<Yellow>().bold(),
            MessageStyle::Ansi16DarkOuterPrompt => Style::new().fg::<Blue>().bold(),
            MessageStyle::Ansi16DarkInnerPrompt => Style::new().fg::<Green>().bold(),
            MessageStyle::Ansi16DarkNormal => Style::new().fg::<White>(),
            MessageStyle::Ansi16DarkDebug => Style::new().fg::<Cyan>(),
            MessageStyle::Xterm256LightError => Style::new().fg::<GuardsmanRed>().bold(),
            MessageStyle::Xterm256LightWarning => Style::new().fg::<DarkViolet>().bold(),
            MessageStyle::Xterm256LightEmphasis => Style::new().fg::<Copperfield>().bold(),
            MessageStyle::Xterm256LightOuterPrompt => Style::new().fg::<DarkMalibuBlue>().bold(),
            MessageStyle::Xterm256LightInnerPrompt => {
                Style::new().fg::<LightCaribbeanGreen>().bold()
            }
            MessageStyle::Xterm256LightNormal => Style::new().fg::<Black>(),
            MessageStyle::Xterm256LightDebug => Style::new().fg::<LochmaraBlue>(),
            MessageStyle::Xterm256DarkError => Style::new().fg::<GuardsmanRed>().bold(),
            MessageStyle::Xterm256DarkWarning => Style::new().fg::<DarkPurplePizzazz>().bold(),
            MessageStyle::Xterm256DarkEmphasis => Style::new().fg::<Copperfield>().bold(),
            MessageStyle::Xterm256DarkOuterPrompt => Style::new().fg::<DarkMalibuBlue>().bold(),
            MessageStyle::Xterm256DarkInnerPrompt => {
                Style::new().fg::<LightCaribbeanGreen>().bold()
            }
            MessageStyle::Xterm256DarkNormal => Style::new().fg::<Silver>(),
            MessageStyle::Xterm256DarkDebug => Style::new().fg::<BondiBlue>(),
        };
        Some(style)
    }
}

fn get_theme() -> Result<Theme, termbg::Error> {
    let timeout = std::time::Duration::from_millis(100);

    // debug!("Check terminal background color");
    let theme: Result<Theme, termbg::Error> = termbg::theme(timeout);
    theme
}

fn main() {
    let term = termbg::terminal();
    debug!("  Term : {:?}", term);

    let maybe_theme = get_theme();
    let term_theme = match maybe_theme {
        Ok(theme) => match theme {
            Theme::Light => TermTheme::Light,
            Theme::Dark => TermTheme::Dark,
        },
        Err(_) => TermTheme::Dark,
    };

    let color_support = match supports_color::on(Stream::Stdout) {
        Some(color_support) => {
            if color_support.has_16m || color_support.has_256 {
                Some(ColorSupport::Xterm256)
            } else {
                Some(ColorSupport::Ansi16)
            }
        }
        None => None,
    };

    if color_support.is_none() {
        println!("No colour support found for terminal");
    } else {
        let msg_level = MessageType::Warning;

        let color_qual = color_support.unwrap().to_string().to_lowercase();
        let theme_qual = term_theme.to_string().to_lowercase();
        let msg_level_qual = msg_level.to_string().to_lowercase();
        // eprintln!("Calling from_str on {}_{}_{}", &color_qual, &theme_qual, &msg_level_qual);
        let style = MessageStyle::from_str(&format!(
            "{}_{}_{}",
            &color_qual, &theme_qual, &msg_level_qual
        ));
        let actual_style = style.unwrap().get_style();

        // Use actual_style for displaying the message
        color_println!(actual_style, "{}", "Colored Warning message\n");

        for variant in MessageStyle::iter() {
            let variant_string: &str = &variant.to_string();
            color_println!(variant.get_style(), "My {variant_string} message");
        }
    }
}
