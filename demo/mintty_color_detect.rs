/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto", features = ["color_detect"] }
crossterm = "0.28"
*/

/// Mintty Color Detection Test
///
/// This script tests mintty's special OSC 7704 sequence for querying palette colors.
/// Mintty uses a non-standard but more reliable method for color queries compared
/// to standard OSC sequences. This can help detect background colors and verify
/// palette colors in mintty terminals.
///
/// Based on the shell script in TODO.md lines 49-74, this implements the same
/// logic in Rust to query mintty ANSI slots 0-15.
//# Purpose: Test mintty-specific color detection using OSC 7704 sequences
//# Categories: color, detection, mintty, terminal, windows
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

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

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Calculate luminance to determine if light or dark
    pub fn luminance(&self) -> f32 {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        0.299 * r + 0.587 * g + 0.114 * b
    }

    pub fn is_light(&self) -> bool {
        self.luminance() > 0.5
    }
}

/// Color pair for foreground and background
#[derive(Debug, Clone)]
pub struct ColorPair {
    pub foreground: Option<Rgb>,
    pub background: Option<Rgb>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Mintty Color Detection Test");
    println!("===============================");
    println!("Testing mintty's special OSC 7704 sequence for palette color queries");
    println!();

    display_environment();

    // Test if we're in mintty
    if !is_mintty() {
        println!("⚠️  Warning: Not detected as mintty terminal");
        println!("   TERM_PROGRAM should be 'mintty' for this test");
        println!("   Continuing anyway to test the sequence...");
        println!();
    }

    // Test individual color queries
    println!("🎨 Testing Individual Palette Colors:");
    println!("=====================================");
    test_specific_colors(&[0, 2, 7, 8, 15]); // Key colors: black, green, white, bright black, bright white

    println!("\n🌈 Full Palette Query (0-15):");
    println!("==============================");
    test_full_palette();

    println!("\n🔍 Background Detection Analysis:");
    println!("==================================");
    analyze_background_detection();

    println!("\n💡 Mintty Detection Integration:");
    println!("=================================");
    show_integration_suggestions();

    Ok(())
}

fn display_environment() {
    println!("📊 Environment:");
    let vars = [
        "TERM",
        "TERM_PROGRAM",
        "COLORTERM",
        "MSYSTEM",      // MSYS2/MinGW environment
        "MINGW_PREFIX", // MinGW environment
    ];

    for var in &vars {
        match std::env::var(var) {
            Ok(value) => println!("   {}: {}", var, value),
            Err(_) => println!("   {}: <not set>", var),
        }
    }

    println!(
        "   Platform: {}",
        if cfg!(windows) {
            "Windows"
        } else {
            "Unix-like"
        }
    );
    println!("   Is Mintty: {}", is_mintty());
    println!();
}

/// Check if running in mintty
fn is_mintty() -> bool {
    std::env::var("TERM_PROGRAM").map_or(false, |term| term == "mintty")
}

/// Test specific palette color indices
fn test_specific_colors(indices: &[u8]) {
    for &index in indices {
        print!("Color {index:2}: ");
        match query_mintty_color(index, Duration::from_millis(300)) {
            Some(colors) => match (colors.foreground, colors.background) {
                (Some(fg), Some(bg)) => {
                    println!(
                        "FG: {} {} BG: {} {}",
                        fg.to_hex(),
                        if fg.is_light() { "Light" } else { "Dark" },
                        bg.to_hex(),
                        if bg.is_light() { "Light" } else { "Dark" }
                    );
                }
                (Some(fg), None) => {
                    println!(
                        "FG: {} {} (no background)",
                        fg.to_hex(),
                        if fg.is_light() { "Light" } else { "Dark" }
                    );
                }
                (None, Some(bg)) => {
                    println!(
                        "BG: {} {} (no foreground)",
                        bg.to_hex(),
                        if bg.is_light() { "Light" } else { "Dark" }
                    );
                }
                (None, None) => println!("❌ No colors detected"),
            },
            None => println!("❌ Query failed or timeout"),
        }
    }
}

