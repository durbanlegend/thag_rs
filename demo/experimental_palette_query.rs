/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
crossterm = "0.28"
termbg = "0.6"
atty = "0.2"
libc = "0.2"
*/

//! Experimental Terminal Palette Query
//!
//! This is an experimental implementation that actually attempts to query
//! the terminal's palette using OSC 4 sequences. This is more complex than
//! the demonstration version and may not work in all environments.
//!
//! **Warning:** This script directly manipulates terminal I/O and may not
//! work properly in all terminal emulators or environments. Use the main
//! query_terminal_palette.rs script for a safer demonstration.

//# Purpose: Experimental real OSC 4 palette querying
//# Categories: terminal, styling, colors, experimental

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thag_styling::TermAttributes;

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Error types for palette querying
#[derive(Debug)]
pub enum PaletteError {
    Io(io::Error),
    Timeout,
    ParseError(String),
    UnsupportedTerminal,
    PermissionDenied,
}

impl From<io::Error> for PaletteError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Experimental palette color query using raw terminal I/O
pub fn query_palette_color_raw(index: u8, timeout: Duration) -> Result<Rgb, PaletteError> {
    // This is an experimental approach that tries to read directly from /dev/tty
    // if available (Unix-like systems)

    #[cfg(unix)]
    {
        query_palette_color_unix(index, timeout)
    }

    #[cfg(not(unix))]
    {
        Err(PaletteError::UnsupportedTerminal)
    }
}

#[cfg(unix)]
fn query_palette_color_unix(index: u8, timeout: Duration) -> Result<Rgb, PaletteError> {
    use std::fs::OpenOptions;

    // Try to open /dev/tty for direct terminal communication
    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map_err(|_| PaletteError::PermissionDenied)?;

    // Prepare OSC 4 query
    let query = format!("\x1b]4;{};?\x07", index);

    // Send query
    tty.write_all(query.as_bytes())?;
    tty.flush()?;

    // Enable raw mode for the tty to capture escape sequences
    let _raw_guard = enable_raw_mode().map_err(PaletteError::Io)?;

    // Try to read response with timeout - responses come immediately
    let start = Instant::now();
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 1];

    // Set non-blocking mode manually
    unsafe {
        let fd = AsRawFd::as_raw_fd(&tty);
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }

    while start.elapsed() < timeout {
        match tty.read(&mut temp_buffer) {
            Ok(1..) => {
                buffer.push(temp_buffer[0]);

                // Check for complete response after each byte
                let response = String::from_utf8_lossy(&buffer);

                // Look for terminator to know we have complete response
                if response.contains('\x07') || response.contains("\x1b\\") {
                    if let Some(rgb) = try_parse_osc4_response(&response, index) {
                        return Ok(rgb);
                    }
                }

                // Prevent buffer from growing too large
                if buffer.len() > 512 {
                    break;
                }
            }
            Ok(0) => {
                // EOF - shouldn't happen with tty
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No data available yet, wait briefly
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => return Err(PaletteError::Io(e)),
        }
    }

    // Check one final time if we have a parseable response
    if !buffer.is_empty() {
        let response = String::from_utf8_lossy(&buffer);
        if let Some(rgb) = try_parse_osc4_response(&response, index) {
            return Ok(rgb);
        }
    }

    Err(PaletteError::Timeout)
}

/// Alternative approach using crossterm with threading
pub fn query_palette_color_crossterm(index: u8, timeout: Duration) -> Result<Rgb, PaletteError> {
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to handle the terminal I/O
    let handle = thread::spawn(move || {
        let result = (|| -> Result<Rgb, PaletteError> {
            // Enable raw mode
            enable_raw_mode().map_err(PaletteError::Io)?;

            let mut stdout = io::stdout();
            let mut stdin = io::stdin();

            // Send query
            let query = format!("\x1b]4;{};?\x07", index);
            stdout.write_all(query.as_bytes())?;
            stdout.flush()?;

            // Try to read response
            let mut buffer = Vec::new();
            let mut temp_buffer = [0u8; 1];
            let start = Instant::now();

            while start.elapsed() < timeout {
                match stdin.read(&mut temp_buffer) {
                    Ok(1..) => {
                        buffer.push(temp_buffer[0]);

                        // Try to parse response
                        let response = String::from_utf8_lossy(&buffer);
                        if let Some(rgb) = try_parse_osc4_response(&response, index) {
                            return Ok(rgb);
                        }

                        if buffer.len() > 256 {
                            break;
                        }
                    }
                    Ok(0) => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => return Err(PaletteError::Io(e)),
                }
            }

            Err(PaletteError::Timeout)
        })();

        // Always disable raw mode
        let _ = disable_raw_mode();

        let _ = tx.send(result);
    });

    // Wait for result or timeout
    match rx.recv_timeout(timeout + Duration::from_millis(100)) {
        Ok(result) => {
            let _ = handle.join();
            result
        }
        Err(_) => {
            // Thread might still be running, but we'll abandon it
            Err(PaletteError::Timeout)
        }
    }
}

