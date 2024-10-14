use std::io::{self, Write};
use win32console::console::Console;
use win32console::console::InputMode;

fn main() -> io::Result<()> {
    // Open the current console
    let console = Console::from_std_handle(io::stdin())?;

    // Get the current input mode flags
    let mut mode = console.mode()?;

    // Set the ENABLE_VIRTUAL_TERMINAL_INPUT flag
    mode |= InputMode::ENABLE_VIRTUAL_TERMINAL_INPUT;

    // Apply the new input mode
    console.set_mode(mode)?;

    // Write an OSC query to the console
    let mut stdout = io::stdout();
    write!(stdout, "\x1b]11;?\x1b\\")?;
    stdout.flush()?;

    println!("Virtual Terminal Input enabled!");

    Ok(())
}
