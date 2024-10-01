#![allow(clippy::implicit_return)]
use crate::generate_styles;
use crate::logging::Verbosity;
#[cfg(not(target_os = "windows"))]
use crate::termbg::{terminal, theme, Theme};
use crate::{config, debug_log, log, ThagResult};

use crossterm::terminal::{self, is_raw_mode_enabled};
use firestorm::profile_fn;
use nu_ansi_term::{Color, Style};
use scopeguard::defer;
use serde::Deserialize;
#[cfg(target_os = "windows")]
use std::env;
use std::sync::OnceLock;
use std::{fmt::Display, str::FromStr};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
#[cfg(not(target_os = "windows"))]
use supports_color::Stream;

pub type NuStyle = nu_ansi_term::Style;

#[derive(Debug)]
pub enum Xterm256LightStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

#[derive(Debug)]
pub enum Xterm256DarkStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

#[derive(Debug)]
pub enum Ansi16LightStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

#[derive(Debug)]
pub enum Ansi16DarkStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

generate_styles!(
    (Xterm256LightStyle, Light, Xterm256),
    (Xterm256DarkStyle, Dark, Xterm256),
    (Ansi16LightStyle, Light, Ansi16),
    (Ansi16DarkStyle, Dark, Ansi16)
);

/// Returns lazy static color values. Converted from `lazy_static` implementation
/// in accordance with the example provided in the `lazy_static` Readme. Converted
/// for the learning experience and to facilitate handling errors and the unwelcome
/// side-effects of calling crates (in particular `termbg`) that switch off raw mode.
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered.
pub fn coloring<'a>() -> (Option<&'a ColorSupport>, &'a TermTheme) {
    profile_fn!(coloring);

    static COLOR_SUPPORT: OnceLock<Option<ColorSupport>> = OnceLock::new();
    static TERM_THEME: OnceLock<TermTheme> = OnceLock::new();
    if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding supports_color for testing");
        return (Some(&ColorSupport::Ansi16), &TermTheme::Dark);
    }
    let raw_before = terminal::is_raw_mode_enabled();
    if let Ok(raw_then) = raw_before {
        defer! {
            let raw_now = match is_raw_mode_enabled() {
                Ok(val) => val,
                Err(e) => {
                    #[cfg(debug_assertions)]
                    debug_log!("Failed to check raw mode status: {:?}", e);
                    return;
                }
            };

            if raw_now == raw_then {
                #[cfg(debug_assertions)]
                debug_log!("Raw mode status unchanged.");
            } else if let Err(e) = restore_raw_status(raw_then) {
                    #[cfg(debug_assertions)]
                    debug_log!("Failed to restore raw mode: {:?}", e);
            } else {
                #[cfg(debug_assertions)]
                debug_log!("Raw mode restored to previous state.");
            }
        }
    }

    let color_support = COLOR_SUPPORT.get_or_init(|| {
        (*config::MAYBE_CONFIG)
            .as_ref()
            .map_or_else(get_color_level, |config| {
                match config.colors.color_support {
                    ColorSupport::Xterm256 | ColorSupport::Ansi16 | ColorSupport::None => {
                        Some(config.colors.color_support.clone())
                    }
                    ColorSupport::Default => get_color_level(),
                }
            })
    });

    let term_theme = TERM_THEME.get_or_init(|| {
        (*config::MAYBE_CONFIG).as_ref().map_or_else(
            || resolve_term_theme().unwrap_or_default(),
            |config| {
                if matches!(&config.colors.term_theme, &TermTheme::None) {
                    resolve_term_theme().unwrap_or_default()
                } else {
                    config.colors.term_theme.clone()
                }
            },
        )
    });

    (color_support.as_ref(), term_theme)
}

/// Initializes and returns the TUI selection background coloring.
pub fn tui_selection_bg(term_theme: &TermTheme) -> TuiSelectionBg {
    static TUI_SELECTION_BG: OnceLock<TuiSelectionBg> = OnceLock::new();
    TUI_SELECTION_BG
        .get_or_init(|| match term_theme {
            TermTheme::Light => TuiSelectionBg::BlueYellow,
            _ => TuiSelectionBg::RedWhite,
        })
        .clone()
}

#[macro_export]
macro_rules! generate_styles {
    (
        $(
            ($style_enum:ident, $term_theme:ident, $color_support:ident)
        ),*
    ) => {
        $(
            impl From<&Lvl> for $style_enum {
                fn from(message_level: &Lvl) -> Self {
                    profile_fn!(style_enum_from_lvl);

                    // dbg!(&$style_enum::Warning);
                    // dbg!(&message_level);
                    match message_level {
                        Lvl::Error => $style_enum::Error,
                        Lvl::Warning => $style_enum::Warning,
                        Lvl::Emphasis => $style_enum::Emphasis,
                        Lvl::Heading => $style_enum::Heading,
                        Lvl::Subheading => $style_enum::Subheading,
                        Lvl::Normal => $style_enum::Normal,
                        Lvl::Debug => $style_enum::Debug,
                        Lvl::Ghost => $style_enum::Ghost,
                    }
                }
            }

            // use crate::styles::$style_enum;
            impl From<&$style_enum> for NuStyle {
                #[must_use]
                fn from(style_enum: &$style_enum) -> Self {
                    profile_fn!(style_from_style_enum);
                    match style_enum {
                        $style_enum::Error => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Error))).bold(),
                        $style_enum::Warning => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Warning))).bold(),
                        $style_enum::Emphasis => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Emphasis))).bold(),
                        $style_enum::Heading => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Heading))).bold(),
                        $style_enum::Subheading => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Subheading))).bold(),
                        $style_enum::Normal => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Normal))),
                        $style_enum::Debug => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Debug))),
                        $style_enum::Ghost => NuStyle::from(nu_ansi_term::Color::Fixed(u8::from(&Lvl::Ghost))).dimmed().italic(),
                    }
                }
            }
        )*

        pub fn init_styles(
            term_theme: &TermTheme,
            color_support: Option<&ColorSupport>,
        ) -> fn(Lvl) -> Style {
            profile_fn!(init_styles);
            static STYLE_MAPPING: OnceLock<fn(Lvl) -> Style> = OnceLock::new();

            *STYLE_MAPPING.get_or_init(|| match (term_theme, color_support) {
                $(
                    (TermTheme::$term_theme, Some(ColorSupport::$color_support)) => {
                        |message_level| NuStyle::from(&$style_enum::from(&message_level))
                    }
                ),*
                _ => |message_level| NuStyle::from(&Ansi16DarkStyle::from(&message_level)), // Fallback
            })
        }
    };
}

