/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

/// Test dynamic ANSI generation for different color support levels
///
/// This script verifies that our new dynamic ANSI generation approach
/// correctly adapts ANSI escape sequences based on terminal color support.
//# Purpose: Test and demonstrate dynamic ANSI generation for different terminal capabilities
//# Categories: color, styling, terminal, testing
use thag_styling::{
    ColorInfo, ColorInitStrategy, ColorSupport, Style, StylingResult, TermAttributes,
};

fn test_color_support_adaptation() -> StylingResult<()> {
    println!("🎨 Testing Dynamic ANSI Generation");
    println!("{}", "=".repeat(50));
    println!();

    // Create a test color (red)
    let red_rgb = [255, 0, 0];
    let color_info = ColorInfo::rgb(red_rgb[0], red_rgb[1], red_rgb[2]);

    println!(
        "🔴 Test Color: RGB({}, {}, {})",
        red_rgb[0], red_rgb[1], red_rgb[2]
    );
    println!("📊 ANSI sequences for different support levels:");
    println!();

    // Test each color support level
    let support_levels = vec![
        ColorSupport::TrueColor,
        ColorSupport::Color256,
        ColorSupport::Basic,
        ColorSupport::None,
    ];

    for support in support_levels {
        let ansi = color_info.to_ansi_for_support(support);
        println!("  {:15} → {:?}", format!("{:?}:", support), ansi);

        // Show what it looks like when painted
        if support != ColorSupport::None {
            print!("                    → ");
            print!("{}████{}", ansi, "\x1b[0m");
            println!(" ← colored blocks");
        }
        println!();
    }

    Ok(())
}

fn test_current_terminal_support() -> StylingResult<()> {
    println!("🖥️  Current Terminal Color Support Test");
    println!("{}", "=".repeat(50));
    println!();

    // Initialize terminal attributes
    TermAttributes::initialize(&ColorInitStrategy::Match);
    let term_attrs = TermAttributes::get_or_init();

    println!("Detected color support: {:?}", term_attrs.color_support);
    println!();

    // Test different color types with current terminal support
    let test_colors = vec![
        ("Red", [255, 0, 0]),
        ("Green", [0, 255, 0]),
        ("Blue", [0, 0, 255]),
        ("Yellow", [255, 255, 0]),
        ("Magenta", [255, 0, 255]),
        ("Cyan", [0, 255, 255]),
        ("Orange", [255, 165, 0]),
        ("Purple", [128, 0, 128]),
    ];

    for (name, rgb) in test_colors {
        let style = Style::with_rgb(rgb);
        let painted = style.paint(format!(
            "████ {} RGB({}, {}, {})",
            name, rgb[0], rgb[1], rgb[2]
        ));
        println!("  {}", painted);

        // Show the underlying ANSI sequence
        if let Some(color_info) = &style.foreground {
            let ansi = color_info.to_ansi_for_support(term_attrs.color_support);
            println!("    ANSI: {:?}", ansi);
        }
        println!();
    }

    Ok(())
}

fn test_compatibility_fallbacks() -> StylingResult<()> {
    println!("🔄 Color Support Fallback Test");
    println!("{}", "=".repeat(50));
    println!();

    // Test a TrueColor RGB value across different support levels
    let test_rgb = [173, 216, 230]; // Light blue
    println!(
        "Test RGB: ({}, {}, {})",
        test_rgb[0], test_rgb[1], test_rgb[2]
    );
    println!();

    let color_info = ColorInfo::rgb(test_rgb[0], test_rgb[1], test_rgb[2]);

    // Show how the same color renders under different support levels
    let levels = vec![
        ("TrueColor (24-bit)", ColorSupport::TrueColor),
        ("256-Color (8-bit)", ColorSupport::Color256),
        ("Basic (4-bit)", ColorSupport::Basic),
        ("No Color", ColorSupport::None),
    ];

    for (desc, support) in levels {
        let ansi = color_info.to_ansi_for_support(support);
        print!("  {:20} ", desc);

        if support != ColorSupport::None {
            print!("{}████████{} ", ansi, "\x1b[0m");
            print!("ANSI: {:?}", ansi);
        } else {
            print!("████████ (no color)");
        }
        println!();
    }
    println!();

    Ok(())
}

fn main() -> StylingResult<()> {
    test_color_support_adaptation()?;
    println!();

    test_current_terminal_support()?;
    println!();

    test_compatibility_fallbacks()?;

    println!("✅ Dynamic ANSI generation test complete!");
    println!();
    println!("💡 Key Benefits:");
    println!("  • Colors automatically adapt to terminal capabilities");
    println!("  • No need to pre-convert themes for different terminals");
    println!("  • Single source of truth for color values");
    println!("  • Proper fallbacks for limited color support");

    Ok(())
}
