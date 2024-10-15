/*[toml]
[dependencies]
win32console = "0.1.5"
*/

#![cfg(target_os = "windows")]
use std::io::{self, Write};
use win32console::console::{ConsoleMode, WinConsole};

fn main() -> io::Result<()> {
    let old_mode = WinConsole::input().get_mode()?;
    let new_mode = old_mode | ConsoleMode::ENABLE_VIRTUAL_TERMINAL_INPUT;
    // We change the input mode so the characters are not displayed
    let _ = WinConsole::input().set_mode(new_mode)?;
    println!("Virtual Terminal Input enabled!");

        // Write an OSC query to the console
    let mut stdout = io::stdout();
    write!(stdout, "\x1b]11;?\x1b\\")?;
    stdout.flush()?;

    let _ = WinConsole::input().set_mode(old_mode)?;

    println!("Virtual Terminal Input disabled!");

    Ok(())
}
