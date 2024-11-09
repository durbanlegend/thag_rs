/*[toml]
[dependencies]
crossterm = "0.28.1"
simplelog = "0.12.2"
#termbg = "0.6.0"
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", rev = "6c5bff14435ebf308795dc5b62e04ea8e8d1e99e" }
*/

/// Published example from `termbg` readme.
///
/// Detects the light or dark theme in use, as well as the colours in use.
//# Purpose: Demo theme detection with `termbg` and clearing terminal state with `crossterm`.
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::env;
use thag_rs::termbg;

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
fn main() {
    let args: Vec<String> = env::args().collect();
    let num_args = args.len();
    match num_args {
        1 => (),
        2 if args[1] == "-d" || args[1] == "--debug" => {
            CombinedLogger::init(vec![TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )])
            .unwrap();
        }
        _ => {
            eprintln!("Usage: {} [--debug/-d]", args[0]);
            std::process::exit(1);
        }
    }

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