/// Generates the alternative style mapping enums.
pub fn gen_mappings(
    term_theme: &TermTheme,
    color_support: Option<&ColorSupport>,
) -> fn(Lvl) -> Style {
    profile_fn!(gen_mappings);
    static STYLE_MAPPING: OnceLock<fn(Lvl) -> Style> = OnceLock::new();
    *STYLE_MAPPING.get_or_init(|| {
        // Call init_styles to ensure styles are initialized on first access
        init_styles(term_theme, color_support)
    })
}

#[cfg(target_os = "windows")]
fn resolve_term_theme() -> ThagResult<TermTheme> {
    profile_fn!(resolve_term_theme);
    Ok(TermTheme::Dark)
}

#[cfg(not(target_os = "windows"))]
fn resolve_term_theme() -> ThagResult<TermTheme> {
    profile_fn!(resolve_term_theme);
    let raw_before = terminal::is_raw_mode_enabled()?;
    #[cfg(debug_assertions)]
    debug_log!("About to call termbg");
    let timeout = std::time::Duration::from_millis(100);

    // #[cfg(debug_assertions)]
    // debug_log!("Check terminal background color");
    let theme = theme(timeout);

    maybe_restore_raw_status(raw_before)?;

    match theme {
        Ok(Theme::Light) => Ok(TermTheme::Light),
        Ok(Theme::Dark) | Err(_) => Ok(TermTheme::Dark),
    }
}

fn maybe_restore_raw_status(raw_before: bool) -> ThagResult<()> {
    profile_fn!(maybe_restore_raw_status);
    let raw_after = terminal::is_raw_mode_enabled()?;
    if raw_before != raw_after {
        restore_raw_status(raw_before)?;
    }
    Ok(())
}

fn restore_raw_status(raw_before: bool) -> ThagResult<()> {
    profile_fn!(restore_raw_status);
    if raw_before {
        terminal::enable_raw_mode()?;
    } else {
        terminal::disable_raw_mode()?;
    }
    Ok(())
}

/// A struct of the color support details, borrowed from crate `supports-color` since we
/// can't import it because the `level` field is indispensable but private.
/// This type is returned from `supports_color::on`. See documentation for its fields for
/// more details.
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorLevel {
    level: usize,
    /// Basic ANSI colors are supported.
    pub has_basic: bool,
    /// 256-bit colors are supported.
    pub has_256: bool,
    /// 16 million (RGB) colors are supported.
    pub has_16m: bool,
}

#[cfg(target_os = "windows")]
fn get_color_level() -> Option<ColorSupport> {
    profile_fn!(get_color_level);
    let color_level = translate_level(supports_color());
    match color_level {
        Some(color_level) => {
            if color_level.has_16m || color_level.has_256 {
                Some(ColorSupport::Xterm256)
            } else {
                Some(ColorSupport::Ansi16)
            }
        }
        None => None,
    }
}

#[cfg(not(target_os = "windows"))]
fn get_color_level() -> Option<ColorSupport> {
    profile_fn!(get_color_level);
    #[cfg(debug_assertions)]
    debug_log!("About to call supports_color");
    let color_level = supports_color::on(Stream::Stdout);
    color_level.map(|color_level| {
        if color_level.has_16m || color_level.has_256 {
            ColorSupport::Xterm256
        } else {
            ColorSupport::Ansi16
        }
    })
}

#[cfg(target_os = "windows")]
fn env_force_color() -> usize {
    profile_fn!(env_force_color);
    if let Ok(force) = env::var("FORCE_COLOR") {
        match force.as_ref() {
            "true" | "" => 1,
            "false" => 0,
            f => std::cmp::min(f.parse().unwrap_or(1), 3),
        }
    } else if let Ok(cli_clr_force) = env::var("CLICOLOR_FORCE") {
        usize::from(cli_clr_force != "0")
    } else {
        0
    }
}

#[cfg(target_os = "windows")]
fn env_no_color() -> bool {
    profile_fn!(env_no_color);
    match as_str(&env::var("NO_COLOR")) {
        Ok("0") | Err(_) => false,
        Ok(_) => true,
    }
}

// same as Option::as_deref
#[cfg(target_os = "windows")]
fn as_str<E>(option: &Result<String, E>) -> Result<&str, &E> {
    match option {
        Ok(inner) => Ok(inner),
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "windows")]
fn translate_level(level: usize) -> Option<ColorLevel> {
    if level == 0 {
        None
    } else {
        Some(ColorLevel {
            level,
            has_basic: true,
            has_256: level >= 2,
            has_16m: level >= 3,
        })
    }
}

