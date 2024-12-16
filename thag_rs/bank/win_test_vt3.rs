/*[toml]
[dependencies]
crossterm = "0.28.1"
winapi = { version = "0.3.9", features = ["consoleapi", "processenv", "winbase"] }
*/

use crossterm::{event::{self, Event, KeyCode}, terminal, ExecutableCommand};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use winapi::um::consoleapi::SetConsoleMode;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::STD_OUTPUT_HANDLE;
use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

// Function to enable virtual terminal processing for Windows
#[cfg(windows)]
fn enable_virtual_terminal_processing() {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != INVALID_HANDLE_VALUE {
            let mut mode: u32 = 0;
            if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode) != 0 {
                SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

// Function to initialize virtual terminal for both Windows and non-Windows
#[cfg(windows)]
fn initialize_virtual_terminal() {
    enable_virtual_terminal_processing();
}

#[cfg(not(windows))]
fn initialize_virtual_terminal() {
    // Nothing to do on non-Windows systems
}

fn main() -> std::io::Result<()> {
    // Initialize virtual terminal support
    initialize_virtual_terminal();

    println!("\nQuerying terminal for colors...");
    query_terminal()?;

    println!("\nProgram complete. Exiting.");
    Ok(())
}

// Function to query terminal for current foreground/background colors
fn query_terminal() -> std::io::Result<()> {
    // Enable raw mode to suppress echoing to the terminal
    terminal::enable_raw_mode()?;

    // Send the color query (OSC 11 for background color)
    let mut stdout = stdout();
    print!("Querying background color: \x1B]11;?\x07");
    stdout.flush().unwrap();

    // Timeout duration
    let timeout = Duration::from_secs(2);
    let start = Instant::now();

    // Listen for terminal events using crossterm
    println!("Waiting for terminal response...");

    loop {
        // Exit if timeout duration is exceeded
        if start.elapsed() > timeout {
            println!("Timeout: No response from terminal.");
            break;
        }

        // Check if an event is available
        if event::poll(Duration::from_millis(100))? {
            // Read the next event
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    // Exit on backslash key
                    KeyCode::Char('\\') => {
                        println!("Exiting on '\\' key.");
                        break;
                    }
                    // Print any other keys pressed
                    _ => {
                        println!("Key pressed: {:?}", key_event);
                    }
                }
            }
        } else {
            // Continue polling for events if none are detected
        }
    }

    // Disable raw mode after we're done
    terminal::disable_raw_mode()?;
    Ok(())
}