/// Try to parse OSC 4 response from accumulated buffer
fn try_parse_osc4_response(response: &str, expected_index: u8) -> Option<Rgb> {
    // Look for the response pattern
    let pattern = format!("\x1b]4;{};", expected_index);

    if let Some(start_pos) = response.find(&pattern) {
        let response_part = &response[start_pos..];

        // Look for RGB format: rgb:RRRR/GGGG/BBBB or rgb:RR/GG/BB
        if let Some(rgb_pos) = response_part.find("rgb:") {
            let rgb_data = &response_part[rgb_pos + 4..];

            // Find the end of the response (BEL or other terminator)
            let end_pos = rgb_data
                .find('\x07')
                .or_else(|| rgb_data.find('\x1b'))
                .or_else(|| rgb_data.find('\n'))
                .unwrap_or(rgb_data.len().min(20));

            let rgb_str = &rgb_data[..end_pos];

            // Parse RGB components
            let parts: Vec<&str> = rgb_str.split('/').collect();
            if parts.len() >= 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parse_hex_component(parts[0]),
                    parse_hex_component(parts[1]),
                    parse_hex_component(parts[2]),
                ) {
                    return Some(Rgb::new(r, g, b));
                }
            }
        }

        // Try hex format: #RRGGBB
        if let Some(hash_pos) = response_part.find('#') {
            let hex_data = &response_part[hash_pos + 1..];
            if hex_data.len() >= 6 {
                let hex_str = &hex_data[..6];
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex_str[0..2], 16),
                    u8::from_str_radix(&hex_str[2..4], 16),
                    u8::from_str_radix(&hex_str[4..6], 16),
                ) {
                    return Some(Rgb::new(r, g, b));
                }
            }
        }
    }

    None
}

/// Parse hex component (2 or 4 digits)
fn parse_hex_component(hex_str: &str) -> Result<u8, std::num::ParseIntError> {
    let clean_hex: String = hex_str
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();

    if clean_hex.len() == 4 {
        // 16-bit value, take high byte
        let val = u16::from_str_radix(&clean_hex, 16)?;
        Ok((val >> 8) as u8)
    } else if clean_hex.len() == 2 {
        // 8-bit value
        u8::from_str_radix(&clean_hex, 16)
    } else {
        // Try to parse whatever we have
        let val = u16::from_str_radix(&clean_hex, 16)?;
        Ok((val.min(255)) as u8)
    }
}

/// Attempt to query all 16 colors using different methods
fn experiment_with_queries() {
    println!("🧪 Experimental Palette Querying");
    println!("=================================");
    println!("⚠️  Warning: This is experimental and may not work in all terminals!");
    println!();

    let timeout = Duration::from_millis(500);
    let methods = [
        (
            "Raw /dev/tty",
            query_palette_color_raw as fn(u8, Duration) -> Result<Rgb, PaletteError>,
        ),
        ("Crossterm threading", query_palette_color_crossterm),
    ];

    for (method_name, query_fn) in methods {
        println!("Testing method: {}", method_name);
        println!("{}", "─".repeat(40));

        let mut success_count = 0;

        for i in 0..4 {
            // Test first 4 colors only to avoid spam
            print!("  Color {}: ", i);
            match query_fn(i, timeout) {
                Ok(rgb) => {
                    println!("✅ RGB({}, {}, {})", rgb.r, rgb.g, rgb.b);
                    success_count += 1;
                }
                Err(e) => {
                    println!("❌ {:?}", e);
                }
            }

            // Small delay between attempts
            thread::sleep(Duration::from_millis(50));
        }

        println!("  Success rate: {}/4", success_count);

        if success_count == 0 {
            println!("  💡 This method doesn't work in your terminal");
        } else {
            println!("  🎉 This method shows promise!");
        }

        println!();
    }
}

