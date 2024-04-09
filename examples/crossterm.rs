//! [dependencies]
//! crossterm = "0.27.0"

use std::io::stdout;

use crossterm::{
    event, execute,
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

    println!("");

    // or using functions
    stdout()
        .execute(SetForegroundColor(Color::DarkBlue))?
        .execute(SetBackgroundColor(Color::Red))?
        .execute(Print("Styled text here."))?
        .execute(ResetColor)?;
    println!("");

    Ok(())
}