#[cfg(target_os = "windows")]
fn supports_color() -> usize {
    profile_fn!(supports_color);
    let force_color = env_force_color();
    if force_color > 0 {
        force_color
    } else if env_no_color()
        || as_str(&env::var("TERM")) == Ok("dumb")
        || env::var("IGNORE_IS_TERMINAL").map_or(false, |v| v != "0")
    {
        0
    } else if env::var("COLORTERM").map(|colorterm| check_colorterm_16m(&colorterm)) == Ok(true)
        || env::var("TERM").map(|term| check_term_16m(&term)) == Ok(true)
        || as_str(&env::var("TERM_PROGRAM")) == Ok("iTerm.app")
    {
        3
    } else if as_str(&env::var("TERM_PROGRAM")) == Ok("Apple_Terminal")
        || env::var("TERM").map(|term| check_256_color(&term)) == Ok(true)
    {
        2
    } else {
        usize::from(
            env::var("COLORTERM").is_ok()
                || env::var("TERM").map(|term| check_ansi_color(&term)) == Ok(true)
                || env::consts::OS == "windows"
                || env::var("CLICOLOR").map_or(false, |v| v != "0"),
        )
    }
}

#[cfg(target_os = "windows")]
fn check_ansi_color(term: &str) -> bool {
    term.starts_with("screen")
        || term.starts_with("xterm")
        || term.starts_with("vt100")
        || term.starts_with("vt220")
        || term.starts_with("rxvt")
        || term.contains("color")
        || term.contains("ansi")
        || term.contains("cygwin")
        || term.contains("linux")
}

#[cfg(target_os = "windows")]
fn check_colorterm_16m(colorterm: &str) -> bool {
    colorterm == "truecolor" || colorterm == "24bit"
}

#[cfg(target_os = "windows")]
fn check_term_16m(term: &str) -> bool {
    term.ends_with("direct") || term.ends_with("truecolor")
}

#[cfg(target_os = "windows")]
fn check_256_color(term: &str) -> bool {
    term.ends_with("256") || term.ends_with("256color")
}

/// Retrieve whether the terminal theme is light or dark, to allow an appropriate colour
/// palette to be chosen.
#[must_use]
pub fn get_term_theme() -> &'static TermTheme {
    profile_fn!(get_term_theme);
    coloring().1
}

/// A trait for common handling of the different colour palettes.
pub trait NuColor: Display {
    fn get_color(&self) -> Color;
    /// Protection in case enum gets out of order, otherwise I think we could cast the variant to a number.
    fn get_fixed_code(&self) -> u8;
}

#[must_use]
pub fn get_style(
    message_level: &Lvl,
    term_theme: &TermTheme,
    color_support: Option<&ColorSupport>,
) -> Style {
    // dbg!();
    let mapping = gen_mappings(term_theme, color_support);
    // dbg!(&mapping);
    mapping(*message_level)
}

/// A version of println that prints an entire message in colour or otherwise styled.
/// Format: `cprtln!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);`
#[macro_export]
macro_rules! cprtln {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
        let style: nu_ansi_term::Style = $style;
        // Qualified form to avoid imports in calling code.
        let painted = style.paint(content);
        let verbosity = $crate::logging::get_verbosity();
        log!(verbosity, "{}\n", painted);
    }};
}

#[macro_export]
macro_rules! cvprtln {
    ($level:expr, $verbosity:expr, $msg:expr) => {{
        if $verbosity >= $crate::logging::get_verbosity() {
            let (maybe_color_support, term_theme) = coloring();
            let style = $crate::colors::get_style(&$level, term_theme, maybe_color_support);
            cprtln!(style, $msg);
        }
    }};
}

/// An enum to categorise the current terminal's level of colour support as detected, configured
/// or defaulted. We fold `TrueColor` into Xterm256 as we're not interested in more than 256
/// colours just for messages.
#[derive(Clone, Debug, Default, Deserialize, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum ColorSupport {
    Xterm256,
    Ansi16,
    None,
    #[default]
    Default,
}

/// An enum to categorise the current terminal's light or dark theme as detected, configured
/// or defaulted.
#[derive(Clone, Debug, Default, Deserialize, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum TermTheme {
    Light,
    Dark,
    #[default]
    None,
}

/// An enum to categorise the current TUI editor highlighting scheme for the selected
/// line as configured or defaulted.
#[derive(Clone, Debug, Default, Deserialize, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum TuiSelectionBg {
    #[default]
    BlueYellow,
    RedWhite,
}

/// An enum to categorise the supported message types for display.
#[derive(Debug, Clone, Copy, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum MessageLevel {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

pub type Lvl = MessageLevel;

impl Lvl {
    pub const ERR: Self = Self::Error;
    pub const WARN: Self = Self::Warning;
    pub const EMPH: Self = Self::Emphasis;
    pub const HEAD: Self = Self::Heading;
    pub const SUBH: Self = Self::Subheading;
    pub const NORM: Self = Self::Normal;
    pub const DBUG: Self = Self::Debug;
    pub const GHST: Self = Self::Ghost;
}

impl From<&Lvl> for u8 {
    fn from(message_level: &Lvl) -> Self {
        let message_style = MessageStyle::from(message_level);
        let xterm_color = XtermColor::from(&message_style);
        xterm_color.get_fixed_code()
    }
}

/// A trait to map a `MessageStyle` to a `nu_ansi_term::Style`.
pub trait NuThemeStyle: Display {
    fn get_style(&self) -> Style;
}

/// An enum of all the supported message styles for different levels of terminal colour support.
#[derive(Clone, Debug, Display, EnumIter, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[strum(use_phf)]
pub enum MessageStyle {
    Ansi16LightError,
    Ansi16LightWarning,
    Ansi16LightEmphasis,
    Ansi16LightHeading,
    Ansi16LightSubheading,
    Ansi16LightNormal,
    Ansi16LightDebug,
    Ansi16LightGhost,

