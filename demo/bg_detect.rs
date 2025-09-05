/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto", features = ["color_detect"] }
crossterm = "0.28"
*/

//! Windows Background Color Detection
//!
//! This script investigates querying palette color 0 as a proxy for background color
//! detection in Windows environments. This approach might work where OSC 11 queries
//! fail, particularly in PowerShell and regular Windows terminals.
//!
//! Based on the observation that demo/truecolor*.rs files work in PowerShell,
//! suggesting we can interrogate palette colors from Rust instead of shell scripts.

//# Purpose: Test Windows background color detection via palette color 0 query
//# Categories: terminal, colors, windows, background, detection

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
        // Using standard luminance formula
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        0.299 * r + 0.587 * g + 0.114 * b
    }

    pub fn is_light(&self) -> bool {
        self.luminance() > 0.5
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Windows Background Color Detection Test");
    println!("=============================================");
    println!("Testing palette color 0 query as background detection method");
    println!();

    // Show environment
    display_environment();

    // Test different background detection methods
    println!("🔍 Background Detection Methods:");
    println!("================================");

    // Method 1: OSC 11 (standard background query)
    println!("1. OSC 11 Background Query:");
    let osc11_result = query_background_osc11(Duration::from_millis(300));
    match osc11_result {
        Some(rgb) => println!(
            "   ✅ Success: {} {} (luminance: {:.2})",
            rgb.to_hex(),
            if rgb.is_light() { "Light" } else { "Dark" },
            rgb.luminance()
        ),
        None => println!("   ❌ Failed: No response or timeout"),
    }

    // Method 2: OSC 4 Palette Color 0 Query
    println!("\n2. OSC 4 Palette Color 0 Query (Background Proxy):");
    let palette0_result = query_palette_color_0(Duration::from_millis(300));
    match palette0_result {
        Some(rgb) => println!(
            "   ✅ Success: {} {} (luminance: {:.2})",
            rgb.to_hex(),
            if rgb.is_light() { "Light" } else { "Dark" },
            rgb.luminance()
        ),
        None => println!("   ❌ Failed: No response or timeout"),
    }

    // Method 3: Compare existing terminal detection
    println!("\n3. Existing thag_common Detection:");
    // Method 3: Compare existing terminal detection (always available since we have the feature)
    {
        let (color_support, term_bg_rgb) = thag_common::terminal::detect_term_capabilities();
        println!("   Color Support: {:?}", color_support);
        if *term_bg_rgb != (0, 0, 0) {
            let bg_rgb = Rgb::new(term_bg_rgb.0, term_bg_rgb.1, term_bg_rgb.2);
            println!(
                "   Background: {} {} (luminance: {:.2})",
                bg_rgb.to_hex(),
                if bg_rgb.is_light() { "Light" } else { "Dark" },
                bg_rgb.luminance()
            );
        } else {
            println!("   ❌ No background detected (default 0,0,0)");
        }
    }

    // Analysis
    println!("\n📊 Analysis:");
    println!("=============");
    compare_methods(osc11_result, palette0_result);

    println!("\n💡 Windows-Specific Notes:");
    println!("===========================");
    display_windows_analysis();

    Ok(())
}

fn display_environment() {
    println!("📊 Environment:");
    let vars = [
        "OS",
        "TERM",
        "TERM_PROGRAM",
        "COLORTERM",
        "WT_SESSION",  // Windows Terminal
        "SESSIONNAME", // Windows session info
    ];

    for var in &vars {
        match std::env::var(var) {
            Ok(value) => println!("   {}: {}", var, value),
            Err(_) => println!("   {}: <not set>", var),
        }
    }

    // Detect Windows environment
    if cfg!(windows) {
        println!("   Platform: Windows (compiled for Windows)");
    } else {
        println!("   Platform: Unix-like (not Windows)");
    }
    println!();
}

/// Query background color using standard OSC 11
fn query_background_osc11(timeout: Duration) -> Option<Rgb> {
    query_osc_color("\x1b]11;?\x07", timeout, "11")
}

/// Query palette color 0 using OSC 4 (potential background proxy)
fn query_palette_color_0(timeout: Duration) -> Option<Rgb> {
    query_osc_color("\x1b]4;0;?\x07", timeout, "4;0")
}

