/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
crossterm = "0.28"
*/

//! Windows TrueColor Detection
//!
//! This script tests TrueColor support on Windows by sending a TrueColor escape
//! sequence and querying the result, as suggested by https://github.com/termstandard/colors.
//!
//! The approach:
//! 1. Query current foreground color (OSC 10)
//! 2. Set a specific TrueColor foreground (OSC 10 with RGB)
//! 3. Query the foreground color again
//! 4. Restore original foreground color
//! 5. Compare set vs queried values to determine TrueColor support

//# Purpose: Test Windows TrueColor support using OSC sequence probing
//# Categories: terminal, colors, windows

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};
use std::sync::mpsc;
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

    /// Calculate color distance for comparison
    pub fn distance_to(&self, other: &Rgb) -> u16 {
        ((self.r as i16 - other.r as i16).abs()
            + (self.g as i16 - other.g as i16).abs()
            + (self.b as i16 - other.b as i16).abs()) as u16
    }
}

/// Parse hex component from OSC response
fn parse_hex_component(hex_str: &str) -> Result<u8, std::num::ParseIntError> {
    let clean_hex: String = hex_str
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();

    match clean_hex.len() {
        4 => {
            let val = u16::from_str_radix(&clean_hex, 16)?;
            Ok((val >> 8) as u8)
        }
        2 => u8::from_str_radix(&clean_hex, 16),
        _ => {
            let val = u16::from_str_radix(&clean_hex, 16).unwrap_or(0);
            Ok((val.min(255)) as u8)
        }
    }
}

/// Detect if we're running in mintty
fn is_mintty() -> bool {
    std::env::var("TERM_PROGRAM").map_or(false, |term| term == "mintty")
}

/// Parse mintty OSC 7704 response for palette colors
fn parse_mintty_response(response: &str) -> Option<Vec<Rgb>> {
    // Response format: ESC]7704;rgb:RRRR/GGGG/BBBB;rgb:XXXX/YYYY/ZZZZ BEL
    // First RGB is foreground, second is background

    if let Some(start_pos) = response.find("\x1b]7704;") {
        let response_part = &response[start_pos + 8..]; // Skip "ESC]7704;"

        // Skip the index number and semicolon
        if let Some(semicolon_pos) = response_part.find(';') {
            let color_part = &response_part[semicolon_pos + 1..];

            let mut colors = Vec::new();

            // Split by semicolon to get individual rgb: sections
            for rgb_section in color_part.split(';') {
                if rgb_section.starts_with("rgb:") {
                    let rgb_data = &rgb_section[4..];

                    // Find end of this RGB section
                    let end_pos = rgb_data
                        .find('\x07')
                        .or_else(|| rgb_data.find('\x1b'))
                        .unwrap_or(rgb_data.len());

                    let rgb_str = &rgb_data[..end_pos];
                    let parts: Vec<&str> = rgb_str.split('/').collect();

                    if parts.len() == 3 {
                        // Take first 2 hex digits of each component (mintty uses 4-digit hex)
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            u8::from_str_radix(&parts[0][..2.min(parts[0].len())], 16),
                            u8::from_str_radix(&parts[1][..2.min(parts[1].len())], 16),
                            u8::from_str_radix(&parts[2][..2.min(parts[2].len())], 16),
                        ) {
                            colors.push(Rgb::new(r, g, b));
                        }
                    }
                }
            }

            if !colors.is_empty() {
                return Some(colors);
            }
        }
    }

    None
}