    Ansi16DarkError,
    Ansi16DarkWarning,
    Ansi16DarkEmphasis,
    Ansi16DarkHeading,
    Ansi16DarkSubheading,
    Ansi16DarkNormal,
    Ansi16DarkDebug,
    Ansi16DarkGhost,

    Xterm256LightError,
    Xterm256LightWarning,
    Xterm256LightEmphasis,
    Xterm256LightHeading,
    Xterm256LightSubheading,
    Xterm256LightNormal,
    Xterm256LightDebug,
    Xterm256LightGhost,

    Xterm256DarkError,
    Xterm256DarkWarning,
    Xterm256DarkEmphasis,
    Xterm256DarkHeading,
    Xterm256DarkSubheading,
    Xterm256DarkNormal,
    Xterm256DarkDebug,
    Xterm256DarkGhost,
}

impl From<&Lvl> for MessageStyle {
    fn from(message_level: &Lvl) -> Self {
        profile_fn!(msg_style_from_lvl);
        let message_style: Self = {
            let (maybe_color_support, term_theme) = coloring();
            maybe_color_support.map_or(Self::Ansi16DarkNormal, |color_support| {
                let color_qual = color_support.to_string().to_lowercase();
                let theme_qual = term_theme.to_string().to_lowercase();
                let msg_level_qual = message_level.to_string().to_lowercase();
                let message_style = Self::from_str(&format!(
                    "{}_{}_{}",
                    &color_qual, &theme_qual, &msg_level_qual
                )).unwrap_or(Self::Ansi16DarkNormal);
                #[cfg(debug_assertions)]
                debug_log!(
                    "Called from_str on {color_qual}_{theme_qual}_{msg_level_qual}, found {message_style:#?}",
                );
                message_style
            })
        };
        message_style
    }
}

/// An implementation to facilitate conversion to `ratatui` and potentially other
/// color implementations.
#[allow(clippy::match_same_arms)]
impl From<&MessageStyle> for XtermColor {
    fn from(message_style: &MessageStyle) -> Self {
        match *message_style {
            MessageStyle::Ansi16LightError => Self::UserRed,
            MessageStyle::Ansi16LightWarning => Self::UserMagenta,
            MessageStyle::Ansi16LightEmphasis => Self::UserYellow,
            MessageStyle::Ansi16LightHeading => Self::UserBlue,
            MessageStyle::Ansi16LightSubheading => Self::UserCyan,
            MessageStyle::Ansi16LightNormal => Self::UserWhite,
            MessageStyle::Ansi16LightDebug => Self::UserCyan,
            MessageStyle::Ansi16LightGhost => Self::UserCyan,
            MessageStyle::Ansi16DarkError => Self::UserRed,
            MessageStyle::Ansi16DarkWarning => Self::UserMagenta,
            MessageStyle::Ansi16DarkEmphasis => Self::UserYellow,
            MessageStyle::Ansi16DarkHeading => Self::UserCyan,
            MessageStyle::Ansi16DarkSubheading => Self::UserGreen,
            MessageStyle::Ansi16DarkNormal => Self::UserWhite,
            MessageStyle::Ansi16DarkDebug => Self::UserCyan,
            MessageStyle::Ansi16DarkGhost => Self::LightGray,
            MessageStyle::Xterm256LightError => Self::GuardsmanRed,
            MessageStyle::Xterm256LightWarning => Self::DarkPurplePizzazz,
            MessageStyle::Xterm256LightEmphasis => Self::Copperfield,
            MessageStyle::Xterm256LightHeading => Self::MidnightBlue,
            MessageStyle::Xterm256LightSubheading => Self::ScienceBlue,
            MessageStyle::Xterm256LightNormal => Self::Black,
            MessageStyle::Xterm256LightDebug => Self::LochmaraBlue,
            MessageStyle::Xterm256LightGhost => Self::Boulder,
            MessageStyle::Xterm256DarkError => Self::GuardsmanRed,
            MessageStyle::Xterm256DarkWarning => Self::DarkViolet,
            MessageStyle::Xterm256DarkEmphasis => Self::Copperfield,
            MessageStyle::Xterm256DarkHeading => Self::CaribbeanGreen,
            MessageStyle::Xterm256DarkSubheading => Self::DarkMalibuBlue,
            MessageStyle::Xterm256DarkNormal => Self::Silver,
            MessageStyle::Xterm256DarkDebug => Self::BondiBlue,
            MessageStyle::Xterm256DarkGhost => Self::Silver,
        }
    }
}