/// Generic OSC color query function
fn query_osc_color(query_sequence: &str, timeout: Duration, query_name: &str) -> Option<Rgb> {
    if enable_raw_mode().is_err() {
        println!("   ❌ Cannot enable raw mode for {}", query_name);
        return None;
    }

    let result = (|| -> Option<Rgb> {
        let mut stdout = io::stdout();
        let mut stdin = io::stdin();

        // Send query
        stdout.write_all(query_sequence.as_bytes()).ok()?;
        stdout.flush().ok()?;

        // Read response
        let response = read_terminal_response(&mut stdin, timeout)?;

        // Parse response
        parse_color_response(&response, query_name)
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

                // Check for response terminators
                if temp_buffer[0] == 0x07 || // BEL
                   (buffer.len() >= 2 && buffer[buffer.len()-2] == 0x1b && buffer[buffer.len()-1] == b'\\')
                {
                    break;
                }

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

/// Parse OSC color response (handles both OSC 11 and OSC 4 responses)
fn parse_color_response(response: &str, query_type: &str) -> Option<Rgb> {
    // OSC 11 response: \x1b]11;rgb:RRRR/GGGG/BBBB\x07
    // OSC 4 response:  \x1b]4;0;rgb:RRRR/GGGG/BBBB\x07

    // Look for rgb: pattern
    if let Some(rgb_start) = response.find("rgb:") {
        let rgb_data = &response[rgb_start + 4..];

        // Find end of RGB data
        let end_pos = rgb_data
            .find('\x07')
            .or_else(|| rgb_data.find('\x1b'))
            .unwrap_or(rgb_data.len());

        if end_pos > 0 {
            let rgb_sequence = &rgb_data[..end_pos];
            let parts: Vec<&str> = rgb_sequence.split('/').collect();

            if parts.len() == 3 {
                // Parse hex components (usually 4 chars each, take high byte)
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parse_hex_component(parts[0]),
                    parse_hex_component(parts[1]),
                    parse_hex_component(parts[2]),
                ) {
                    return Some(Rgb::new(r, g, b));
                }
            }
        }
    }

    // Also try #RRGGBB format
    if let Some(hash_pos) = response.find('#') {
        let hex_data = &response[hash_pos + 1..];
        if hex_data.len() >= 6 {
            let hex_str = &hex_data[..6];
            if hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
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

    println!("   Debug: Raw response for {}: {:?}", query_type, response);
    None
}

/// Parse hex component from OSC response
fn parse_hex_component(hex_str: &str) -> Result<u8, std::num::ParseIntError> {
    let clean_hex: String = hex_str
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();

    match clean_hex.len() {
        4 => {
            // 4-char hex: take high byte (first 2 chars give us the 8-bit value)
            let val = u16::from_str_radix(&clean_hex, 16)?;
            Ok((val >> 8) as u8)
        }
        2 => u8::from_str_radix(&clean_hex, 16),
        _ => {
            let val = u16::from_str_radix(&clean_hex, 16).unwrap_or(0);
            Ok(val.min(255) as u8)
        }
    }
}

/// Compare different detection methods
fn compare_methods(osc11: Option<Rgb>, palette0: Option<Rgb>) {
    match (osc11, palette0) {
        (Some(bg), Some(p0)) => {
            println!("   OSC 11 (Background): {}", bg.to_hex());
            println!("   OSC 4;0 (Palette 0):  {}", p0.to_hex());

            if bg == p0 {
                println!("   ✅ Perfect match! Palette 0 equals background");
            } else {
                let distance = color_distance(bg, p0);
                println!("   Color distance: {}", distance);
                if distance < 30.0 {
                    println!("   ✅ Very close match! Palette 0 approximates background");
                } else {
                    println!("   ⚠️  Different colors - palette 0 may not be background");
                }
            }
        }
        (Some(bg), None) => {
            println!("   OSC 11 works, but OSC 4;0 failed");
            println!("   Background: {}", bg.to_hex());
        }
        (None, Some(p0)) => {
            println!("   ✅ OSC 4;0 works where OSC 11 failed!");
            println!("   Palette 0: {} (potential background proxy)", p0.to_hex());
        }
        (None, None) => {
            println!("   ❌ Both methods failed - terminal may not support color queries");
        }
    }
}

/// Calculate color distance (simple Euclidean)
fn color_distance(c1: Rgb, c2: Rgb) -> f32 {
    let dr = c1.r as f32 - c2.r as f32;
    let dg = c1.g as f32 - c2.g as f32;
    let db = c1.b as f32 - c2.b as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

fn display_windows_analysis() {
    println!("🔍 Key Insights:");

    if cfg!(windows) {
        println!("   • Running on Windows - testing Windows-specific behavior");
        println!("   • PowerShell terminals often support color queries");
        println!("   • Windows Terminal (WT_SESSION) has good color support");
        println!("   • Legacy Command Prompt may have limited support");
    } else {
        println!("   • Running on Unix-like system - testing for comparison");
        println!("   • Most Unix terminals support OSC 11 background queries");
    }

    println!("\n💡 If OSC 4;0 works where OSC 11 fails:");
    println!("   • Palette color 0 can serve as background proxy");
    println!("   • This method might work in more Windows terminals");
    println!("   • Could solve Windows background detection issues");

    println!("\n🔧 Implementation Strategy:");
    println!("   1. Try OSC 11 first (standard method)");
    println!("   2. Fall back to OSC 4;0 if OSC 11 fails");
    println!("   3. Use existing detection as final fallback");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_luminance() {
        let black = Rgb::new(0, 0, 0);
        let white = Rgb::new(255, 255, 255);

        assert!(black.luminance() < 0.5);
        assert!(!black.is_light());

        assert!(white.luminance() > 0.5);
        assert!(white.is_light());
    }

    #[test]
    fn test_parse_hex_component() {
        assert_eq!(parse_hex_component("ff00").unwrap(), 255);
        assert_eq!(parse_hex_component("8000").unwrap(), 128);
        assert_eq!(parse_hex_component("ff").unwrap(), 255);
    }

    #[test]
    fn test_color_distance() {
        let c1 = Rgb::new(255, 0, 0);
        let c2 = Rgb::new(255, 0, 0);
        let c3 = Rgb::new(0, 255, 0);

        assert_eq!(color_distance(c1, c2), 0.0);
        assert!(color_distance(c1, c3) > 300.0);
    }
}
