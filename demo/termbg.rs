/*[toml]
[dependencies]
crossterm = "0.28.1"
#termbg = "0.7.0"
thag_rs = { path = "C:/Users/donforbes/Documents/GitHub/thag_rs" }
*/

/// Published example from `termbg` readme.
///
/// Detects the light or dark theme in use, as well as the colours in use.
//# Purpose: Demo theme detection with `termbg` and clearing terminal state with `crossterm`.
// use crossterm::{
//     cursor::{MoveTo, Show},
//     terminal::{Clear, ClearType},
//     ExecutableCommand,
// };
// use std::io::{stdout, Write};
use thag_rs::termbg;

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
fn main() {
    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let term = termbg::terminal();
    let rgb = termbg::rgb(timeout);
    let theme = termbg::theme(timeout);

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
