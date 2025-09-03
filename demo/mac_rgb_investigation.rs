/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
crossterm = "0.28"
*/

//! Mac RGB Color Investigation
//!
//! This script investigates the specific issue on Mac where:
//! - Palette-indexed colors (ESC[38;5;Nm) display correctly
//! - RGB truecolor sequences (ESC[38;2;R;G;Bm) display incorrectly as washed-out colors
//!
//! The script tests various color output methods to understand what's happening
//! with RGB color interpretation on macOS terminals.

//# Purpose: Investigate Mac RGB color display issues with different escape sequence methods
//# Categories: terminal, colors, debugging, macos

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

/// Test color struct for our investigations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub name: &'static str,
}

impl TestColor {
    pub const fn new(r: u8, g: u8, b: u8, name: &'static str) -> Self {
        Self { r, g, b, name }
    }
}

/// Test colors - specifically chosen to be distinctive
const TEST_COLORS: &[TestColor] = &[
    TestColor::new(91, 116, 116, "Dark Duck-egg Blue-green"),
    TestColor::new(255, 0, 0, "Pure Red"),
    TestColor::new(0, 255, 0, "Pure Green"),
    TestColor::new(0, 0, 255, "Pure Blue"),
    TestColor::new(128, 64, 192, "Purple"),
    TestColor::new(255, 165, 0, "Orange"),
    TestColor::new(64, 224, 208, "Turquoise"),
    TestColor::new(139, 69, 19, "Saddle Brown"),
];

/// Display a color using different methods for comparison
fn display_color_comparison(color: &TestColor) {
    println!(
        "🎨 Testing: {} RGB({}, {}, {})",
        color.name, color.r, color.g, color.b
    );
    println!("   Expected: A block of {} color", color.name);
    println!();

    // Method 1: RGB Truecolor (24-bit) - the problematic one
    print!("   RGB Truecolor:  ");
    print!(
        "\x1b[38;2;{};{};{}m████████████\x1b[0m",
        color.r, color.g, color.b
    );
    println!(" ESC[38;2;{};{};{}m", color.r, color.g, color.b);

    // Method 2: RGB Background
    print!("   RGB Background: ");
    print!(
        "\x1b[48;2;{};{};{}m            \x1b[0m",
        color.r, color.g, color.b
    );
    println!(" ESC[48;2;{};{};{}m", color.r, color.g, color.b);

    // Method 3: Find closest 256-color equivalent and display it
    let closest_256 = find_closest_256_color(color.r, color.g, color.b);
    print!("   256-Color ({:3}): ", closest_256);
    print!("\x1b[38;5;{}m████████████\x1b[0m", closest_256);
    println!(" ESC[38;5;{}m", closest_256);

    // Method 4: 256-color background
    print!("   256-Color BG:   ");
    print!("\x1b[48;5;{}m            \x1b[0m", closest_256);
    println!(" ESC[48;5;{}m", closest_256);

    println!();
}

/// Find the closest 256-color palette index for an RGB color
fn find_closest_256_color(r: u8, g: u8, b: u8) -> u8 {
    let mut best_index = 0u8;
    let mut best_distance = u32::MAX;

    // Check the 216 color cube (16-231)
    for color_index in 16..232 {
        let cube_index = color_index - 16;
        let cube_r = (cube_index / 36) * 51;
        let cube_g = ((cube_index % 36) / 6) * 51;
        let cube_b = (cube_index % 6) * 51;

        let distance = color_distance(r, g, b, cube_r as u8, cube_g as u8, cube_b as u8);
        if distance < best_distance {
            best_distance = distance;
            best_index = color_index as u8;
        }
    }

    // Check grayscale ramp (232-255)
    for gray_index in 232..=255 {
        let gray_value = 8 + (gray_index - 232) * 10;
        let distance = color_distance(
            r,
            g,
            b,
            gray_value as u8,
            gray_value as u8,
            gray_value as u8,
        );
        if distance < best_distance {
            best_distance = distance;
            best_index = gray_index as u8;
        }
    }

    // Check basic 16 colors (0-15)
    let basic_colors = [
        (0, 0, 0),       // 0: Black
        (128, 0, 0),     // 1: Red
        (0, 128, 0),     // 2: Green
        (128, 128, 0),   // 3: Yellow
        (0, 0, 128),     // 4: Blue
        (128, 0, 128),   // 5: Magenta
        (0, 128, 128),   // 6: Cyan
        (192, 192, 192), // 7: White
        (128, 128, 128), // 8: Bright Black
        (255, 0, 0),     // 9: Bright Red
        (0, 255, 0),     // 10: Bright Green
        (255, 255, 0),   // 11: Bright Yellow
        (0, 0, 255),     // 12: Bright Blue
        (255, 0, 255),   // 13: Bright Magenta
        (0, 255, 255),   // 14: Bright Cyan
        (255, 255, 255), // 15: Bright White
    ];

    for (i, (basic_r, basic_g, basic_b)) in basic_colors.iter().enumerate() {
        let distance = color_distance(r, g, b, *basic_r, *basic_g, *basic_b);
        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    best_index
}

/// Calculate color distance (simple Manhattan distance)
fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
    let dr = (r1 as i32 - r2 as i32).abs() as u32;
    let dg = (g1 as i32 - g2 as i32).abs() as u32;
    let db = (b1 as i32 - b2 as i32).abs() as u32;
    dr + dg + db
}

