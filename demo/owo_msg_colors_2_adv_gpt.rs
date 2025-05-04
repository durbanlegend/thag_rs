/*[toml]
[dependencies]
crossterm = "0.29"
enum-assoc = "1.1.0"
log = "0.4.22"
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
strum = { version = "0.26.3", features = ["derive", "strum_macros", "phf"] }
supports-color= "3.0.0"
termbg = "0.5.2"
*/

/// More fully worked-out prototype of colouring and styling messages based on the level of
/// colour support of the current terminal and whether a light or dark theme is currently
/// selected. This was the result of good deal of exploration and dialog with ChatGPT.  Try it on dark vs light
/// backgrounds to see how some of the same colours "pop" when shown against a light or dark theme
/// and how some virtually or literally disappear when not well matched to the theme.
//# Purpose: Demo detection of terminal colour support and dark or light theme, colouring and styling of messages, use of `strum` crate to get enum variant from string, and AI-generated code.
//# Categories: crates, prototype, technique
use enum_assoc::Assoc;
use log::debug;
use owo_ansi::xterm as owo_xterm;
use owo_ansi::{Blue, Cyan, Green, Red, White, Yellow};
use owo_colors::colors::{self as owo_ansi, Magenta};
use owo_colors::Style;
use owo_xterm::{
    Black, BondiBlue, Copperfield, DarkMalibuBlue, DarkPurplePizzazz, DarkViolet, GuardsmanRed,
    LightCaribbeanGreen, LochmaraBlue, Silver,
};
use std::str::FromStr;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use supports_color::Stream;
use termbg::Theme;

// A version of println that prints an entire message in colour or otherwise styled.
//
// Format: `color_println`!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);
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
enum ColorLevel {
    Xterm256,
    Ansi16,
    None,
}

#[derive(EnumString, Display, PartialEq)]
enum TermBgLuma {
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

#[derive(Assoc, Clone, Debug, Display, EnumIter, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
#[strum(use_phf)]
#[func(pub fn value(&self) -> Style)]
enum MessageStyle {
    // Use Assoc to associate owo-colors::Style with each variant
    #[assoc(value = Style::new().fg::<Red>().bold())]
    Ansi16LightError,
    #[assoc(value = Style::new().fg::<Magenta>().bold())]
    Ansi16LightWarning,
    #[assoc(value = Style::new().fg::<Yellow>().bold())]
    Ansi16LightEmphasis,
    #[assoc(value = Style::new().fg::<Blue>().bold())]
    Ansi16LightOuterPrompt,
    #[assoc(value = Style::new().fg::<Cyan>().bold())]
    Ansi16LightInnerPrompt,
    #[assoc(value = Style::new().fg::<Black>())]
    Ansi16LightNormal,
    #[assoc(value = Style::new().fg::<Cyan>())]
    Ansi16LightDebug,

    #[assoc(value = Style::new().fg::<Red>().bold())]
    Ansi16DarkError,
    #[assoc(value = Style::new().fg::<Magenta>().bold())]
    Ansi16DarkWarning,
    #[assoc(value = Style::new().fg::<Yellow>().bold())]
    Ansi16DarkEmphasis,
    #[assoc(value = Style::new().fg::<Blue>().bold())]
    Ansi16DarkOuterPrompt,
    #[assoc(value = Style::new().fg::<Green>().bold())]
    Ansi16DarkInnerPrompt,
    #[assoc(value = Style::new().fg::<White>())]
    Ansi16DarkNormal,
    #[assoc(value = Style::new().fg::<Cyan>())]
    Ansi16DarkDebug,

    #[assoc(value = Style::new().fg::<GuardsmanRed>().bold())]
    Xterm256LightError,
    #[assoc(value = Style::new().fg::<DarkViolet>().bold())]
    Xterm256LightWarning,
    #[assoc(value = Style::new().fg::<Copperfield>().bold())]
    Xterm256LightEmphasis,
    #[assoc(value = Style::new().fg::<DarkMalibuBlue>().bold())]
    Xterm256LightOuterPrompt,
    #[assoc(value = Style::new().fg::<LightCaribbeanGreen>().bold())]
    Xterm256LightInnerPrompt,
    #[assoc(value = Style::new().fg::<Black>())]
    Xterm256LightNormal,
    #[assoc(value = Style::new().fg::<LochmaraBlue>())]
    Xterm256LightDebug,

    #[assoc(value = Style::new().fg::<GuardsmanRed>().bold())]
    Xterm256DarkError,
    #[assoc(value = Style::new().fg::<DarkPurplePizzazz>().bold())]
    Xterm256DarkWarning,
    #[assoc(value = Style::new().fg::<Copperfield>().bold())]
    Xterm256DarkEmphasis,
    #[assoc(value = Style::new().fg::<DarkMalibuBlue>().bold())]
    Xterm256DarkOuterPrompt,
    #[assoc(value = Style::new().fg::<LightCaribbeanGreen>().bold())]
    Xterm256DarkInnerPrompt,
    #[assoc(value = Style::new().fg::<Silver>())]
    Xterm256DarkNormal,
    #[assoc(value = Style::new().fg::<BondiBlue>())]
    Xterm256DarkDebug,
}

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
fn clear_screen() {
    //     let mut out = stdout();
    //     out.execute(MoveToColumn(0)).unwrap();
    //     out.execute(Show).unwrap();
    //     out.flush().unwrap();
}

fn get_theme() -> Result<Theme, termbg::Error> {
    let timeout = std::time::Duration::from_millis(100);

    println!("Checking terminal background color");
    let theme: Result<Theme, termbg::Error> = termbg::theme(timeout);
    // clear_screen();
    theme
}

/// Fully worked-out demonstration of colouring and styling display messages according
/// to message level.
fn main() {
    let term = termbg::terminal();
    // clear_screen();
    debug!("  Term : {:?}", term);

    let maybe_theme = get_theme();
    let term_bg_luma = match maybe_theme {
        Ok(theme) => match theme {
            Theme::Light => TermBgLuma::Light,
            Theme::Dark => TermBgLuma::Dark,
        },
        Err(_) => TermBgLuma::Dark,
    };

    let maybe_color_support = supports_color::on(Stream::Stdout);
    let color_level = match maybe_color_support {
        Some(color_support) => {
            if color_support.has_16m || color_support.has_256 {
                Some(ColorLevel::Xterm256)
            } else {
                Some(ColorLevel::Ansi16)
            }
        }
        None => None,
    };

    if color_level.is_none() {
        println!("No colour support found for terminal");
    } else {
        let msg_level = MessageType::Warning;

        let color_qual = color_level.unwrap().to_string().to_lowercase();
        let theme_qual = term_bg_luma.to_string().to_lowercase();
        let msg_level_qual = msg_level.to_string().to_lowercase();
        // debug!("Calling from_str on {}_{}_{}", &color_qual, &theme_qual, &msg_level_qual);
        let style = MessageStyle::from_str(&format!(
            "{}_{}_{}",
            &color_qual, &theme_qual, &msg_level_qual
        ));
        let actual_style = Some(style.unwrap().value());

        // Use actual_style for displaying the message
        color_println!(actual_style, "{}", "Colorized Warning message\n");

        for variant in MessageStyle::iter() {
            let variant_string: &str = &variant.to_string();
            color_println!(Some(variant.value()), "My {variant_string} message");
        }
    }
}