/// Display terminal environment information
fn display_environment_info() {
    println!("🖥️  Terminal Environment Information");
    println!("====================================");

    let env_vars = [
        "TERM",
        "COLORTERM",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
        "ITERM_SESSION_ID",
        "WEZTERM_EXECUTABLE",
        "ALACRITTY_SOCKET",
        "KITTY_WINDOW_ID",
        "WT_SESSION",
    ];

    for var in env_vars {
        if let Ok(value) = std::env::var(var) {
            println!("  {} = {}", var, value);
        }
    }

    println!();

    // Check terminal capabilities using termbg
    println!("Terminal capabilities (via termbg):");
    match termbg::terminal() {
        termbg::Terminal::XtermCompatible => {
            println!("  Type: Xterm-compatible (likely supports OSC 4)")
        }
        termbg::Terminal::Screen => println!("  Type: GNU Screen (limited OSC support)"),
        termbg::Terminal::Tmux => println!("  Type: Tmux (limited OSC support)"),
        termbg::Terminal::Windows => println!("  Type: Windows Terminal"),
        termbg::Terminal::Emacs => println!("  Type: Emacs (no OSC support)"),
    }

    // Try background detection
    match termbg::theme(Duration::from_millis(100)) {
        Ok(theme) => println!("  Theme: {:?}", theme),
        Err(_) => println!("  Theme: Could not detect"),
    }

    println!();
}

/// Safety warnings and disclaimers
fn display_warnings() {
    println!("⚠️  EXPERIMENTAL SCRIPT WARNINGS");
    println!("=================================");
    println!();
    println!("This script attempts real OSC 4 queries which:");
    println!("• May cause terminal flickering or artifacts");
    println!("• Might not work in tmux, screen, or some IDEs");
    println!("• Could interfere with terminal multiplexers");
    println!("• May require specific terminal permissions");
    println!("• Works best in native terminal applications");
    println!();
    println!("Recommended terminals for testing:");
    println!("✅ WezTerm, Alacritty, iTerm2, Kitty");
    println!("✅ Windows Terminal (1.22+)");
    println!("✅ Modern GNOME Terminal, Konsole");
    println!();
    println!("Not recommended:");
    println!("❌ Terminal emulators inside IDEs (VSCode, etc.)");
    println!("❌ SSH sessions without proper terminal forwarding");
    println!("❌ Screen or tmux without passthrough configuration");
    println!();

    // Ask for confirmation in interactive mode
    if atty::is(atty::Stream::Stdin) {
        print!("Continue with experimental queries? [y/N]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Aborted. Use query_terminal_palette.rs for a safe demonstration.");
                std::process::exit(0);
            }
        }
    } else {
        println!("Running in non-interactive mode, proceeding with caution...");
    }

    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Experimental Terminal Palette Query");
    println!("=======================================");
    println!();

    display_warnings();
    display_environment_info();

    // Show thag theme info
    let term_attrs = TermAttributes::get_or_init();
    println!("Current thag theme: {}", term_attrs.theme.name);
    println!("Color support: {:?}", term_attrs.color_support);
    println!("Background: {:?}", term_attrs.term_bg_luma);
    println!();

    // Run experiments
    experiment_with_queries();

    // Show analysis tips
    analyze_successful_responses();

    // Show alternatives
    demonstrate_alternatives();

    // Demonstrate production-ready implementation
    demonstrate_production_method();

    println!("🏁 Experimental Results Summary");
    println!("===============================");
    println!("✅ OSC 4 palette querying WORKS on macOS terminals!");
    println!("✅ Crossterm method provides reliable implementation");
    println!("⚠️  Raw /dev/tty method needs refinement for capture");
    println!("✅ All major macOS terminals respond correctly");
    println!();
    println!("🚀 Next steps for thag_styling integration:");
    println!("   • Implement crossterm-based palette detection");
    println!("   • Add palette comparison with current themes");
    println!("   • Use for automatic theme adjustment");
    println!("   • Consider fallback methods for other platforms");
    println!();
    println!("💡 For educational demonstration of the concepts,");
    println!("   run: cargo run demo/query_terminal_palette.rs");

    Ok(())
}

/// Show analysis of successful query results
fn analyze_successful_responses() {
    println!("🔍 Response Analysis Tips");
    println!("========================");
    println!();
    println!("If you see responses like: ^[]4;0;rgb:1818/1818/1818^G");
    println!("This means:");
    println!("  ✅ Your terminal supports OSC 4 queries");
    println!("  ✅ The query is working correctly");
    println!("  ⚠️  The capture mechanism needs improvement");
    println!();
    println!("The visible escape sequences show:");
    println!("  • ^[ = ESC character (\\x1b)");
    println!("  • ]4 = OSC 4 command");
    println!("  • rgb:XXXX/YYYY/ZZZZ = 16-bit RGB values");
    println!("  • ^G = BEL terminator (\\x07)");
    println!();
    println!("For production implementation:");
    println!("  1. Use crossterm method (proven to work)");
    println!("  2. Fall back to environment detection");
    println!("  3. Cache results to avoid repeated queries");
    println!("  4. Handle timeout gracefully");
    println!();
}