/// Test the full palette (0-15)
fn test_full_palette() {
    let mut successful_queries = 0;
    let mut background_candidates = Vec::new();

    for index in 0..16 {
        if let Some(colors) = query_mintty_color(index, Duration::from_millis(200)) {
            successful_queries += 1;

            print!("{index:2}: ");
            match (colors.foreground, colors.background) {
                (Some(fg), Some(bg)) => {
                    print!("FG:{} BG:{}", fg.to_hex(), bg.to_hex());
                    // Color 0 is often the background
                    if index == 0 {
                        background_candidates.push(("Palette 0 FG", fg));
                        background_candidates.push(("Palette 0 BG", bg));
                    }
                }
                (Some(fg), None) => {
                    print!("FG:{}", fg.to_hex());
                    if index == 0 {
                        background_candidates.push(("Palette 0 FG", fg));
                    }
                }
                (None, Some(bg)) => {
                    print!("BG:{}", bg.to_hex());
                    if index == 0 {
                        background_candidates.push(("Palette 0 BG", bg));
                    }
                }
                (None, None) => print!("No colors"),
            }
            println!();
        } else {
            println!("{index:2}: ❌ Query failed");
        }
    }

    println!("\n📊 Summary:");
    println!("   Successful queries: {}/16", successful_queries);

    if !background_candidates.is_empty() {
        println!("   Background color candidates from palette 0:");
        for (source, rgb) in background_candidates {
            println!(
                "   • {}: {} {} (luminance: {:.2})",
                source,
                rgb.to_hex(),
                if rgb.is_light() { "Light" } else { "Dark" },
                rgb.luminance()
            );
        }
    }
}

/// Query mintty palette color using OSC 7704
fn query_mintty_color(index: u8, timeout: Duration) -> Option<ColorPair> {
    if enable_raw_mode().is_err() {
        return None;
    }

    let result = (|| -> Option<ColorPair> {
        let mut stdout = io::stdout();
        let mut stdin = io::stdin();

        // Send mintty-specific OSC 7704 query
        // Format: ESC]7704;{index};?BEL
        let query = format!("\x1b]7704;{};?\x07", index);
        stdout.write_all(query.as_bytes()).ok()?;
        stdout.flush().ok()?;

        // Read response with timeout
        let response = read_terminal_response(&mut stdin, timeout)?;

        // Parse mintty response
        parse_mintty_response(&response)
    })();

    let _ = disable_raw_mode();
    result
}

/// Read terminal response with timeout
fn read_terminal_response(stdin: &mut io::Stdin, timeout: Duration) -> Option<String> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 1];
    let start = Instant::now();

    while start.elapsed() < timeout {
        match stdin.read(&mut temp_buffer) {
            Ok(1) => {
                buffer.push(temp_buffer[0]);

                // Check for BEL terminator (mintty uses this)
                if temp_buffer[0] == 0x07 {
                    break;
                }

                // Safety limit
                if buffer.len() > 1024 {
                    break;
                }
            }
            Ok(0) => thread::sleep(Duration::from_millis(1)),
            Ok(_) => thread::sleep(Duration::from_millis(1)),
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

/// Parse mintty OSC 7704 response
/// Expected format: ESC]7704;rgb:RRRR/GGGG/BBBB;rgb:RRRR/GGGG/BBBBBEL
/// or: ESC]7704;{index};rgb:RRRR/GGGG/BBBB;rgb:RRRR/GGGG/BBBBBEL
fn parse_mintty_response(response: &str) -> Option<ColorPair> {
    // Remove the ESC]7704; prefix and BEL suffix
    let clean_response = response
        .strip_prefix("\x1b]7704;")
        .unwrap_or(response)
        .strip_suffix('\x07')
        .unwrap_or(response);

    // Split by semicolons to find rgb: parts
    let parts: Vec<&str> = clean_response.split(';').collect();

    let mut colors = Vec::new();

    for part in parts {
        if let Some(rgb) = parse_rgb_component(part) {
            colors.push(rgb);
        }
    }

    // mintty returns foreground then background (usually)
    match colors.len() {
        0 => None,
        1 => Some(ColorPair {
            foreground: Some(colors[0]),
            background: None,
        }),
        _ => Some(ColorPair {
            foreground: Some(colors[0]),
            background: Some(colors[1]),
        }),
    }
}

/// Parse a single rgb: component from mintty response
fn parse_rgb_component(part: &str) -> Option<Rgb> {
    if let Some(rgb_data) = part.strip_prefix("rgb:") {
        let components: Vec<&str> = rgb_data.split('/').collect();

        if components.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parse_hex_component(components[0]),
                parse_hex_component(components[1]),
                parse_hex_component(components[2]),
            ) {
                return Some(Rgb::new(r, g, b));
            }
        }
    }
    None
}

