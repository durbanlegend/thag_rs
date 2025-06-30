/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "core", "simplelog"] }
*/

/// Demonstrate embellishing an enum of the 16 basic colours, to allow renaming, generating a descriptive
/// name, and instantiating a variant from its name string.
///
/// The `ansi_name` attribute is used to override the default name of "Bright Black" to the alternative name
/// "Dark Gray".
///
/// This proc macro was originally used by the `thag` `styling` module.
//# Purpose: Sample model of a basic attribute proc macro.
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::Display;
use thag_demo_proc_macros::AnsiCodeDerive;
use thag_rs::{errors::ThemeError, ThagError, ThagResult};

#[derive(Debug, Deserialize, Display, Clone, Copy, PartialEq, Eq, AnsiCodeDerive)]
#[serde(rename_all = "snake_case")]
pub enum AnsiCode {
    // Standard colors (30-37)
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,

    // High intensity colors (90-97)
    #[ansi_name("Dark Gray")]
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

impl AnsiCode {
    // Get the numeric code
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}


println!("Bright Yellow description should stay as is:");
let bright_yellow = AnsiCode::from_str("bright_yellow")?;
println!(
    "bright_yellow: variant={bright_yellow}, code={}, name={}",
    bright_yellow.code(),
    bright_yellow.name()
);

println!();
println!("Bright Black description should be overridden by Dark Gray:");
let bright_black = AnsiCode::from_str("bright_black")?;
println!(
    "bright_black: variant={bright_black}, code={}, name={}",
    bright_black.code(),
    bright_black.name()
);
