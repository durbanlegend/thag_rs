/*[toml]
[dependencies]
ratatui = "0.28.1"
*/

/// Published example from crossterm crate. Macro version of the example:
/// "Print a rectangle colored with magenta and use both direct execution and lazy execution."
/// Direct execution with `execute` and lazy execution with `queue`.
///
/// Url: https://docs.rs/crossterm/latest/crossterm/
//# Purpose: Demo `crossterm` command API.
//# Categories: crates, technique
//# Sample arguments: `-- true`
use std::{
    env,
    io::{stderr, Result},
    thread::sleep,
    time::Duration,
};

use ratatui::crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} [true | false]", args[0]);
        std::process::exit(1);
    }

    let should_enter_alternate_screen: bool =
        args[1].parse().expect("Please provide true or false");

    if should_enter_alternate_screen {
        stderr().execute(EnterAlternateScreen)?; // remove this line
    }

    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;

    terminal.draw(|f| {
        f.render_widget(Paragraph::new("Hello World!"), Rect::new(10, 20, 20, 1));
    })?;
    sleep(Duration::from_secs(2));

    if should_enter_alternate_screen {
        stderr().execute(LeaveAlternateScreen)?; // remove this line
    }
    Ok(())
}
