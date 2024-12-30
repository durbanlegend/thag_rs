use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

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
