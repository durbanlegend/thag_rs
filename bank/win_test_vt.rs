/*[toml]
[dependencies]
crossterm = "0.28.1"
winapi = { version = "0.3.9", features = ["consoleapi", "processenv", "winbase"] }
*/

use crossterm::{execute, queue, style::*, terminal::*, ExecutableCommand};
use std::io::{stdin, stdout, BufRead, Write};
use std::thread::sleep;
use std::time::Duration;

// Function to enable virtual terminal processing for Windows
#[cfg(windows)]
fn enable_virtual_terminal_processing() {
    use std::os::windows::io::AsRawHandle;
    use winapi::um::consoleapi::SetConsoleMode;
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;
    use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

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

fn main() {
    // Initialize virtual terminal support
    initialize_virtual_terminal();

    let mut stdout = stdout();

    // Test writing formatted strings
    println!("Testing virtual terminal support with formatted output:");

    // Change text color to Blue, background to White
    stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
    stdout.execute(SetBackgroundColor(Color::White)).unwrap();
    println!("This is blue text on a white background.");

    // Reset formatting
    stdout.execute(ResetColor).unwrap();

    // Test text formatting
    stdout.execute(SetAttribute(Attribute::Bold)).unwrap();
    println!("This is bold text.");
    stdout.execute(SetAttribute(Attribute::Underlined)).unwrap();
    println!("This is underlined text.");
    stdout.execute(ResetColor).unwrap();

    // Query foreground and background colors (using ANSI sequences)
    println!("\nQuerying terminal for colors and cursor position...");
    query_terminal();

    println!("\nProgram complete. Exiting.");
}

// Function to query terminal for current foreground/background colors and cursor position
fn query_terminal() {
    let mut stdout = stdout();

    // // Query current cursor position
    // queue!(stdout, Print("\x1B[6n")).unwrap(); // ANSI escape sequence to query cursor position
    // stdout.flush().unwrap();

    // // Give the terminal some time to respond
    // sleep(Duration::from_millis(100));

    // // Query foreground color
    // queue!(stdout, Print("\x1B]10;?\x07")).unwrap(); // Query foreground color
    // stdout.flush().unwrap();

    // sleep(Duration::from_millis(100));

    // Query background color
    queue!(stdout, Print("\x1B]11;?\x07")).unwrap(); // Query background color
    stdout.flush().unwrap();

    // Wait briefly for the terminal to respond
    sleep(Duration::from_millis(100));

    // Try reading response from stdin
    let mut input = String::new();
    stdin().lock().read_line(&mut input).unwrap();

    println!("\nQueries sent. If terminal supports responses, you should see them in the input buffer or terminal.");
}
