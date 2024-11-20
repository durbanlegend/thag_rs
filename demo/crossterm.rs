/*[toml]
[dependencies]
crossterm = "0.28.1"
*/

/// Published example from crossterm crate.
///
/// Url: https://github.com/crossterm-rs/crossterm/blob/master/README.md
//# Purpose: Demo crossterm terminal manipulation.
//# Categories: crates, technique
use std::io::stdout;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

fn main() -> std::io::Result<()> {
    // using the macro
    execute!(
        stdout(),
        SetForegroundColor(Color::DarkBlue),
        SetBackgroundColor(Color::Yellow),
        Print("Styled text here."),
        ResetColor
    )?;

    println!();

    // or using functions
    stdout()
        .execute(SetForegroundColor(Color::DarkBlue))?
        .execute(SetBackgroundColor(Color::Red))?
        .execute(Print("Styled text here."))?
        .execute(ResetColor)?;
    println!("");

    Ok(())
}
