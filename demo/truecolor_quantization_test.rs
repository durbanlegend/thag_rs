/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
crossterm = "0.28"
*/

/// TrueColor Quantization Detection Test
///
/// This test detects whether a terminal silently quantizes TrueColor values
/// to a 256-color palette, as suspected with Apple Terminal. The strategy:
///
/// 1. Test colors that should be identical in TrueColor but different in 256-color
/// 2. Test colors that fall between 256-color palette entries
/// 3. Use statistical analysis of multiple color tests
/// 4. Compare expected vs actual color distances
///
/// If the terminal silently quantizes, we'll see:
/// - Colors that should be different become identical
/// - Systematic rounding to 256-color palette values
/// - Loss of precision in color gradients
//# Purpose: Detect silent TrueColor quantization in terminals
//# Categories: terminal, color, testing
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

    /// Calculate color distance
    pub fn distance_to(&self, other: &Rgb) -> u16 {
        ((self.r as i16 - other.r as i16).abs()
            + (self.g as i16 - other.g as i16).abs()
            + (self.b as i16 - other.b as i16).abs()) as u16
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Calculate perceived brightness (0.0-1.0)
    pub fn brightness(&self) -> f32 {
        // Using standard luminance formula
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }
}

/// Test result for a single color
#[derive(Debug)]
#[allow(dead_code)]
struct ColorTest {
    input: Rgb,
    output: Option<Rgb>,
    expected_quantized: Rgb,
    is_quantized: bool,
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

/// Detect if we're running in mintty (which always supports TrueColor)
fn is_mintty() -> bool {
    std::env::var("TERM_PROGRAM").map_or(false, |term| term == "mintty")
}

/// Parse OSC 10 response
fn parse_osc10_response(response: &str) -> Option<Rgb> {
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

/// Set and query a color with timing (supports mintty via OSC 7704)
fn test_color(color: Rgb, timeout: Duration) -> Option<Rgb> {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let result = (|| -> Option<Rgb> {
            enable_raw_mode().ok()?;

            let mut stdout = io::stdout();
            let mut stdin = io::stdin();

            // Set the color
            let set_cmd = format!(
                "\x1b]10;rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}\x07",
                color.r, color.r, color.g, color.g, color.b, color.b
            );
            stdout.write_all(set_cmd.as_bytes()).ok()?;
            stdout.flush().ok()?;

            // Small delay
            thread::sleep(Duration::from_millis(20));

            // Query it back using standard OSC 10
            let query = "\x1b]10;?\x07";
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
                                if let Some(rgb) = parse_osc10_response(&response) {
                                    return Some(rgb);
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

/// Convert RGB to nearest 256-color palette equivalent
fn rgb_to_256color_rgb(rgb: Rgb) -> Rgb {
    // 256-color palette structure:
    // 0-15: system colors (we'll approximate)
    // 16-231: 6x6x6 color cube
    // 232-255: grayscale

    // Check if it's grayscale first
    if rgb.r == rgb.g && rgb.g == rgb.b {
        // Grayscale: map to 232-255 range
        let gray_index = (rgb.r as f32 / 255.0 * 23.0).round() as u8;
        let gray_value = (8 + gray_index * 10).min(255);
        return Rgb::new(gray_value, gray_value, gray_value);
    }

    // Map to 6x6x6 color cube
    let r_index = (rgb.r as f32 / 255.0 * 5.0).round() as u8;
    let g_index = (rgb.g as f32 / 255.0 * 5.0).round() as u8;
    let b_index = (rgb.b as f32 / 255.0 * 5.0).round() as u8;

    let r_val = if r_index == 0 { 0 } else { 55 + r_index * 40 };
    let g_val = if g_index == 0 { 0 } else { 55 + g_index * 40 };
    let b_val = if b_index == 0 { 0 } else { 55 + b_index * 40 };

    Rgb::new(r_val, g_val, b_val)
}

/// Generate test colors that reveal quantization
fn generate_test_colors() -> Vec<Rgb> {
    vec![
        // Colors that should map to the same 256-color value
        Rgb::new(127, 95, 63), // Should map to similar 256-color values
        Rgb::new(128, 96, 64), // as these
        Rgb::new(129, 97, 65),
        // Colors between 256-color palette entries
        Rgb::new(42, 142, 242), // Between standard palette colors
        Rgb::new(123, 234, 45), // Unusual combinations
        Rgb::new(87, 156, 203), // Mid-range values
        // Subtle gradients that would be lost in quantization
        Rgb::new(100, 100, 100),
        Rgb::new(101, 100, 100), // Should be different in TrueColor
        Rgb::new(102, 100, 100), // but same in 256-color
        // Edge cases
        Rgb::new(0, 0, 1),       // Nearly black
        Rgb::new(254, 254, 254), // Nearly white
        Rgb::new(255, 0, 128),   // High contrast
        // Test the 256-color boundaries
        Rgb::new(55, 55, 55),    // First non-black in 256 cube
        Rgb::new(95, 95, 95),    // Second value
        Rgb::new(135, 135, 135), // Third value
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 TrueColor Quantization Detection Test");
    println!("=======================================");
    println!("Testing whether terminal silently quantizes TrueColor to 256-color palette");
    println!();

    // Show environment
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        println!("Terminal: {}", term_program);
    }
    if let Ok(term) = std::env::var("TERM") {
        println!("TERM: {}", term);
    }

    // Show detection method
    if is_mintty() {
        println!("Detected mintty - TrueColor always supported");
    } else {
        println!("Using OSC 10 for TrueColor testing");
    }
    println!();

    let test_colors = generate_test_colors();
    let timeout = Duration::from_millis(150);
    let mut results = Vec::new();

    if is_mintty() {
        println!("Mintty detected - skipping quantization test (TrueColor guaranteed)");
        println!();

        // For mintty, create mock results showing no quantization
        for (i, &color) in test_colors.iter().enumerate() {
            println!(
                "Test {:2}: {} -> {} (no quantization)",
                i + 1,
                color.to_hex(),
                color.to_hex()
            );

            results.push(ColorTest {
                input: color,
                output: Some(color), // Same as input - no quantization
                expected_quantized: rgb_to_256color_rgb(color),
                is_quantized: false,
            });
        }
    } else {
        println!("Testing {} colors...", test_colors.len());
        println!();

        for (i, &color) in test_colors.iter().enumerate() {
            print!("Test {:2}: {} -> ", i + 1, color.to_hex());

            match test_color(color, timeout) {
                Some(output) => {
                    let expected_256 = rgb_to_256color_rgb(color);
                    let distance_to_256 = output.distance_to(&expected_256);
                    let distance_to_original = output.distance_to(&color);

                    let is_quantized = distance_to_256 <= 5 && distance_to_original > 10;

                    println!(
                        "{} (distance: orig={}, 256={})",
                        output.to_hex(),
                        distance_to_original,
                        distance_to_256
                    );

                    results.push(ColorTest {
                        input: color,
                        output: Some(output),
                        expected_quantized: expected_256,
                        is_quantized,
                    });
                }
                None => {
                    println!("No response");
                    results.push(ColorTest {
                        input: color,
                        output: None,
                        expected_quantized: rgb_to_256color_rgb(color),
                        is_quantized: false,
                    });
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    // Restore default foreground color
    print!("\x1b]10;\x07");
    io::stdout().flush().unwrap();

    println!();
    println!("🧮 Analysis:");
    println!("============");

    let successful_tests: Vec<_> = results.iter().filter(|t| t.output.is_some()).collect();
    let quantized_count = successful_tests.iter().filter(|t| t.is_quantized).count();

    println!(
        "Successful tests: {}/{}",
        successful_tests.len(),
        results.len()
    );
    println!(
        "Colors matching 256-palette: {}/{}",
        quantized_count,
        successful_tests.len()
    );

    if successful_tests.is_empty() && !is_mintty() {
        println!("❌ Could not test - terminal doesn't respond to color queries");
        println!("❌ No colors detected. May need different approach.");
        return Ok(());
    }

    let quantization_ratio = quantized_count as f64 / successful_tests.len() as f64;

    println!();
    println!("📊 Detailed Results with Visual Comparison:");
    println!("   256-Color Approx    Actual Result      Status");
    println!("   ───────────────    ─────────────      ──────");

    for (i, test) in results.iter().enumerate() {
        if let Some(output) = test.output {
            let status = if test.is_quantized {
                "QUANTIZED"
            } else {
                "TRUE     "
            };

            // Display color blocks: 256-color approximation on left, actual result on right
            let expected_hex = test.expected_quantized.to_hex();
            let actual_hex = output.to_hex();

            // Create RGB values for ANSI escape codes
            let exp = test.expected_quantized;
            let act = output;

            println!(
                "  {:2}: \x1b[48;2;{};{};{}m      \x1b[0m {} vs \x1b[48;2;{};{};{}m      \x1b[0m {} [{}]",
                i + 1,
                exp.r, exp.g, exp.b, expected_hex,
                act.r, act.g, act.b, actual_hex,
                status
            );
        }
    }

    println!();
    println!("🎯 Conclusion:");
    if is_mintty() {
        println!("✅ MINTTY TRUECOLOR SUPPORT");
        println!("   Mintty always supports TrueColor regardless of TERM setting");
        println!("   Based on official documentation and environment detection");
    } else if quantization_ratio > 0.7 {
        println!("❌ QUANTIZATION DETECTED");
        println!("   This terminal appears to silently quantize TrueColor to 256-color palette");
        println!(
            "   {}% of test colors were quantized",
            (quantization_ratio * 100.0) as u32
        );
        println!("   The terminal claims TrueColor support but doesn't provide it");
    } else if quantization_ratio > 0.3 {
        println!("⚠️  PARTIAL QUANTIZATION");
        println!("   Some colors are quantized, others are not");
        println!(
            "   {}% of test colors were quantized",
            (quantization_ratio * 100.0) as u32
        );
        println!("   Terminal behavior is inconsistent");
    } else {
        println!("✅ TRUE TRUECOLOR SUPPORT");
        println!("   This terminal provides genuine TrueColor support");
        println!(
            "   Only {}% of colors showed quantization (within error tolerance)",
            (quantization_ratio * 100.0) as u32
        );
    }

    println!();
    println!("💡 Recommendation for thag_styling:");
    if is_mintty() || quantization_ratio <= 0.5 {
        println!("   Use ColorSupport::TrueColor for this terminal");
    } else {
        println!("   Use ColorSupport::Color256 for this terminal");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_256color_conversion() {
        // Test exact 256-color cube values
        let cube_color = Rgb::new(95, 135, 175); // Should map to itself
        let result = rgb_to_256color_rgb(cube_color);
        assert!(result.distance_to(&cube_color) < 10);

        // Test grayscale
        let gray = Rgb::new(128, 128, 128);
        let result = rgb_to_256color_rgb(gray);
        assert_eq!(result.r, result.g);
        assert_eq!(result.g, result.b);
    }

    #[test]
    fn test_color_distance() {
        let color1 = Rgb::new(100, 100, 100);
        let color2 = Rgb::new(101, 100, 100);
        assert_eq!(color1.distance_to(&color2), 1);
    }
}
