/*[toml]
[dependencies]
lazy_static = "1.4.0"
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

use lazy_static::lazy_static;
use log::debug;
use owo_colors::Style;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use supports_color::Stream;
use termbg::Theme;

lazy_static! {
    static ref COLOR_SUPPORT: Option<ColorSupport> = match supports_color::on(Stream::Stdout) {
        Some(color_support) => {
            if color_support.has_16m || color_support.has_256 {
                Some(ColorSupport::Xterm256)
            } else {
                Some(ColorSupport::Ansi16)
            }
        }
        None => None,
    };
}

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
#[strum(serialize_all = "snake_case")]
pub enum MessageLevel {
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
            MessageStyle::Ansi16LightError | MessageStyle::Ansi16DarkError => {
                Style::new().fg::<Red>().bold()
            }
            MessageStyle::Ansi16LightWarning | MessageStyle::Ansi16DarkWarning => {
                Style::new().fg::<Magenta>().bold()
            }
            MessageStyle::Ansi16LightEmphasis | MessageStyle::Ansi16DarkEmphasis => {
                Style::new().fg::<Yellow>().bold()
            }
            MessageStyle::Ansi16LightOuterPrompt | MessageStyle::Ansi16DarkOuterPrompt => {
                Style::new().fg::<Blue>().bold()
            }
            MessageStyle::Ansi16LightInnerPrompt => Style::new().fg::<Cyan>().bold(),
            #[allow(clippy::match_same_arms)]
            MessageStyle::Ansi16LightNormal => Style::new().fg::<White>(), // Reversal beats me
            MessageStyle::Ansi16LightDebug | MessageStyle::Ansi16DarkDebug => {
                Style::new().fg::<Cyan>()
            }
            MessageStyle::Ansi16DarkInnerPrompt => Style::new().fg::<Green>().bold(),
            #[allow(clippy::match_same_arms)]
            MessageStyle::Ansi16DarkNormal => Style::new().fg::<White>(),
            MessageStyle::Xterm256LightError | MessageStyle::Xterm256DarkError => {
                Style::new().fg::<GuardsmanRed>().bold()
            }
            MessageStyle::Xterm256LightWarning => Style::new().fg::<DarkViolet>().bold(),
            MessageStyle::Xterm256LightEmphasis | MessageStyle::Xterm256DarkEmphasis => {
                Style::new().fg::<Copperfield>().bold()
            }
            MessageStyle::Xterm256LightOuterPrompt | MessageStyle::Xterm256DarkOuterPrompt => {
                Style::new().fg::<DarkMalibuBlue>().bold()
            }
            MessageStyle::Xterm256LightInnerPrompt | MessageStyle::Xterm256DarkInnerPrompt => {
                Style::new().fg::<LightCaribbeanGreen>()
            }
            MessageStyle::Xterm256LightNormal => Style::new().fg::<Black>(),
            MessageStyle::Xterm256LightDebug => Style::new().fg::<LochmaraBlue>(),
            MessageStyle::Xterm256DarkWarning => Style::new().fg::<DarkPurplePizzazz>().bold(),
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

pub(crate) fn get_term_theme() -> TermTheme {
    match get_theme() {
        Ok(theme) => match theme {
            Theme::Light => TermTheme::Light,
            Theme::Dark => TermTheme::Dark,
        },
        Err(_) => TermTheme::Dark,
    }
}

pub fn resolve_style(message_level: MessageLevel) -> Option<Style> {
    let term_theme = get_term_theme();
    let color_qual = COLOR_SUPPORT.as_ref().unwrap().to_string().to_lowercase();
    let theme_qual = term_theme.to_string().to_lowercase();
    let msg_level_qual = message_level.to_string().to_lowercase();
    let message_style = MessageStyle::from_str(&format!(
        "{}_{}_{}",
        &color_qual, &theme_qual, &msg_level_qual
    ));
    // debug!(
    //     "Called from_str on {}_{}_{}, found {message_style:#?}",
    //     &color_qual, &theme_qual, &msg_level_qual,
    // );
    match message_style {
        Ok(message_style) => message_style.get_style(),
        Err(_) => None,
    }
}

#[allow(dead_code)]
fn main() {
    let term = termbg::terminal();
    debug!("  Term : {:?}", term);

    let term_theme = get_term_theme();

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
        let msg_level = MessageLevel::Warning;

        let color_qual = color_support.unwrap().to_string().to_lowercase();
        let theme_qual = term_theme.to_string().to_lowercase();
        let msg_level_qual = msg_level.to_string().to_lowercase();
        // eprintln!("Calling from_str on {}_{}_{}", &color_qual, &theme_qual, &msg_level_qual);
        let message_style = MessageStyle::from_str(&format!(
            "{}_{}_{}",
            &color_qual, &theme_qual, &msg_level_qual
        ));
        let style = message_style.unwrap().get_style();

        // Use style for displaying the message
        color_println!(style, "{}", "Colored Warning message\n");

        for variant in MessageStyle::iter() {
            let variant_string: &str = &variant.to_string();
            color_println!(variant.get_style(), "My {variant_string} message");
        }
    }
}