/// Parse hex component (mintty uses 4-digit hex, we want the high byte)
fn parse_hex_component(hex_str: &str) -> Result<u8, std::num::ParseIntError> {
    let clean_hex: String = hex_str
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();

    match clean_hex.len() {
        4 => {
            // Take first 2 characters for 8-bit value
            u8::from_str_radix(&clean_hex[0..2], 16)
        }
        2 => u8::from_str_radix(&clean_hex, 16),
        _ => {
            let val = u16::from_str_radix(&clean_hex, 16).unwrap_or(0);
            Ok(val.min(255) as u8)
        }
    }
}

/// Analyze background detection possibilities
fn analyze_background_detection() {
    // Try to detect background using palette color 0
    println!("Testing palette color 0 as background proxy...");

    if let Some(colors) = query_mintty_color(0, Duration::from_millis(300)) {
        match (colors.foreground, colors.background) {
            (Some(fg), Some(bg)) => {
                println!("✅ Palette 0 returned both FG and BG colors:");
                println!(
                    "   Foreground: {} {} (luminance: {:.2})",
                    fg.to_hex(),
                    if fg.is_light() { "Light" } else { "Dark" },
                    fg.luminance()
                );
                println!(
                    "   Background: {} {} (luminance: {:.2})",
                    bg.to_hex(),
                    if bg.is_light() { "Light" } else { "Dark" },
                    bg.luminance()
                );
                println!("   💡 Background color can be used for theme detection");
            }
            (Some(fg), None) => {
                println!("✅ Palette 0 returned foreground color: {}", fg.to_hex());
                println!("   💡 This might be the background color (common in some terminals)");
            }
            _ => {
                println!("❌ Palette 0 query didn't return useful colors");
            }
        }
    } else {
        println!("❌ Failed to query palette color 0");
    }

    // Compare with existing detection
    println!("\nComparing with thag_common detection:");
    let (color_support, term_bg_rgb) = thag_common::terminal::detect_term_capabilities();
    println!("   thag_common color support: {:?}", color_support);
    let [r, g, b] = *term_bg_rgb;
    if [r, g, b] != [0, 0, 0] {
        let bg_rgb = Rgb::new(r, g, b);
        println!(
            "   thag_common background: {} {} (luminance: {:.2})",
            bg_rgb.to_hex(),
            if bg_rgb.is_light() { "Light" } else { "Dark" },
            bg_rgb.luminance()
        );
    } else {
        println!("   thag_common: No background detected");
    }
}

/// Show suggestions for integrating mintty detection
fn show_integration_suggestions() {
    println!("🔧 Integration Strategy:");
    println!("========================");

    if is_mintty() {
        println!("✅ Running in mintty - OSC 7704 should work");
        println!("   • Use OSC 7704 for reliable color queries");
        println!("   • Query palette 0 for background detection");
        println!("   • Fallback to standard OSC if 7704 fails");
    } else {
        println!("❌ Not running in mintty");
        println!("   • OSC 7704 sequences won't work");
        println!("   • Use standard OSC sequences instead");
    }

    println!("\n🚀 Implementation in thag_common:");
    println!("   1. Detect mintty via TERM_PROGRAM=mintty");
    println!("   2. Use OSC 7704;0;? for background detection");
    println!("   3. Parse both FG and BG from response");
    println!("   4. Use BG color for theme detection");
    println!("   5. Assume TrueColor support (mintty always supports it)");

    println!("\n📝 Code Pattern:");
    println!("   if is_mintty() {{");
    println!("       // Use OSC 7704 for color queries");
    println!("       // Always assume TrueColor support");
    println!("   }} else {{");
    println!("       // Use standard OSC 10/11 queries");
    println!("   }}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_component() {
        assert_eq!(parse_hex_component("ff00").unwrap(), 255);
        assert_eq!(parse_hex_component("8080").unwrap(), 128);
        assert_eq!(parse_hex_component("ff").unwrap(), 255);
    }

    #[test]
    fn test_parse_rgb_component() {
        let rgb = parse_rgb_component("rgb:ff00/8080/0000").unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 128);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_rgb_luminance() {
        let black = Rgb::new(0, 0, 0);
        let white = Rgb::new(255, 255, 255);

        assert!(!black.is_light());
        assert!(white.is_light());
        assert!(white.luminance() > black.luminance());
    }
}
