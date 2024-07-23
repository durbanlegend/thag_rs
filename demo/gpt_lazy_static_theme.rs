/*[toml]
[dependencies]
crossterm = "0.27.0"
lazy_static = "1.4.0"
termbg = "0.5.0"
*/

/// Prototype of detecting the light or dark theme in use, and registering it
/// as a static enum value for use in message style selection. Example of using
/// an LLM to generate a prototype to a simple spec. The `clear_screen` function
/// was added manually later. This prototype is one of many that was incorporated
/// into `rs_script`.
//# Demo theme detection with `termbg`, clearing terminal state with `crossterm` and setting it as a static enum value using `lazy_static`.
use crossterm::{
    cursor::{MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use lazy_static::lazy_static;
use std::io::{stdout, Write};
use termbg::Theme;

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
pub fn clear_screen() {
    let mut out = stdout();
    out.execute(Clear(ClearType::All)).unwrap();
    out.execute(MoveTo(0, 0)).unwrap();
    out.execute(Show).unwrap();
    out.flush().unwrap();
}

#[derive(Debug, PartialEq)]
enum TermTheme {
    Light,
    Dark,
}

lazy_static! {
    static ref TERM_THEME: TermTheme = {
        let timeout = std::time::Duration::from_millis(100);
        // debug!("Check terminal background color");
        let theme = termbg::theme(timeout);
        clear_screen();
        match theme {
            Ok(Theme::Light) => TermTheme::Light,
            Ok(Theme::Dark) | Err(_) => TermTheme::Dark,
        }
    };
}

fn main() {
    // Directly match the static variable without a mutex
    match *TERM_THEME {
        TermTheme::Light => println!("The theme is Light"),
        TermTheme::Dark => println!("The theme is Dark"),
    }
}