#[allow(clippy::match_same_arms)]
impl From<MessageStyle> for Style {
    fn from(value: MessageStyle) -> Self {
        profile_fn!(style_from_msg_style);
        match value {
            MessageStyle::Ansi16LightError => Color::Red.bold(),
            MessageStyle::Ansi16LightWarning => Color::Magenta.bold(),
            MessageStyle::Ansi16LightEmphasis => Color::Yellow.bold(),
            MessageStyle::Ansi16LightHeading => Color::Blue.bold(),
            MessageStyle::Ansi16LightSubheading => Color::Cyan.bold(),
            MessageStyle::Ansi16LightNormal => Color::White.normal(),
            MessageStyle::Ansi16LightDebug => Color::Cyan.normal(),
            MessageStyle::Ansi16LightGhost => Color::Cyan.dimmed().italic(),
            MessageStyle::Ansi16DarkError => Color::Red.bold(),
            MessageStyle::Ansi16DarkWarning => Color::Magenta.bold(),
            MessageStyle::Ansi16DarkEmphasis => Color::Yellow.bold(),
            MessageStyle::Ansi16DarkHeading => Color::Cyan.bold(),
            MessageStyle::Ansi16DarkSubheading => Color::Green.bold(),
            MessageStyle::Ansi16DarkNormal => Color::White.normal(),
            MessageStyle::Ansi16DarkDebug => Color::Cyan.normal(),
            MessageStyle::Ansi16DarkGhost => Color::LightGray.dimmed().italic(),
            MessageStyle::Xterm256LightError => XtermColor::GuardsmanRed.get_color().bold(),
            MessageStyle::Xterm256LightWarning => XtermColor::DarkPurplePizzazz.get_color().bold(),
            MessageStyle::Xterm256LightEmphasis => XtermColor::Copperfield.get_color().bold(),
            MessageStyle::Xterm256LightHeading => XtermColor::MidnightBlue.get_color().bold(),
            MessageStyle::Xterm256LightSubheading => XtermColor::ScienceBlue.get_color().normal(),
            MessageStyle::Xterm256LightNormal => XtermColor::Black.get_color().normal(),
            MessageStyle::Xterm256LightDebug => XtermColor::LochmaraBlue.get_color().normal(),
            MessageStyle::Xterm256LightGhost => XtermColor::Boulder.get_color().normal().italic(),
            MessageStyle::Xterm256DarkError => XtermColor::GuardsmanRed.get_color().bold(),
            MessageStyle::Xterm256DarkWarning => XtermColor::DarkViolet.get_color().bold(),
            MessageStyle::Xterm256DarkEmphasis => XtermColor::Copperfield.get_color().bold(),
            MessageStyle::Xterm256DarkHeading => XtermColor::CaribbeanGreen.get_color().bold(),
            MessageStyle::Xterm256DarkSubheading => XtermColor::DarkMalibuBlue.get_color().normal(),
            MessageStyle::Xterm256DarkNormal => XtermColor::Silver.get_color().normal(),
            MessageStyle::Xterm256DarkDebug => XtermColor::BondiBlue.get_color().normal(),
            MessageStyle::Xterm256DarkGhost => XtermColor::Silver.get_color().normal().italic(),
        }
    }
}

/// Define the implementation of the `NuThemeStyle` trait for `MessageStyle` to facilitate
/// resolution of the `MessageStyle` variant to an `nu_ansi_term::Style`.
#[allow(clippy::match_same_arms)]
impl NuThemeStyle for MessageStyle {
    fn get_style(&self) -> Style {
        profile_fn!(nu_get_style);
        match self {
            Self::Ansi16LightError => Color::Red.bold(),
            Self::Ansi16LightWarning => Color::Magenta.bold(),
            Self::Ansi16LightEmphasis => Color::Yellow.bold(),
            Self::Ansi16LightHeading => Color::Blue.bold(),
            Self::Ansi16LightSubheading => Color::Cyan.bold(),
            Self::Ansi16LightNormal => Color::White.normal(),
            Self::Ansi16LightDebug => Color::Cyan.normal(),
            Self::Ansi16LightGhost => Color::Cyan.dimmed().italic(),
            Self::Ansi16DarkError => Color::Red.bold(),
            Self::Ansi16DarkWarning => Color::Magenta.bold(),
            Self::Ansi16DarkEmphasis => Color::Yellow.bold(),
            Self::Ansi16DarkHeading => Color::Cyan.bold(),
            Self::Ansi16DarkSubheading => Color::Green.bold(),
            Self::Ansi16DarkNormal => Color::White.normal(),
            Self::Ansi16DarkDebug => Color::Cyan.normal(),
            Self::Ansi16DarkGhost => Color::LightGray.dimmed().italic(),
            Self::Xterm256LightError => XtermColor::GuardsmanRed.get_color().bold(),
            Self::Xterm256LightWarning => XtermColor::DarkPurplePizzazz.get_color().bold(),
            Self::Xterm256LightEmphasis => XtermColor::Copperfield.get_color().bold(),
            Self::Xterm256LightHeading => XtermColor::MidnightBlue.get_color().bold(),
            Self::Xterm256LightSubheading => XtermColor::ScienceBlue.get_color().normal(),
            Self::Xterm256LightNormal => XtermColor::Black.get_color().normal(),
            Self::Xterm256LightDebug => XtermColor::LochmaraBlue.get_color().normal(),
            Self::Xterm256LightGhost => XtermColor::Boulder.get_color().normal().italic(),
            Self::Xterm256DarkError => XtermColor::GuardsmanRed.get_color().bold(),
            Self::Xterm256DarkWarning => XtermColor::DarkViolet.get_color().bold(),
            Self::Xterm256DarkEmphasis => XtermColor::Copperfield.get_color().bold(),
            Self::Xterm256DarkHeading => XtermColor::CaribbeanGreen.get_color().bold(),
            Self::Xterm256DarkSubheading => XtermColor::DarkMalibuBlue.get_color().normal(),
            Self::Xterm256DarkNormal => XtermColor::Silver.get_color().normal(),
            Self::Xterm256DarkDebug => XtermColor::BondiBlue.get_color().normal(),
            Self::Xterm256DarkGhost => XtermColor::Silver.get_color().normal().italic(),
        }
    }
}

/// Determine what message colour and style to use based on the current terminal's level of
/// colour support and light or dark theme, and the category of message to be displayed.
#[must_use]
pub fn nu_resolve_style(message_level: Lvl) -> Style {
    profile_fn!(nu_resolve_style);
    NuThemeStyle::get_style(&Into::<MessageStyle>::into(&message_level))
}

