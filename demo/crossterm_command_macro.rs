/*[toml]
[dependencies]
crossterm = "0.28.1"
*/

use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal,
};
use std::io::{self, Write};

/// Published example from crossterm crate. Macro version of the example:
/// "Print a rectangle colored with magenta and use both direct execution and lazy execution."
/// Direct execution with `execute` and lazy execution with `queue`.
///
/// Url: https://docs.rs/crossterm/latest/crossterm/
//# Purpose: Demo `crossterm` command API.
//# Categories: crates, technique
use std::io::stdout;
fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    for y in 0..40 {
        for x in 0..150 {
            if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                // in this loop we are more efficient by not flushing the buffer.
                queue!(
                    stdout,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent("█".magenta())
                )?;
            }
        }
    }
    stdout.flush()?;
    Ok(())
}
