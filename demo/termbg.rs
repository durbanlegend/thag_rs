/*[toml]
[dependencies]
crossterm = "0.27.0"
termbg = "0.5.0"
*/

/// Published example from `termbg` readme, only I've added the `clear_screen` method
/// because `termbg` messes with the terminal settings. Obviously that doesn't matter
/// in a demo that exists before doing serious work, but having struggled with a
/// persistent issue of rightward drift in println! output that turned out to be
/// caused by an overlooked termbg call buried in the message colour resolution logic,
/// I think it's important to make it a habit.
///
/// Detects the light or dark theme in use, as well as the colours in use.
//# Demo theme detection with `termbg` and clearing terminal state with `crossterm`.
use crossterm::{
    cursor::{MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Write};

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
pub fn clear_screen() {
    let mut out = stdout();
    out.execute(Clear(ClearType::All)).unwrap();
    out.execute(MoveTo(0, 0)).unwrap();
    out.execute(Show).unwrap();
    out.flush().unwrap();
}

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let term = termbg::terminal();
    let rgb = termbg::rgb(timeout);
    let theme = termbg::theme(timeout);
    clear_screen();

    println!("  Term : {:?}", term);

    match rgb {
        Ok(rgb) => {
            println!("  Color: R={:x}, G={:x}, B={:x}", rgb.r, rgb.g, rgb.b);
        }
        Err(e) => {
            println!("  Color: detection failed {:?}", e);
        }
    }

    match theme {
        Ok(theme) => {
            println!("  Theme: {:?}", theme);
        }
        Err(e) => {
            println!("  Theme: detection failed {:?}", e);
        }
    }
}