/// Main function for use by testing or the script runner.
#[allow(dead_code)]
pub fn main() {
    #[cfg(not(target_os = "windows"))]
    {
        #[allow(unused_variables)]
        let term = terminal();
        // shared::clear_screen();

        #[cfg(debug_assertions)]
        debug_log!("  Term : {term:?}");
    }

    let (maybe_color_support, _term_theme) = coloring();

    match maybe_color_support {
        None => {
            log!(Verbosity::Normal, "No colour support found for terminal");
        }
        Some(support) => {
            log!(
                Verbosity::Normal,
                "{}",
                nu_resolve_style(Lvl::Warning).paint("Colored Warning message\n")
            );

            for variant in MessageStyle::iter() {
                let variant_string: &str = &variant.to_string();
                log!(
                    Verbosity::Normal,
                    "My {} message",
                    variant.get_style().paint(variant_string)
                );
            }

            if matches!(support, ColorSupport::Xterm256) {
                log!(Verbosity::Normal, "");
                XtermColor::iter().for_each(|variant| {
                    let color = variant.get_color();
                    log!(Verbosity::Normal, "{}", color.paint(variant.to_string()));
                });
            }
        }
    }
}

/// An enum of the colours in a 256-colour palette, per the naming in `https://docs.rs/owo-colors/latest/owo_colors/colors/xterm/index.html#`.
#[allow(dead_code)]
#[derive(Display, EnumIter)]
pub enum XtermColor {
    UserBlack,
    UserRed,
    UserGreen,
    UserYellow,
    UserBlue,
    UserMagenta,
    UserCyan,
    UserWhite,
    UserBrightBlack,
    UserBrightRed,
    UserBrightGreen,
    UserBrightYellow,
    UserBrightBlue,
    UserBrightMagenta,
    UserBrightCyan,
    UserBrightWhite,
    Black,
    StratosBlue,
    NavyBlue,
    MidnightBlue,
    DarkBlue,
    Blue,
    CamaroneGreen,
    BlueStone,
    OrientBlue,
    EndeavourBlue,
    ScienceBlue,
    BlueRibbon,
    JapaneseLaurel,
    DeepSeaGreen,
    Teal,
    DeepCerulean,
    LochmaraBlue,
    AzureRadiance,
    LightJapaneseLaurel,
    Jade,
    PersianGreen,
    BondiBlue,
    Cerulean,
    LightAzureRadiance,
    DarkGreen,
    Malachite,
    CaribbeanGreen,
    LightCaribbeanGreen,
    RobinEggBlue,
    Aqua,
    Green,
    DarkSpringGreen,
    SpringGreen,
    LightSpringGreen,
    BrightTurquoise,
    Cyan,
    Rosewood,
    PompadourMagenta,
    PigmentIndigo,
    DarkPurple,
    ElectricIndigo,
    ElectricPurple,
    VerdunGreen,
    ScorpionOlive,
    Lilac,
    ScampiIndigo,
    Indigo,
    DarkCornflowerBlue,
    DarkLimeade,
    GladeGreen,
    JuniperGreen,
    HippieBlue,
    HavelockBlue,
    CornflowerBlue,
    Limeade,
    FernGreen,
    SilverTree,
    Tradewind,
    ShakespeareBlue,
    DarkMalibuBlue,
    DarkBrightGreen,
    DarkPastelGreen,
    PastelGreen,
    DownyTeal,
    Viking,
    MalibuBlue,
    BrightGreen,
    DarkScreaminGreen,
    ScreaminGreen,
    DarkAquamarine,
    Aquamarine,
    LightAquamarine,
    Maroon,
    DarkFreshEggplant,
    LightFreshEggplant,
    Purple,
    ElectricViolet,
    LightElectricViolet,
    Brown,
    CopperRose,
    StrikemasterPurple,
    DelugePurple,
    DarkMediumPurple,
    DarkHeliotropePurple,
    Olive,
    ClayCreekOlive,
    DarkGray,
    WildBlueYonder,
    ChetwodeBlue,
    SlateBlue,
    LightLimeade,
    ChelseaCucumber,
    BayLeaf,
    GulfStream,
    PoloBlue,
    LightMalibuBlue,
    Pistachio,
    LightPastelGreen,
    DarkFeijoaGreen,
    VistaBlue,
    Bermuda,
    DarkAnakiwaBlue,
    ChartreuseGreen,
    LightScreaminGreen,
    DarkMintGreen,
    MintGreen,
    LighterAquamarine,
    AnakiwaBlue,
    BrightRed,
    DarkFlirt,
    Flirt,
    LightFlirt,
    DarkViolet,
    BrightElectricViolet,
    RoseofSharonOrange,
    MatrixPink,
    TapestryPink,
    FuchsiaPink,
    MediumPurple,
    Heliotrope,
    PirateGold,
    MuesliOrange,
    PharlapPink,
    Bouquet,
    Lavender,
    LightHeliotrope,
    BuddhaGold,
    OliveGreen,
    HillaryOlive,
    SilverChalice,
    WistfulLilac,
    MelroseLilac,
    RioGrandeGreen,
    ConiferGreen,
    Feijoa,
    PixieGreen,
    JungleMist,
    LightAnakiwaBlue,
    Lime,
    GreenYellow,
    LightMintGreen,
    Celadon,
    AeroBlue,
    FrenchPassLightBlue,
    GuardsmanRed,
    RazzmatazzCerise,
    MediumVioletRed,
    HollywoodCerise,
    DarkPurplePizzazz,
    BrighterElectricViolet,
    TennOrange,
    RomanOrange,
    CranberryPink,
    HopbushPink,
    Orchid,
    LighterHeliotrope,
    MangoTango,
    Copperfield,
    SeaPink,
    CanCanPink,
    LightOrchid,
    BrightHeliotrope,
    DarkCorn,
    DarkTachaOrange,
    TanBeige,
    ClamShell,
    ThistlePink,
    Mauve,
    Corn,
    TachaOrange,
    DecoOrange,
    PaleGoldenrod,
    AltoBeige,
    FogPink,
    ChartreuseYellow,
    Canary,
    Honeysuckle,
    ReefPaleYellow,
    SnowyMint,
    OysterBay,
    Red,
    DarkRose,
    Rose,
    LightHollywoodCerise,
    PurplePizzazz,
    Fuchsia,
    BlazeOrange,
    BittersweetOrange,
    WildWatermelon,
    DarkHotPink,
    HotPink,
    PinkFlamingo,
    FlushOrange,
    Salmon,
    VividTangerine,
    PinkSalmon,
    DarkLavenderRose,
    BlushPink,
    YellowSea,
    TexasRose,
    Tacao,
    Sundown,
    CottonCandy,
    LavenderRose,
    Gold,
    Dandelion,
    GrandisCaramel,
    Caramel,
    CosmosSalmon,
    PinkLace,
    Yellow,
    LaserLemon,
    DollyYellow,
    PortafinoYellow,
    Cumulus,
    White,
    DarkCodGray,
    CodGray,
    LightCodGray,
    DarkMineShaft,
    MineShaft,
    LightMineShaft,
    DarkTundora,
    Tundora,
    ScorpionGray,
    DarkDoveGray,
    DoveGray,
    Boulder,
    Gray,
    LightGray,
    DustyGray,
    NobelGray,
    DarkSilverChalice,
    LightSilverChalice,
    DarkSilver,
    Silver,
    DarkAlto,
    Alto,
    Mercury,
    GalleryGray,
}

