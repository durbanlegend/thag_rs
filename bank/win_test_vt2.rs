/*[toml]
[dependencies]
crossterm = "0.29"
winapi = { version = "0.3.9", features = ["consoleapi", "processenv", "winbase"] }
*/

use crossterm::{terminal, ExecutableCommand};
use std::io::{stdin, stdout, Read, Write};
use std::time::Duration;
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

    // Buffer to store the response
    let mut response = Vec::new();
    let mut stdin = stdin();

    // Set a timeout to avoid hanging indefinitely
    let timeout = Duration::from_millis(500);

    // Read the terminal's response, byte by byte
    let mut buffer = [0; 1]; // Read one byte at a time
    let mut num_bytes_read = 0;

    // Keep reading until we encounter the end of the OSC sequence ('\x1B\\')
    loop {
        // Try to read a byte from stdin
        if let Ok(bytes_read) = stdin.read(&mut buffer) {
            if bytes_read > 0 {
                response.push(buffer[0]);
                num_bytes_read += 1;

                // Check if we have the end of the OSC sequence
                if response.ends_with(&[0x1B, b'\\']) {
                    break;
                }
            } else {
                // Stop if we don't receive any data within the timeout
                if num_bytes_read == 0 {
                    println!("Timeout or no response received.");
                    break;
                }
            }
        }
    }

    // Print the captured response as a string
    let response_str = String::from_utf8_lossy(&response);
    println!("Terminal response: {}", response_str);

    // Disable raw mode after we're done
    terminal::disable_raw_mode()?;
    Ok(())
}