/// Parse OSC 10 (foreground color) response
fn parse_osc10_response(response: &str) -> Option<Rgb> {
    // Look for OSC 10 response: ESC]10;rgb:RRRR/GGGG/BBBB BEL
    if let Some(start_pos) = response.find("\x1b]10;") {
        let response_part = &response[start_pos..];

        if let Some(rgb_pos) = response_part.find("rgb:") {
            let rgb_data = &response_part[rgb_pos..];

            let end_pos = rgb_data
                .find('\x07')
                .or_else(|| rgb_data.find('\x1b'))
                .unwrap_or(rgb_data.len());

            if end_pos >= 18 {
                let rgb_sequence = &rgb_data[4..end_pos];
                let parts: Vec<&str> = rgb_sequence.split('/').collect();

                if parts.len() == 3
                    && parts[0].len() == 4
                    && parts[1].len() == 4
                    && parts[2].len() == 4
                    && parts
                        .iter()
                        .all(|part| part.chars().all(|c| c.is_ascii_hexdigit()))
                {
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
        if let Some(hash_pos) = response_part.find('#') {
            let hex_data = &response_part[hash_pos + 1..];
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
    }

    None
}

// /// Parse mintty OSC 7704 response for palette colors
// fn parse_mintty_response(response: &str) -> Option<Vec<Rgb>> {
//     // Response format: ESC]7704;rgb:RRRR/GGGG/BBBB;rgb:XXXX/YYYY/ZZZZ BEL
//     // First RGB is foreground, second is background

//     if let Some(start_pos) = response.find("\x1b]7704;") {
//         let response_part = &response[start_pos + 8..]; // Skip "ESC]7704;"

//         // Skip the index number and semicolon
//         if let Some(semicolon_pos) = response_part.find(';') {
//             let color_part = &response_part[semicolon_pos + 1..];

//             let mut colors = Vec::new();

//             // Split by semicolon to get individual rgb: sections
//             for rgb_section in color_part.split(';') {
//                 if rgb_section.starts_with("rgb:") {
//                     let rgb_data = &rgb_section[4..];

//                     // Find end of this RGB section
//                     let end_pos = rgb_data
//                         .find('\x07')
//                         .or_else(|| rgb_data.find('\x1b'))
//                         .unwrap_or(rgb_data.len());

//                     let rgb_str = &rgb_data[..end_pos];
//                     let parts: Vec<&str> = rgb_str.split('/').collect();

//                     if parts.len() == 3 {
//                         // Take first 2 hex digits of each component (mintty uses 4-digit hex)
//                         if let (Ok(r), Ok(g), Ok(b)) = (
//                             u8::from_str_radix(&parts[0][..2.min(parts[0].len())], 16),
//                             u8::from_str_radix(&parts[1][..2.min(parts[1].len())], 16),
//                             u8::from_str_radix(&parts[2][..2.min(parts[2].len())], 16),
//                         ) {
//                             colors.push(Rgb::new(r, g, b));
//                         }
//                     }
//                 }
//             }

//             if !colors.is_empty() {
//                 return Some(colors);
//             }
//         }
//     }

//     None
// }

/// Query current foreground color using OSC 10
fn query_foreground_color(timeout: Duration) -> Option<Rgb> {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let result = (|| -> Option<Rgb> {
            enable_raw_mode().ok()?;

            let mut stdout = io::stdout();
            let mut stdin = io::stdin();

            // Send query (use mintty OSC 7704 if in mintty, otherwise standard OSC 10)
            let query = if is_mintty() {
                "\x1b]7704;7;?\x07" // Query palette color 7 (white/foreground) in mintty
            } else {
                "\x1b]10;?\x07"
            };
            stdout.write_all(query.as_bytes()).ok()?;
            stdout.flush().ok()?;

            let mut buffer = Vec::new();
            let mut temp_buffer = [0u8; 1];
            let start = Instant::now();

            while start.elapsed() < timeout {
                match stdin.read(&mut temp_buffer) {
                    Ok(1..) => {
                        buffer.push(temp_buffer[0]);

                        if buffer.len() >= 20 {
                            let response = String::from_utf8_lossy(&buffer);
                            if response.contains('\x07') || response.contains("\x1b\\") {
                                if is_mintty() {
                                    if let Some(colors) = parse_mintty_response(&response) {
                                        return colors.first().copied();
                                    }
                                } else {
                                    if let Some(rgb) = parse_osc10_response(&response) {
                                        return Some(rgb);
                                    }
                                }
                            }
                        }

                        /// Query mintty palette color using OSC 7704
                        fn query_mintty_palette_color(index: u8, timeout: Duration) -> Option<Rgb> {
                            let (tx, rx) = mpsc::channel();

                            let handle = thread::spawn(move || {
                                let result = (|| -> Option<Rgb> {
                                    enable_raw_mode().ok()?;

                                    let mut stdout = io::stdout();
                                    let mut stdin = io::stdin();

                                    // Send mintty OSC 7704 query
                                    let query = format!("\x1b]7704;{};?\x07", index);
                                    stdout.write_all(query.as_bytes()).ok()?;
                                    stdout.flush().ok()?;

                                    let mut buffer = Vec::new();
                                    let mut temp_buffer = [0u8; 1];
                                    let start = Instant::now();

                                    while start.elapsed() < timeout {
                                        match stdin.read(&mut temp_buffer) {
                                            Ok(1..) => {
                                                buffer.push(temp_buffer[0]);

                                                if buffer.len() >= 30 {
                                                    let response = String::from_utf8_lossy(&buffer);
                                                    if response.contains('\x07')
                                                        || response.contains("\x1b\\")
                                                    {
                                                        if let Some(colors) =
                                                            parse_mintty_response(&response)
                                                        {
                                                            // Return the foreground color (first in response)
                                                            return colors.first().copied();
                                                        }
                                                    }
                                                }

                                                if buffer.len() > 512 {
                                                    break;
                                                }
                                            }
                                            Ok(0) => thread::sleep(Duration::from_millis(1)),
                                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                                thread::sleep(Duration::from_millis(1))
                                            }
                                            Err(_) => break,
                                        }
                                    }

                                    None
                                })();

                                let _ = disable_raw_mode();
                                let _ = tx.send(result);
                            });

                            match rx.recv_timeout(timeout + Duration::from_millis(100)) {
                                Ok(result) => {
                                    let _ = handle.join();
                                    result
                                }
                                Err(_) => None,
                            }
                        }

                        if buffer.len() > 512 {
                            break;
                        }
                    }
                    Ok(0) => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(_) => break,
                }
            }

            None
        })();

        let _ = disable_raw_mode();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout + Duration::from_millis(100)) {
        Ok(result) => {
            let _ = handle.join();
            result
        }
        Err(_) => None,
    }
}

/// Set foreground color using OSC 10
fn set_foreground_color(rgb: Rgb) -> bool {
    let mut stdout = io::stdout();

    // Send OSC 10 set: ESC]10;rgb:RRRR/GGGG/BBBB BEL
    let set_cmd = format!(
        "\x1b]10;rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}\x07",
        rgb.r, rgb.r, rgb.g, rgb.g, rgb.b, rgb.b
    );

    stdout.write_all(set_cmd.as_bytes()).is_ok() && stdout.flush().is_ok()
}

/// Test TrueColor support by setting and querying
fn test_truecolor_support() -> (bool, String) {
    println!("🔍 Testing TrueColor support on Windows...");
    println!();

    let timeout = Duration::from_millis(200);

    // Step 1: Query original foreground color
    println!("1. Querying original foreground color...");
    let original_color = query_foreground_color(timeout);

    match original_color {
        Some(color) => println!("   Original: RGB({}, {}, {})", color.r, color.g, color.b),
        None => {
            println!("   Could not query original color");
            return (
                false,
                "Cannot query colors - terminal may not support OSC sequences".to_string(),
            );
        }
    }

    // Step 2: Set a distinctive TrueColor (not likely to be a standard 16-color)
    println!();
    println!("2. Setting test TrueColor RGB(123, 234, 45)...");
    let test_color = Rgb::new(123, 234, 45);

    if !set_foreground_color(test_color) {
        return (false, "Failed to send color set command".to_string());
    }

    // Small delay to let the color change take effect
    thread::sleep(Duration::from_millis(50));

    // Step 3: Query the color back
    println!("3. Querying color after setting...");
    let queried_color = query_foreground_color(timeout);

    let result = match queried_color {
        Some(color) => {
            println!(
                "   Queried back: RGB({}, {}, {})",
                color.r, color.g, color.b
            );

            let distance = test_color.distance_to(&color);
            println!("   Color distance: {}", distance);

            // Step 4: Restore original color immediately to prevent flicker
            if let Some(orig) = original_color {
                set_foreground_color(orig);
                println!("4. Restored original color");
            }

            // Analyze results
            if distance <= 10 {
                (true, "TrueColor supported - exact match".to_string())
            } else if distance <= 50 {
                (
                    true,
                    "TrueColor supported - close match (possible rounding)".to_string(),
                )
            } else {
                (
                    false,
                    format!(
                        "TrueColor not supported - significant difference (distance: {})",
                        distance
                    ),
                )
            }
        }
        None => {
            // Try to restore original color anyway
            if let Some(orig) = original_color {
                set_foreground_color(orig);
                println!("4. Restored original color");
            }
            (false, "Could not query color after setting".to_string())
        }
    };

    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Windows TrueColor Detection Test");
    println!("=====================================");
    println!("Testing TrueColor support by setting and querying foreground colors.");
    println!();

    // Show environment info
    if let Ok(term) = std::env::var("TERM") {
        println!("TERM: {}", term);
    }
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        println!("COLORTERM: {}", colorterm);
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        println!("TERM_PROGRAM: {}", term_program);
    }

    // Show detection method
    if is_mintty() {
        println!("Using mintty OSC 7704 for color queries");
    } else {
        println!("Using standard OSC 10 for color queries");
    }
    println!();

    let (supported, explanation) = test_truecolor_support();

    println!();
    println!("🎯 Final Result:");
    if supported {
        println!("✅ TrueColor IS supported");
    } else {
        println!("❌ TrueColor is NOT supported");
    }
    println!("📋 Details: {}", explanation);

    if supported {
        println!();
        println!("💡 This means:");
        println!("   • 24-bit RGB colors work correctly");
        println!("   • thag_styling can use full TrueColor palette");
        println!("   • Color comparisons should work properly");
    } else {
        println!();
        println!("💡 Fallback options:");
        println!("   • Use 256-color mode instead");
        println!("   • Limit palette to basic 16 colors");
        println!("   • Disable advanced color features");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_distance() {
        let color1 = Rgb::new(123, 234, 45);
        let color2 = Rgb::new(123, 234, 45);
        assert_eq!(color1.distance_to(&color2), 0);

        let color3 = Rgb::new(120, 230, 50);
        let distance = color1.distance_to(&color3);
        assert_eq!(distance, 3 + 4 + 5); // |123-120| + |234-230| + |45-50|
    }

    #[test]
    fn test_parse_osc10_response() {
        let response = "\x1b]10;rgb:7b7b/eaea/2d2d\x07";
        let result = parse_osc10_response(response);
        assert_eq!(result, Some(Rgb::new(123, 234, 45)));
    }

    #[test]
    fn test_parse_hex_component() {
        assert_eq!(parse_hex_component("7b7b").unwrap(), 123);
        assert_eq!(parse_hex_component("eaea").unwrap(), 234);
        assert_eq!(parse_hex_component("2d2d").unwrap(), 45);
    }
}