impl NuColor for XtermColor {
    fn get_color(&self) -> Color {
        Color::Fixed(self.get_fixed_code())
    }

    #[allow(clippy::too_many_lines)]
    fn get_fixed_code(&self) -> u8 {
        match self {
            Self::UserBlack => 0,
            Self::UserRed => 1,
            Self::UserGreen => 2,
            Self::UserYellow => 3,
            Self::OrientBlue => 24,
            Self::EndeavourBlue => 25,
            Self::ScienceBlue => 26,
            Self::BlueRibbon => 27,
            Self::JapaneseLaurel => 28,
            Self::DeepSeaGreen => 29,
            Self::Teal => 30,
            Self::DeepCerulean => 31,
            Self::LochmaraBlue => 32,
            Self::AzureRadiance => 33,
            Self::LightJapaneseLaurel => 34,
            Self::Jade => 35,
            Self::PersianGreen => 36,
            Self::BondiBlue => 37,
            Self::Cerulean => 38,
            Self::LightAzureRadiance => 39,
            Self::DarkGreen => 40,
            Self::Malachite => 41,
            Self::CaribbeanGreen => 42,
            Self::LightCaribbeanGreen => 43,
            Self::RobinEggBlue => 44,
            Self::Aqua => 45,
            Self::Green => 46,
            Self::DarkSpringGreen => 47,
            Self::SpringGreen => 48,
            Self::LightSpringGreen => 49,
            Self::BrightTurquoise => 50,
            Self::Cyan => 51,
            Self::Rosewood => 52,
            Self::PompadourMagenta => 53,
            Self::PigmentIndigo => 54,
            Self::DarkPurple => 55,
            Self::ElectricIndigo => 56,
            Self::ElectricPurple => 57,
            Self::VerdunGreen => 58,
            Self::ScorpionOlive => 59,
            Self::Lilac => 60,
            Self::ScampiIndigo => 61,
            Self::Indigo => 62,
            Self::DarkCornflowerBlue => 63,
            Self::DarkLimeade => 64,
            Self::GladeGreen => 65,
            Self::JuniperGreen => 66,
            Self::HippieBlue => 67,
            Self::HavelockBlue => 68,
            Self::CornflowerBlue => 69,
            Self::Limeade => 70,
            Self::FernGreen => 71,
            Self::SilverTree => 72,
            Self::Tradewind => 73,
            Self::ShakespeareBlue => 74,
            Self::DarkMalibuBlue => 75,
            Self::DarkBrightGreen => 76,
            Self::DarkPastelGreen => 77,
            Self::PastelGreen => 78,
            Self::DownyTeal => 79,
            Self::Viking => 80,
            Self::MalibuBlue => 81,
            Self::BrightGreen => 82,
            Self::DarkScreaminGreen => 83,
            Self::ScreaminGreen => 84,
            Self::DarkAquamarine => 85,
            Self::Aquamarine => 86,
            Self::LightAquamarine => 87,
            Self::Maroon => 88,
            Self::DarkFreshEggplant => 89,
            Self::LightFreshEggplant => 90,
            Self::Purple => 91,
            Self::ElectricViolet => 92,
            Self::LightElectricViolet => 93,
            Self::Brown => 94,
            Self::CopperRose => 95,
            Self::StrikemasterPurple => 96,
            Self::DelugePurple => 97,
            Self::DarkMediumPurple => 98,
            Self::DarkHeliotropePurple => 99,
            Self::Olive => 100,
            Self::ClayCreekOlive => 101,
            Self::DarkGray => 102,
            Self::WildBlueYonder => 103,
            Self::ChetwodeBlue => 104,
            Self::SlateBlue => 105,
            Self::LightLimeade => 106,
            Self::ChelseaCucumber => 107,
            Self::BayLeaf => 108,
            Self::GulfStream => 109,
            Self::PoloBlue => 110,
            Self::LightMalibuBlue => 111,
            Self::Pistachio => 112,
            Self::LightPastelGreen => 113,
            Self::DarkFeijoaGreen => 114,
            Self::VistaBlue => 115,
            Self::Bermuda => 116,
            Self::DarkAnakiwaBlue => 117,
            Self::ChartreuseGreen => 118,
            Self::LightScreaminGreen => 119,
            Self::DarkMintGreen => 120,
            Self::MintGreen => 121,
            Self::LighterAquamarine => 122,
            Self::AnakiwaBlue => 123,
            Self::BrightRed => 124,
            Self::DarkFlirt => 125,
            Self::Flirt => 126,
            Self::LightFlirt => 127,
            Self::DarkViolet => 128,
            Self::BrightElectricViolet => 129,
            Self::RoseofSharonOrange => 130,
            Self::MatrixPink => 131,
            Self::UserBlue => 4,
            Self::UserMagenta => 5,
            Self::UserCyan => 6,
            Self::UserWhite => 7,
            Self::UserBrightBlack => 8,
            Self::UserBrightRed => 9,
            Self::UserBrightGreen => 10,
            Self::UserBrightYellow => 11,
            Self::UserBrightBlue => 12,
            Self::UserBrightMagenta => 13,
            Self::UserBrightCyan => 14,
            Self::UserBrightWhite => 15,
            Self::Black => 16,
            Self::StratosBlue => 17,
            Self::NavyBlue => 18,
            Self::MidnightBlue => 19,
            Self::DarkBlue => 20,
            Self::Blue => 21,
            Self::CamaroneGreen => 22,
            Self::BlueStone => 23,
            Self::TapestryPink => 132,
            Self::FuchsiaPink => 133,
            Self::MediumPurple => 134,
            Self::Heliotrope => 135,
            Self::PirateGold => 136,
            Self::MuesliOrange => 137,
            Self::PharlapPink => 138,
            Self::Bouquet => 139,
            Self::Lavender => 140,
            Self::LightHeliotrope => 141,
            Self::BuddhaGold => 142,
            Self::OliveGreen => 143,
            Self::HillaryOlive => 144,
            Self::SilverChalice => 145,
            Self::WistfulLilac => 146,
            Self::MelroseLilac => 147,
            Self::RioGrandeGreen => 148,
            Self::ConiferGreen => 149,
            Self::Feijoa => 150,
            Self::PixieGreen => 151,
            Self::JungleMist => 152,
            Self::LightAnakiwaBlue => 153,
            Self::Lime => 154,
            Self::GreenYellow => 155,
            Self::LightMintGreen => 156,
            Self::Celadon => 157,
            Self::AeroBlue => 158,
            Self::FrenchPassLightBlue => 159,
            Self::GuardsmanRed => 160,
            Self::RazzmatazzCerise => 161,
            Self::MediumVioletRed => 162,
            Self::HollywoodCerise => 163,
            Self::DarkPurplePizzazz => 164,
            Self::BrighterElectricViolet => 165,
            Self::TennOrange => 166,
            Self::RomanOrange => 167,
            Self::CranberryPink => 168,
            Self::HopbushPink => 169,
            Self::Orchid => 170,
            Self::LighterHeliotrope => 171,
            Self::MangoTango => 172,
            Self::Copperfield => 173,
            Self::SeaPink => 174,
            Self::CanCanPink => 175,
            Self::LightOrchid => 176,
            Self::BrightHeliotrope => 177,
            Self::DarkCorn => 178,
            Self::DarkTachaOrange => 179,
            Self::TanBeige => 180,
            Self::ClamShell => 181,
            Self::ThistlePink => 182,
            Self::Mauve => 183,
            Self::Corn => 184,
            Self::TachaOrange => 185,
            Self::DecoOrange => 186,
            Self::PaleGoldenrod => 187,
            Self::AltoBeige => 188,
            Self::FogPink => 189,
            Self::ChartreuseYellow => 190,
            Self::Canary => 191,
            Self::Honeysuckle => 192,
            Self::ReefPaleYellow => 193,
            Self::SnowyMint => 194,
            Self::OysterBay => 195,
            Self::Red => 196,
            Self::DarkRose => 197,
            Self::Rose => 198,
            Self::LightHollywoodCerise => 199,
            Self::PurplePizzazz => 200,
            Self::Fuchsia => 201,
            Self::BlazeOrange => 202,
            Self::BittersweetOrange => 203,
            Self::WildWatermelon => 204,
            Self::DarkHotPink => 205,
            Self::HotPink => 206,
            Self::PinkFlamingo => 207,
            Self::FlushOrange => 208,
            Self::Salmon => 209,
            Self::VividTangerine => 210,
            Self::PinkSalmon => 211,
            Self::DarkLavenderRose => 212,
            Self::BlushPink => 213,
            Self::YellowSea => 214,
            Self::TexasRose => 215,
            Self::Tacao => 216,
            Self::Sundown => 217,
            Self::CottonCandy => 218,
            Self::LavenderRose => 219,
            Self::Gold => 220,
            Self::Dandelion => 221,
            Self::GrandisCaramel => 222,
            Self::Caramel => 223,
            Self::CosmosSalmon => 224,
            Self::PinkLace => 225,
            Self::Yellow => 226,
            Self::LaserLemon => 227,
            Self::DollyYellow => 228,
            Self::PortafinoYellow => 229,
            Self::Cumulus => 230,
            Self::White => 231,
            Self::DarkCodGray => 232,
            Self::CodGray => 233,
            Self::LightCodGray => 234,
            Self::DarkMineShaft => 235,
            Self::MineShaft => 236,
            Self::LightMineShaft => 237,
            Self::DarkTundora => 238,
            Self::Tundora => 239,
            Self::ScorpionGray => 240,
            Self::DarkDoveGray => 241,
            Self::DoveGray => 242,
            Self::Boulder => 243,
            Self::Gray => 244,
            Self::LightGray => 245,
            Self::DustyGray => 246,
            Self::NobelGray => 247,
            Self::DarkSilverChalice => 248,
            Self::LightSilverChalice => 249,
            Self::DarkSilver => 250,
            Self::Silver => 251,
            Self::DarkAlto => 252,
            Self::Alto => 253,
            Self::Mercury => 254,
            Self::GalleryGray => 255,
        }
    }
}