/// Test OSC sequence color setting and querying
fn test_osc_color_handling() {
    println!("🔍 Testing OSC Color Sequence Handling");
    println!("=======================================");
    println!();

    let test_color = &TEST_COLORS[0]; // Dark duck-egg blue-green

    println!("Testing OSC 4 (palette color setting) vs OSC 10 (foreground color setting)");
    println!(
        "Target color: {} RGB({}, {}, {})",
        test_color.name, test_color.r, test_color.g, test_color.b
    );
    println!();

    // Test 1: Try to set palette color 15 (bright white) to our test color
    println!("1. Setting palette color 15 to our test color using OSC 4:");
    let osc4_command = format!(
        "\x1b]4;15;rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}\x07",
        test_color.r, test_color.r, test_color.g, test_color.g, test_color.b, test_color.b
    );
    print!("{}", osc4_command);
    io::stdout().flush().unwrap();

    // Display color 15
    print!("   Palette 15: ");
    print!("\x1b[38;5;15m████████████\x1b[0m");
    println!(" (should show our test color if OSC 4 worked)");

    // Test 2: Reset palette color 15 back to white
    println!("\n2. Resetting palette color 15 back to white:");
    let reset_command = "\x1b]4;15;rgb:ffff/ffff/ffff\x07";
    print!("{}", reset_command);
    io::stdout().flush().unwrap();

    print!("   Palette 15: ");
    print!("\x1b[38;5;15m████████████\x1b[0m");
    println!(" (should be white again)");

    println!();

    // Test 3: Compare different RGB sequence formats
    println!("3. Testing different RGB escape sequence formats:");

    // Standard format
    print!("   Standard RGB:   ");
    print!(
        "\x1b[38;2;{};{};{}m████████████\x1b[0m",
        test_color.r, test_color.g, test_color.b
    );
    println!(
        " ESC[38;2;{};{};{}m",
        test_color.r, test_color.g, test_color.b
    );

    // Alternative separator (some terminals might handle colons differently)
    print!("   With colons:    ");
    print!(
        "\x1b[38:2:{}:{}:{}m████████████\x1b[0m",
        test_color.r, test_color.g, test_color.b
    );
    println!(
        " ESC[38:2:{}:{}:{}m",
        test_color.r, test_color.g, test_color.b
    );

    println!();
}

/// Query terminal capabilities using OSC sequences
fn query_terminal_capabilities() {
    println!("🔎 Querying Terminal Capabilities");
    println!("==================================");
    println!();

    // Enable raw mode for reading responses
    if enable_raw_mode().is_err() {
        println!("❌ Could not enable raw mode for terminal querying");
        return;
    }

    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    // Query 1: OSC 10 (foreground color)
    println!("1. Querying current foreground color (OSC 10):");
    stdout.write_all(b"\x1b]10;?\x07").unwrap();
    stdout.flush().unwrap();

    let response = read_terminal_response(&mut stdin, Duration::from_millis(200));
    match response {
        Some(resp) => println!("   Response: {:?}", resp),
        None => println!("   No response (timeout)"),
    }

    // Query 2: OSC 11 (background color)
    println!("\n2. Querying current background color (OSC 11):");
    stdout.write_all(b"\x1b]11;?\x07").unwrap();
    stdout.flush().unwrap();

    let response = read_terminal_response(&mut stdin, Duration::from_millis(200));
    match response {
        Some(resp) => println!("   Response: {:?}", resp),
        None => println!("   No response (timeout)"),
    }

    // Query 3: OSC 4 (palette color query)
    println!("\n3. Querying palette color 15 (OSC 4):");
    stdout.write_all(b"\x1b]4;15;?\x07").unwrap();
    stdout.flush().unwrap();

    let response = read_terminal_response(&mut stdin, Duration::from_millis(200));
    match response {
        Some(resp) => println!("   Response: {:?}", resp),
        None => println!("   No response (timeout)"),
    }

    let _ = disable_raw_mode();
    println!();
}

