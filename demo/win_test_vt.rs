/*[toml]
[dependencies]
crossterm = "0.28.1"
winapi = { version = "0.3.9", features = ["consoleapi", "processenv", "winbase"] }
*/

/// Exploration of `Windows Terminal` virtual terminal processing with respect to the `termbg` crate.
/// `termbg` comment states: "Windows Terminal is Xterm-compatible"
/// https://github.com/microsoft/terminal/issues/3718.
/// Unfortunately it turns out that this is only partially true and misleading, because
/// this compatibility excludes OSC 10/11 colour queries until Windows Terminal 1.22,
/// which was only released in preview in August 2024.
/// https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release/:
/// "Applications can now query ... the default foreground (OSC 10 ?) [and] background (OSC 11 ?)"
/// Another finding is that WT_SESSION is not recommended as a marker for VT capabilities:
/// https://github.com/Textualize/rich/issues/140.
/// Also, but out of scope of this script, there is no good fallback detection method provided by Windows,
/// as per my comments in the adapted module `thag_rs::termbg`. Unless you have WT 1.22 or higher as above,
/// the best bet for supporting colour schemes other than the default black is to fallback to using a
/// configuration file (as we do) or allowing the user to specify the theme in real time.
/// Finally, the `termbg` crate was swallowing the first character of input in Windows and causing a
/// "rightward march" of log output due to suppression of carriage returns in all environments. I've
/// addressed the former by using non-blocking `crossterm` event reads instead of `stdin`, and also
/// introduced a
//# Purpose: Debug `termbg`
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
#[cfg(target_os = "windows")]
use {
    winapi::um::consoleapi::SetConsoleMode, winapi::um::handleapi::INVALID_HANDLE_VALUE,
    winapi::um::processenv::GetStdHandle, winapi::um::winbase::STD_OUTPUT_HANDLE,
    winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING,
};

// Function to enable virtual terminal processing for Windows
#[cfg(target_os = "windows")]
fn enable_virtual_terminal_processing() -> bool {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != INVALID_HANDLE_VALUE {
            let mut mode: u32 = 0;
            if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode) != 0 {
                // Try to set virtual terminal processing mode
                if SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0 {
                    // Success in enabling VT
                    return true;
                } else {
                    // Failed to enable VT, optionally log error
                    eprintln!("Failed to enable Virtual Terminal Processing.");
                }
            }
        }
    }
    // Return false if enabling VT failed
    false
}

// Function to initialize virtual terminal for both Windows and non-Windows
#[cfg(windows)]
fn initialize_virtual_terminal() {
    if !enable_virtual_terminal_processing() {
        eprintln!(
            "Virtual Terminal Processing could not be enabled. Falling back to default behavior."
        );
        // Optionally, add fallback behavior here, such as forcing a default dark mode
    }
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
    println!("Querying background color...");
    print!("\x1b]11;?\x1b\\\r");
    stdout.flush().unwrap();

    // Buffer to store key events (characters)
    let mut response_buffer = String::new();

    // Timeout duration
    let timeout = Duration::from_secs(2); // Increased for better chance of response
    let start = Instant::now();

    // Listen for terminal events using crossterm
    println!("Waiting for terminal response...\r");

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
                    // End on backslash character
                    KeyCode::Char('\\') => {
                        println!("End of response detected (backslash character).");
                        response_buffer.push('\\');
                        break;
                    }
                    // Append other characters to buffer
                    KeyCode::Char(c) => {
                        response_buffer.push(c);
                    }
                    _ => {
                        // Ignore other keys
                    }
                }
            }
        }
    }

    // Disable raw mode after we're done
    terminal::disable_raw_mode()?;

    // Print the full response buffer
    println!("\nTerminal response: {}", response_buffer);

    // Print the duration it took to capture the response
    let elapsed = start.elapsed();
    println!("Elapsed time: {:.2?}", elapsed);

    Ok(())
}
