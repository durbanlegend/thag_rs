use crate::{debug_log, profile, ThagResult};
use crossterm::terminal;
use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use supports_color::Stream;
use termbg::{theme, Theme};

/// An enum to categorise the current terminal's level of colour support as detected, configured
/// or defaulted.
///
/// We fold `TrueColor` into Xterm256 as we're not interested in more than 256
/// colours just for messages.
#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    Display,
    Documented,
    DocumentedVariants,
    EnumIter,
    EnumString,
    IntoStaticStr,
    PartialEq,
    Eq,
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
pub enum ColorSupport {
    /// Full color support, suitable for color palettes of 256 colours (16 bit) or higher.
    Xterm256,
    /// Basic 16-color support
    Ansi16,
    /// No color support
    None,
    /// Auto-detect from terminal
    #[default]
    AutoDetect,
}

/// An enum to categorise the current terminal's light or dark theme as detected, configured
/// or defaulted.
#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    Documented,
    DocumentedVariants,
    Display,
    EnumIter,
    EnumString,
    IntoStaticStr,
    PartialEq,
    Eq,
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
pub enum TermTheme {
    /// Light background terminal
    Light,
    /// Dark background terminal (default)
    #[default]
    Dark,
    /// Let `thag` autodetect the background luminosity
    AutoDetect,
}

/// Represents different message/content levels for styling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    Error,
    Warning,
    Heading, // HEAD in the original
    Subheading,
    Emphasis,
    Bright,
    Normal,
    Debug,
    Ghost,
}

// We can implement conversions to u8 directly here
impl From<&Level> for u8 {
    fn from(level: &Level) -> u8 {
        match level {
            Level::Error => 160,     // GuardsmanRed
            Level::Warning => 164,   // DarkPurplePizzazz
            Level::Heading => 19,    // MidnightBlue
            Level::Subheading => 26, // ScienceBlue
            Level::Emphasis => 173,  // Copperfield
            Level::Bright => 46,     // Green
            Level::Normal => 16,     // Black
            Level::Debug => 32,      // LochmaraBlue
            Level::Ghost => 232,     // DarkCodGray
        }
    }
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
    profile!("get_color_level");
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
pub fn get_color_level() -> Option<ColorSupport> {
    profile!("get_color_level");
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
    profile!("env_force_color");
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
    profile!("env_no_color");
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
    profile!("supports_color");
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

pub fn resolve_term_theme() -> ThagResult<TermTheme> {
    profile!("resolve_term_theme");
    let raw_before = terminal::is_raw_mode_enabled()?;
    #[cfg(debug_assertions)]
    debug_log!("About to call termbg");
    let timeout = std::time::Duration::from_millis(1000);

    // #[cfg(debug_assertions)]
    // debug_log!("Check terminal background color");
    let theme = theme(timeout);
    debug_log!("Found theme {theme:?}");
    maybe_restore_raw_status(raw_before)?;

    match theme {
        Ok(Theme::Light) => Ok(TermTheme::Light),
        Ok(Theme::Dark) | Err(_) => Ok(TermTheme::Dark),
    }
}

fn maybe_restore_raw_status(raw_before: bool) -> ThagResult<()> {
    profile!("maybe_restore_raw_status");
    let raw_after = terminal::is_raw_mode_enabled()?;
    if raw_before == raw_after {
        debug_log!("No need to restore raw status");
    } else {
        debug_log!("Restored raw status");
        restore_raw_status(raw_before)?;
    }
    Ok(())
}

pub fn restore_raw_status(raw_before: bool) -> ThagResult<()> {
    profile!("restore_raw_status");
    if raw_before {
        terminal::enable_raw_mode()?;
    } else {
        terminal::disable_raw_mode()?;
    }
    Ok(())
}
