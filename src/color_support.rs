use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

// /// Represents the level of color support available in the terminal
// #[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize)]
// pub enum ColorSupport {
//     /// No color support
//     None,
//     /// Basic 16-color ANSI support
//     Ansi16,
//     /// 256-color support
//     Xterm256,
//     /// Auto-detect color support
//     AutoDetect,
// }

// /// Represents the terminal theme (light or dark)
// #[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize)]
// pub enum TermTheme {
//     /// Light theme
//     Light,
//     /// Dark theme
//     Dark,
//     /// Auto-detect theme
//     AutoDetect,
// }

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
