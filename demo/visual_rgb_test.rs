/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Visual RGB Rendering Test
///
/// This script tests whether terminals actually render RGB truecolor sequences correctly
/// by displaying them side-by-side with their closest 256-color palette equivalents.
/// This helps detect terminals that accept RGB sequences but render them incorrectly
/// (like Apple Terminal showing salmon pink instead of duck-egg blue-green).
//# Purpose: Visual test to detect accurate RGB truecolor rendering vs palette quantization
//# Categories: color, diagnosis, terminal, testing
use std::io::{self, Write};

/// Test colors specifically chosen to reveal rendering issues
#[derive(Debug, Clone, Copy)]
struct TestColor {
    r: u8,
    g: u8,
    b: u8,
    name: &'static str,
    description: &'static str,
}

impl TestColor {
    const fn new(r: u8, g: u8, b: u8, name: &'static str, description: &'static str) -> Self {
        Self {
            r,
            g,
            b,
            name,
            description,
        }
    }
}

/// Colors designed to reveal different types of rendering issues
const DIAGNOSTIC_COLORS: &[TestColor] = &[
    TestColor::new(
        91,
        116,
        116,
        "Duck-egg Blue-green",
        "Known to render as salmon pink in Apple Terminal",
    ),
    TestColor::new(123, 234, 45, "Lime Green", "Distinctive non-standard green"),
    TestColor::new(200, 100, 50, "Burnt Orange", "Mid-range RGB values"),
    TestColor::new(75, 150, 225, "Sky Blue", "Should be clearly blue"),
    TestColor::new(180, 80, 190, "Purple-Magenta", "Distinctive purple hue"),
    TestColor::new(64, 64, 64, "Dark Gray", "Low RGB values test"),
    TestColor::new(
        220,
        220,
        100,
        "Pale Yellow",
        "High RGB values with color cast",
    ),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Visual RGB Rendering Test");
    println!("=============================");
    println!("Testing whether terminals render RGB truecolor sequences accurately");
    println!("or quantize them to nearest palette colors.");
    println!();

    display_environment_info();

    println!("🔍 Visual Comparison Test:");
    println!("===========================");
    println!("Compare RGB (left) vs 256-color approximation (right).");
    println!("They should look similar if RGB rendering is accurate.");
    println!("Significant differences indicate RGB rendering issues.");
    println!();

    for color in DIAGNOSTIC_COLORS {
        test_color_rendering(color);
        println!();
    }

    display_analysis_guide();

    Ok(())
}

fn display_environment_info() {
    println!("📊 Environment:");
    let vars = [
        ("TERM", "Terminal type"),
        ("TERM_PROGRAM", "Terminal program"),
        ("COLORTERM", "Color capability claim"),
        ("THAG_COLOR_MODE", "Thag override mode"),
    ];

    for (var, desc) in &vars {
        match std::env::var(var) {
            Ok(value) => println!("   {}: {} ({})", var, value, desc),
            Err(_) => println!("   {}: <not set> ({})", var, desc),
        }
    }
    println!();
}

fn test_color_rendering(color: &TestColor) {
    let closest_256 = find_closest_256_color(color.r, color.g, color.b);

    println!(
        "🎨 {} RGB({}, {}, {})",
        color.name, color.r, color.g, color.b
    );
    println!("   Description: {}", color.description);
    println!();

    // RGB rendering test
    print!("   RGB Sequence:  ");
    print!(
        "\x1b[38;2;{};{};{}m████████████████\x1b[0m",
        color.r, color.g, color.b
    );
    println!(" ESC[38;2;{};{};{}m", color.r, color.g, color.b);
    io::stdout().flush().unwrap();

    // 256-color reference
    print!("   256-Reference: ");
    print!("\x1b[38;5;{}m████████████████\x1b[0m", closest_256);
    println!(" ESC[38;5;{}m (index {})", closest_256, closest_256);

    // Background comparison too
    print!("   RGB Background:  ");
    print!(
        "\x1b[48;2;{};{};{}m                \x1b[0m",
        color.r, color.g, color.b
    );
    println!(" Background RGB");

    print!("   256-BG Ref:      ");
    print!("\x1b[48;5;{}m                \x1b[0m", closest_256);
    println!(" Background 256-color");

    println!("   💡 If RGB and 256-color versions look different, RGB isn't working correctly");
}

