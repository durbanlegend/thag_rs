/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

//! RGB vs Palette Color Comparison
//!
//! This script directly demonstrates the issue where RGB truecolor sequences
//! display differently than expected compared to palette-indexed colors.
//! It tests the specific color mentioned: RGB(91, 116, 116) which should be
//! a dark duck-egg blue-green but appears as washed-out salmon pink.

//# Purpose: Demonstrate RGB vs palette color display differences on Mac
//# Categories: terminal, colors, debugging, macos

use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 RGB vs Palette Color Comparison");
    println!("===================================");
    println!();

    // The specific problematic color mentioned
    let test_rgb = (91, 116, 116);
    println!(
        "Testing RGB({}, {}, {})",
        test_rgb.0, test_rgb.1, test_rgb.2
    );
    println!("Expected: Dark duck-egg blue-green");
    println!("Actual on Mac: Washed-out salmon pink (if bug present)");
    println!();

    // Test 1: Direct RGB truecolor sequence
    println!("1. RGB Truecolor Sequence:");
    print!("   ");
    print!(
        "\x1b[38;2;{};{};{}m████████████████████\x1b[0m",
        test_rgb.0, test_rgb.1, test_rgb.2
    );
    println!(
        " #{:02x}{:02x}{:02x} ({},{},{})",
        test_rgb.0, test_rgb.1, test_rgb.2, test_rgb.0, test_rgb.1, test_rgb.2
    );
    io::stdout().flush()?;

    // Test 2: RGB background
    println!("\n2. RGB Background:");
    print!("   ");
    print!(
        "\x1b[48;2;{};{};{}m                    \x1b[0m",
        test_rgb.0, test_rgb.1, test_rgb.2
    );
    println!(
        " #{:02x}{:02x}{:02x} ({},{},{})",
        test_rgb.0, test_rgb.1, test_rgb.2, test_rgb.0, test_rgb.1, test_rgb.2
    );

    // Test 3: Find and display closest 256-color equivalent
    let closest_256 = find_closest_256_color(test_rgb.0, test_rgb.1, test_rgb.2);
    println!("\n3. Closest 256-Color Equivalent (Index {}):", closest_256);
    print!("   ");
    print!("\x1b[38;5;{}m████████████████████\x1b[0m", closest_256);
    println!(" Color index {}", closest_256);

    // Test 4: 256-color background
    println!("\n4. 256-Color Background:");
    print!("   ");
    print!("\x1b[48;5;{}m                    \x1b[0m", closest_256);
    println!(" Color index {}", closest_256);

    // Test 5: Manual printf-style test (the working one mentioned)
    println!("\n5. Manual escape sequence (what works):");
    print!("   ");
    print!("\x1b[38;5;15m████████████████████\x1b[0m");
    println!(" Palette index 15 (bright white)");

    // Test 6: The exact sequences mentioned in the problem
    println!("\n6. Exact sequences from problem description:");
    println!("   Working palette sequence:");
    print!("   ");
    print!("\x1b[38;5;15m████████████████████\x1b[0m");
    println!(" printf \"\\x1b[38;5;15m████\\x1b[0m\"");

    println!("\n   Problematic RGB sequence:");
    print!("   ");
    print!("\x1b[38;2;91;116;116m████████████████████\x1b[0m");
    println!(" printf \"\\x1b[38;2;91;116;116m████\\x1b[0m\"");

    // Test multiple RGB colors to see if it's a general problem
    println!("\n7. Multiple RGB Colors Test:");
    let test_colors = [
        (255, 0, 0, "Pure Red"),
        (0, 255, 0, "Pure Green"),
        (0, 0, 255, "Pure Blue"),
        (255, 255, 0, "Yellow"),
        (255, 0, 255, "Magenta"),
        (0, 255, 255, "Cyan"),
        (128, 128, 128, "Gray"),
        (91, 116, 116, "Duck-egg Blue-green"),
    ];

    for (r, g, b, name) in &test_colors {
        print!("   RGB({:3},{:3},{:3}): ", r, g, b);
        print!("\x1b[38;2;{};{};{}m██████\x1b[0m", r, g, b);
        print!(" vs 256: ");
        let idx = find_closest_256_color(*r, *g, *b);
        print!("\x1b[38;5;{}m██████\x1b[0m", idx);
        println!(" {} (idx:{})", name, idx);
    }

    println!();
    println!("📋 Analysis:");
    println!("=============");
    println!("• If RGB and 256-color versions look different, there's a color handling issue");
    println!("• If RGB colors appear washed out or wrong, the terminal may not be");
    println!("  properly interpreting ESC[38;2;R;G;B sequences");
    println!("• The working palette sequence (ESC[38;5;N) uses indexed colors");
    println!("• The problematic RGB sequence (ESC[38;2;R;G;B) uses direct RGB values");
    println!();
    println!("Expected result: RGB and 256-color approximations should look similar");
    println!("Problem result: RGB shows as washed-out/incorrect colors");

    Ok(())
}

/// Find the closest 256-color palette index for an RGB color
fn find_closest_256_color(r: u8, g: u8, b: u8) -> u8 {
    let mut best_index = 0u8;
    let mut best_distance = u32::MAX;

    // Check basic 16 colors first (0-15)
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

    best_index
}

/// Calculate color distance (Manhattan distance)
fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
    let dr = (r1 as i32 - r2 as i32).abs() as u32;
    let dg = (g1 as i32 - g2 as i32).abs() as u32;
    let db = (b1 as i32 - b2 as i32).abs() as u32;
    dr + dg + db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_closest_256_color() {
        // Test pure colors
        assert_eq!(find_closest_256_color(255, 0, 0), 9); // Bright red
        assert_eq!(find_closest_256_color(0, 255, 0), 10); // Bright green
        assert_eq!(find_closest_256_color(0, 0, 255), 12); // Bright blue
        assert_eq!(find_closest_256_color(255, 255, 255), 15); // White
        assert_eq!(find_closest_256_color(0, 0, 0), 0); // Black
    }

    #[test]
    fn test_color_distance() {
        assert_eq!(color_distance(0, 0, 0, 0, 0, 0), 0);
        assert_eq!(color_distance(255, 255, 255, 0, 0, 0), 765);
        assert_eq!(color_distance(100, 150, 200, 100, 150, 200), 0);
    }
}
