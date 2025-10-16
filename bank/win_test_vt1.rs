/*[toml]
[dependencies]
# crossterm = "0.29"
winapi = { version = "0.3.9", features = ["consoleapi", "handleapi", "processenv", "winbase"] }
*/

// use crossterm::{style::*, ExecutableCommand};
use std::io::{stderr, stdin, stdout, BufRead, Result, Write};
// use std::thread::sleep;
// use std::time::Duration;

// Function to enable virtual terminal processing for Windows
#[cfg(windows)]
fn enable_virtual_terminal_processing() {
    // use std::os::windows::io::AsRawHandle;
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

fn main() -> Result<()> {
    // Initialize virtual terminal support
    initialize_virtual_terminal();

    let mut stdout = stdout();

    // Test writing formatted strings
    println!("Testing virtual terminal support with formatted output:");

    // // Change text color to Blue, background to White
    // stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
    // stdout.execute(SetBackgroundColor(Color::White)).unwrap();
    // println!("This is blue text on a white background.");

    // // Reset formatting
    // stdout.execute(ResetColor).unwrap();

    // // Test text formatting
    // stdout.execute(SetAttribute(Attribute::Bold)).unwrap();
    // println!("This is bold text.");
    // stdout.execute(SetAttribute(Attribute::Underlined)).unwrap();
    // println!("This is underlined text.");
    // stdout.execute(ResetColor).unwrap();

    // Try some Set Graphics Rendition (SGR) terminal escape sequences
    let mut stderr = stderr();
    writeln!(
        stderr,
        "\x1b[31mThis text has a red foreground using SGR.31."
    )?;
    writeln!(stderr, "\x1b[1mThis text has a bright (bold) red foreground using SGR.1 to affect the previous color setting.")?;
    writeln!(
        stderr,
        "\x1b[mThis text has returned to default colors using SGR.0 implicitly."
    )?;
    writeln!(
        stderr,
        "\x1b[34;46mThis text shows the foreground and background change at the same time."
    )?;
    writeln!(
        stderr,
        "\x1b[0mThis text has returned to default colors using SGR.0 explicitly."
    )?;
    writeln!(stderr, "\x1b[31;32;33;34;35;36;101;102;103;104;105;106;107mThis text attempts to apply many colors in the same command. Note the colors are applied from left to right so only the right-most option of foreground cyan (SGR.36) and background bright white (SGR.107) is effective.")?;
    writeln!(
        stderr,
        "\x1b[39mThis text has restored the foreground color only."
    )?;
    writeln!(
        stderr,
        "\x1b[49mThis text has restored the background color only."
    )?;

    // print!("Setting background color: \x1B]11;rgb:00/00/00;\x07"); // ANSI sequence for querying background
    // print!("Querying background color: \x1B]11;rgb:00/00/00;\x07"); // ANSI sequence for querying background
    stdout.flush().unwrap();

    // Query foreground and background colors (using ANSI sequences)
    println!("\nQuerying terminal for colors and cursor position...");
    query_terminal();

    println!("\nProgram complete. Exiting.");
    Ok(())
}

// Function to query terminal for current foreground/background colors and cursor position
fn query_terminal() {
    // Send the color query
    let mut stdout = stdout();
    print!("Querying background color: \x1B]11;?\x07"); // ANSI sequence for querying background
                                                        // print!("Querying background color: \x1B]11;rgb:00/00/00;\x07"); // ANSI sequence for querying background
    stdout.flush().unwrap();

    // Try reading response from stdin
    let mut input = String::new();
    stdin().lock().read_line(&mut input).unwrap();

    // sleep(Duration::from_millis(100)); // Give terminal time to respond

    println!("Terminal response: {}", input); // Print the response

    println!("\nQueries sent. If terminal supports responses, you should see them in the input buffer or terminal.");
}