fn display_analysis_guide() {
    println!("📋 Analysis Guide:");
    println!("==================");
    println!();

    println!("✅ **Good RGB Support Indicators:**");
    println!("   • RGB and 256-color versions look very similar");
    println!("   • Colors appear as expected (duck-egg blue-green looks blue-green)");
    println!("   • Smooth gradations in complex colors");
    println!();

    println!("❌ **RGB Issues Indicators:**");
    println!("   • RGB colors look completely different from 256-color versions");
    println!("   • Duck-egg blue-green appears as salmon pink (Apple Terminal issue)");
    println!("   • RGB colors look washed out, oversaturated, or wrong hue");
    println!("   • Background and foreground RGB show different issues");
    println!();

    println!("⚠️  **Quantization Indicators:**");
    println!("   • RGB colors look like nearest 256-color matches");
    println!("   • Subtle RGB variations get 'flattened' to same color");
    println!("   • Still reasonable colors, just less precise");
    println!();

    match std::env::var("TERM_PROGRAM").ok().as_deref() {
        Some("Apple_Terminal") => {
            println!("🍎 **Apple Terminal Detected:**");
            println!("   Expected: RGB sequences will show incorrect colors (salmon pink issue)");
            println!("   Recommendation: Use THAG_COLOR_MODE=256 for correct colors");
        }
        Some("zed") => {
            println!("⚡ **Zed Terminal Detected:**");
            println!("   Expected: Should have proper RGB support");
            println!("   Note: Earlier RGB issues were reported but may be resolved");
        }
        Some("iTerm.app") => {
            println!("🖥️  **iTerm2 Detected:**");
            println!("   Expected: Excellent RGB support, colors should match closely");
        }
        Some("WezTerm") => {
            println!("🚀 **WezTerm Detected:**");
            println!("   Expected: Excellent RGB support, colors should match closely");
        }
        _ => {
            println!("❓ **Unknown Terminal:**");
            println!("   Use the visual comparison to assess RGB rendering quality");
        }
    }
    println!();

    println!("🔧 **Troubleshooting:**");
    println!("   If RGB rendering looks wrong:");
    println!("   • Try: export THAG_COLOR_MODE=256");
    println!("   • Or:  export THAG_COLOR_MODE=basic");
    println!("   • Test in different terminal to isolate the issue");
}

/// Find the closest 256-color palette index for an RGB color
fn find_closest_256_color(r: u8, g: u8, b: u8) -> u8 {
    let mut best_index = 0u8;
    let mut best_distance = u32::MAX;

    // Basic 16 colors (0-15)
    let basic_colors = [
        (0, 0, 0),       // 0: Black
        (128, 0, 0),     // 1: Red
        (0, 128, 0),     // 2: Green
        (128, 128, 0),   // 3: Yellow
        (0, 0, 128),     // 4: Blue
        (128, 0, 128),   // 5: Magenta
        (0, 128, 128),   // 6: Cyan
        (192, 192, 192), // 7: White
        (128, 128, 128), // 8: Bright Black (Gray)
        (255, 0, 0),     // 9: Bright Red
        (0, 255, 0),     // 10: Bright Green
        (255, 255, 0),   // 11: Bright Yellow
        (0, 0, 255),     // 12: Bright Blue
        (255, 0, 255),   // 13: Bright Magenta
        (0, 255, 255),   // 14: Bright Cyan
        (255, 255, 255), // 15: Bright White
    ];

    for (i, (br, bg, bb)) in basic_colors.iter().enumerate() {
        let distance = color_distance(r, g, b, *br, *bg, *bb);
        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    // 216 color cube (16-231): 6×6×6 RGB cube
    for i in 16..232 {
        let cube_index = i - 16;
        let cube_r = (cube_index / 36) * 51;
        let cube_g = ((cube_index % 36) / 6) * 51;
        let cube_b = (cube_index % 6) * 51;

        let distance = color_distance(r, g, b, cube_r as u8, cube_g as u8, cube_b as u8);
        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    // Grayscale ramp (232-255): 24 grays
    for i in 232..=255 {
        let gray_value = 8 + (i - 232) * 10;
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
            best_index = i as u8;
        }
    }

    best_index
}

/// Calculate Manhattan distance between two RGB colors
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
        assert_eq!(find_closest_256_color(0, 0, 0), 0); // Black
        assert_eq!(find_closest_256_color(255, 255, 255), 15); // White
    }

    #[test]
    fn test_color_distance() {
        assert_eq!(color_distance(0, 0, 0, 0, 0, 0), 0);
        assert_eq!(color_distance(255, 255, 255, 0, 0, 0), 765);
        assert_eq!(color_distance(100, 150, 200, 100, 150, 200), 0);
    }

    #[test]
    fn test_diagnostic_colors() {
        // Ensure our test colors are valid
        for color in DIAGNOSTIC_COLORS {
            assert!(color.r <= 255);
            assert!(color.g <= 255);
            assert!(color.b <= 255);
            assert!(!color.name.is_empty());
            assert!(!color.description.is_empty());
        }
    }
}
