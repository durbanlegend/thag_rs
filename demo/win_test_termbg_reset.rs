/*[toml]
[dependencies]
crossterm = "0.28.1"
termbg = "0.5.0"
*/

/// This still seems to "reliably" swallow the very first character entered in Windows.
/// The `crossterm` reset doesn't seem to help. My disappointment is immeasurable and
/// my day is ruined.
//# Purpose: Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.
use crossterm::{
    cursor::{MoveToColumn, Show},
    ExecutableCommand,
};

use std::io::{self, stdout, Read, Write};

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
// # Panics
// Will panic if a crossterm execute command fails.
pub fn clear_screen() {
    // let mut out = stdout();
    // out.execute(MoveToColumn(0)).unwrap();
    // out.execute(Show).unwrap();
    // out.flush().unwrap();
}

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    // let term = termbg::terminal();
    let _ = termbg::rgb(timeout);
    // let theme = termbg::theme(timeout);
    // clear_screen();

    println!("Run with -qq in Windows Terminal to suppress colored lines, type in something and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