/// Read a terminal response with timeout
fn read_terminal_response(stdin: &mut io::Stdin, timeout: Duration) -> Option<String> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 1];
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match stdin.read(&mut temp_buffer) {
            Ok(1) => {
                buffer.push(temp_buffer[0]);

                // Check for response terminators
                if temp_buffer[0] == 0x07 || // BEL
                   (buffer.len() >= 2 && buffer[buffer.len()-2] == 0x1b && buffer[buffer.len()-1] == b'\\')
                {
                    break;
                }

                // Safety limit
                if buffer.len() > 1024 {
                    break;
                }
            }
            Ok(0) => {
                thread::sleep(Duration::from_millis(1));
            }
            Ok(_) => {
                // Handle case where more than 1 byte is read
                thread::sleep(Duration::from_millis(1));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(_) => break,
        }
    }

    if buffer.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&buffer).to_string())
    }
}

/// Display environment information that might affect color handling
fn display_environment_info() {
    println!("🖥️  Environment Information");
    println!("============================");

    let env_vars = [
        "TERM",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
        "COLORTERM",
        "ITERM_SESSION_ID",
        "LC_TERMINAL",
        "LC_TERMINAL_VERSION",
        "TMUX",
        "SSH_TTY",
    ];

    for var in &env_vars {
        if let Ok(value) = std::env::var(var) {
            println!("   {}: {}", var, value);
        }
    }

    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍎 Mac RGB Color Investigation");
    println!("==============================");
    println!("Investigating why RGB truecolor sequences display incorrectly on Mac");
    println!("while palette-indexed colors display correctly.");
    println!();

    // Display system information
    display_environment_info();

    // Test color display methods
    println!("🎨 Color Display Method Comparison");
    println!("===================================");
    println!("Each color will be displayed using different methods.");
    println!("Compare the results to see if RGB and palette colors match.");
    println!();

    for color in TEST_COLORS.iter().take(3) {
        display_color_comparison(color);
        thread::sleep(Duration::from_millis(100)); // Small delay for visual separation
    }

    // Test OSC sequence handling
    test_osc_color_handling();

    // Query terminal capabilities
    query_terminal_capabilities();

    // Additional diagnostic information
    println!("📋 Diagnostic Summary");
    println!("=====================");
    println!("If RGB truecolor sequences show as washed-out or incorrect colors:");
    println!("• The terminal may not be properly interpreting ESC[38;2;R;G;Bm sequences");
    println!("• OSC 4 palette manipulation might work while direct RGB doesn't");
    println!("• Terminal color profile or gamma settings might be interfering");
    println!();
    println!("Expected behavior:");
    println!("• RGB truecolor and 256-color approximations should look similar");
    println!("• OSC sequence queries should return color information");
    println!("• Both foreground and background RGB should work consistently");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_distance() {
        assert_eq!(color_distance(0, 0, 0, 0, 0, 0), 0);
        assert_eq!(color_distance(255, 255, 255, 0, 0, 0), 765);
        assert_eq!(color_distance(100, 150, 200, 100, 150, 200), 0);
    }

    #[test]
    fn test_find_closest_256_color() {
        // Pure red should map to color 9 (bright red) or similar
        let red_index = find_closest_256_color(255, 0, 0);
        assert!(red_index == 9 || red_index == 196); // Bright red or cube red

        // Pure white should map to color 15
        let white_index = find_closest_256_color(255, 255, 255);
        assert_eq!(white_index, 15);

        // Pure black should map to color 0
        let black_index = find_closest_256_color(0, 0, 0);
        assert_eq!(black_index, 0);
    }
}