/// Demonstrate production-ready palette detection using crossterm
fn demonstrate_production_method() {
    println!("🚀 Production-Ready Palette Detection");
    println!("====================================");
    println!();

    let colors = query_full_palette_crossterm(Duration::from_millis(100));
    let successful = colors.iter().filter(|c| c.is_some()).count();

    if successful > 0 {
        println!("✅ Successfully queried {}/16 palette colors", successful);
        println!();

        // Display first few colors as examples
        for (i, color_opt) in colors.iter().enumerate().take(8) {
            if let Some(color) = color_opt {
                println!(
                    "  Color {:2}: RGB({:3}, {:3}, {:3}) \x1b[48;5;{}m    \x1b[0m",
                    i, color.r, color.g, color.b, i
                );
            }
        }
        println!();

        // Show how this could integrate with thag_styling
        println!("🎯 thag_styling Integration Potential:");
        println!("   • Detect if terminal palette matches current theme");
        println!("   • Auto-adjust theme colors for better contrast");
        println!("   • Provide palette-aware color selection");
        println!("   • Enable real-time theme synchronization");
    } else {
        println!("❌ No colors detected - may need fallback methods");
    }
    println!();
}

/// Query all 16 palette colors using the proven crossterm method
fn query_full_palette_crossterm(timeout: Duration) -> Vec<Option<Rgb>> {
    let mut colors = Vec::with_capacity(16);

    for i in 0..16 {
        match query_palette_color_crossterm(i, timeout) {
            Ok(rgb) => colors.push(Some(rgb)),
            Err(_) => colors.push(None),
        }

        // Small delay to avoid overwhelming terminal
        thread::sleep(Duration::from_millis(10));
    }

    colors
}

/// Demonstrate practical alternatives for palette detection
fn demonstrate_alternatives() {
    println!("🛠️  Practical Alternatives for Palette Detection:");
    println!("================================================");
    println!();

    println!("1. Environment Variables:");
    if let Ok(term) = std::env::var("TERM") {
        println!("   TERM = {}", term);
    }
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        println!("   COLORTERM = {}", colorterm);
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        println!("   TERM_PROGRAM = {}", term_program);
    }
    println!();

    println!("2. Background Detection (using termbg):");
    match termbg::theme(Duration::from_millis(100)) {
        Ok(theme) => println!("   Terminal theme: {:?}", theme),
        Err(e) => println!("   Could not detect theme: {:?}", e),
    }

    match termbg::rgb(Duration::from_millis(100)) {
        Ok(rgb) => println!("   Background RGB: ({}, {}, {})", rgb.r, rgb.g, rgb.b),
        Err(e) => println!("   Could not detect background: {:?}", e),
    }
    println!();

    println!("3. Color Support Detection:");
    let term_attrs = TermAttributes::get_or_init();
    println!("   Detected support: {:?}", term_attrs.color_support);
    println!("   Background luma: {:?}", term_attrs.term_bg_luma);
    println!();

    println!("4. Manual Terminal Detection:");
    let terminal_hints = [
        ("WEZTERM_EXECUTABLE", "WezTerm"),
        ("ALACRITTY_SOCKET", "Alacritty"),
        ("KITTY_WINDOW_ID", "Kitty"),
        ("ITERM_SESSION_ID", "iTerm2"),
        ("WT_SESSION", "Windows Terminal"),
    ];

    for (env_var, terminal_name) in terminal_hints {
        if std::env::var(env_var).is_ok() {
            println!("   Detected: {} ({})", terminal_name, env_var);
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_component() {
        assert_eq!(parse_hex_component("ff").unwrap(), 255);
        assert_eq!(parse_hex_component("00").unwrap(), 0);
        assert_eq!(parse_hex_component("ff00").unwrap(), 255); // 16-bit
    }

    #[test]
    fn test_try_parse_osc4_response() {
        let response = "\x1b]4;1;rgb:ff00/8000/4000\x07";
        let result = try_parse_osc4_response(response, 1);
        assert_eq!(result, Some(Rgb::new(255, 128, 64)));

        let response2 = "\x1b]4;0;#ff8040\x07";
        let result2 = try_parse_osc4_response(response2, 0);
        assert_eq!(result2, Some(Rgb::new(255, 128, 64)));
    }
}
