/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_styling = { version = "0.2, thag-auto" }
*/

/// Enhanced styled! Macro Demonstration
///
/// This demo showcases the enhanced styled! macro with support for:
/// - Basic ANSI colors (original functionality)
/// - 256-color palette indices
/// - True RGB colors
/// - Multiple text effects
///
/// The enhanced macro now supports three color formats:
/// 1. Basic colors: Red, Green, Blue, etc. (uses terminal palette)
/// 2. Color256(index): 256-color palette (0-255)
/// 3. Rgb(r, g, b): True RGB colors (0-255 per component)
//# Purpose: Demonstrate enhanced styled! macro with 256-color and RGB support
//# Categories: styling, macros, color, demo
use thag_proc_macros::{ansi_styling_support, styled};

// Enable the ANSI styling support
ansi_styling_support! {}

fn main() {
    println!("🎨 Enhanced styled! Macro Demonstration\n");

    // Basic ANSI colors (original functionality)
    println!("=== Basic ANSI Colors (Terminal Palette) ===");
    println!("{}", styled!("Red text", fg = Red));
    println!("{}", styled!("Green bold", fg = Green, bold));
    println!(
        "{}",
        styled!("Blue italic underline", fg = Blue, italic, underline)
    );
    println!("{}", styled!("Yellow reversed", fg = Yellow, reversed));
    println!(
        "{}",
        styled!(
            "Magenta with multiple effects",
            fg = Magenta,
            bold,
            italic,
            underline
        )
    );
    println!();

    // 256-color palette
    println!("=== 256-Color Palette ===");
    println!("{}", styled!("Bright red (196)", fg = Color256(196)));
    println!("{}", styled!("Orange (214)", fg = Color256(214), bold));
    println!("{}", styled!("Deep blue (21)", fg = Color256(21), italic));
    println!("{}", styled!("Purple (93)", fg = Color256(93), underline));
    println!(
        "{}",
        styled!("Forest green (34)", fg = Color256(34), bold, italic)
    );

    // Color spectrum demonstration
    println!("\n256-Color Spectrum Sample:");
    print!("{}", styled!("█", fg = Color256(196)));
    print!("{}", styled!("█", fg = Color256(202)));
    print!("{}", styled!("█", fg = Color256(214)));
    print!("{}", styled!("█", fg = Color256(226)));
    print!("{}", styled!("█", fg = Color256(190)));
    print!("{}", styled!("█", fg = Color256(154)));
    print!("{}", styled!("█", fg = Color256(118)));
    print!("{}", styled!("█", fg = Color256(82)));
    print!("{}", styled!("█", fg = Color256(46)));
    print!("{}", styled!("█", fg = Color256(21)));
    print!("{}", styled!("█", fg = Color256(57)));
    print!("{}", styled!("█", fg = Color256(93)));
    print!("{}", styled!("█", fg = Color256(129)));
    print!("{}", styled!("█", fg = Color256(165)));
    print!("{}", styled!("█", fg = Color256(201)));
    println!(" (256-color palette)\n");

    // True RGB colors
    println!("=== True RGB Colors ===");
    println!(
        "{}",
        styled!("Crimson (220, 20, 60)", fg = Rgb(220, 20, 60))
    );
    println!(
        "{}",
        styled!("Orange (255, 165, 0)", fg = Rgb(255, 165, 0), bold)
    );
    println!(
        "{}",
        styled!("Lime green (50, 205, 50)", fg = Rgb(50, 205, 50), italic)
    );
    println!(
        "{}",
        styled!(
            "Deep sky blue (0, 191, 255)",
            fg = Rgb(0, 191, 255),
            underline
        )
    );
    println!(
        "{}",
        styled!(
            "Hot pink (255, 105, 180)",
            fg = Rgb(255, 105, 180),
            bold,
            reversed
        )
    );

    // RGB gradient demonstration
    println!("\nRGB Gradient Sample:");
    print!("{}", styled!("█", fg = Rgb(255, 0, 0))); // Red
    print!("{}", styled!("█", fg = Rgb(255, 127, 0))); // Orange
    print!("{}", styled!("█", fg = Rgb(255, 255, 0))); // Yellow
    print!("{}", styled!("█", fg = Rgb(127, 255, 0))); // Yellow-green
    print!("{}", styled!("█", fg = Rgb(0, 255, 0))); // Green
    print!("{}", styled!("█", fg = Rgb(0, 255, 127))); // Green-cyan
    print!("{}", styled!("█", fg = Rgb(0, 255, 255))); // Cyan
    print!("{}", styled!("█", fg = Rgb(0, 127, 255))); // Cyan-blue
    print!("{}", styled!("█", fg = Rgb(0, 0, 255))); // Blue
    print!("{}", styled!("█", fg = Rgb(127, 0, 255))); // Blue-magenta
    print!("{}", styled!("█", fg = Rgb(255, 0, 255))); // Magenta
    print!("{}", styled!("█", fg = Rgb(255, 0, 127))); // Magenta-red
    println!(" (RGB gradient)\n");

    // Hex color demonstration
    println!("=== Hex Colors ===");
    println!("{}", styled!("Hex red (#ff0000)", fg = "#ff0000"));
    println!("{}", styled!("Hex orange (#ffa500)", fg = "#ffa500", bold));
    println!(
        "{}",
        styled!("Hex purple (#800080)", fg = "#800080", italic)
    );
    println!(
        "{}",
        styled!("Hex teal (#008080)", fg = "#008080", underline)
    );
    println!(
        "{}",
        styled!("Hex gold (#ffd700)", fg = "#ffd700", bold, italic)
    );

    // Mixed examples showing practical usage
    println!("=== Practical Usage Examples ===");

    // Error messages with specific colors
    println!("{}", styled!("ERROR:", fg = Rgb(220, 50, 47), bold));
    println!(
        "{}",
        styled!("  File not found: config.toml", fg = Color256(167))
    );

    // Success messages
    println!("{}", styled!("SUCCESS:", fg = Rgb(133, 153, 0), bold));
    println!(
        "{}",
        styled!("  Build completed successfully", fg = Color256(107))
    );

    // Warning messages
    println!("{}", styled!("WARNING:", fg = Rgb(181, 137, 0), bold));
    println!(
        "{}",
        styled!("  Deprecated function used", fg = Color256(178))
    );

    // Code highlighting
    println!("\nCode syntax highlighting:");
    println!(
        "{}{}{}{}{}",
        styled!("fn ", fg = Rgb(133, 153, 0), bold), // keyword
        styled!("main", fg = Rgb(38, 139, 210)),     // function name
        styled!("() {", fg = Color256(247)),         // punctuation
        styled!(" println!", fg = Rgb(220, 50, 47)), // macro
        styled!(" }", fg = Color256(247))            // punctuation
    );

    println!("\n=== Color Format Comparison ===");
    println!("Basic ANSI:  {}", styled!("Red", fg = Red));
    println!("256-color:   {}", styled!("Red", fg = Color256(196)));
    println!("RGB:         {}", styled!("Red", fg = Rgb(255, 0, 0)));
    println!("Hex:         {}", styled!("Red", fg = "#ff0000"));

    println!("\n=== Text Effects Showcase ===");
    println!("{}", styled!("Bold text", bold));
    println!("{}", styled!("Italic text", italic));
    println!("{}", styled!("Underlined text", underline));
    println!("{}", styled!("Reversed text", reversed));
    println!(
        "{}",
        styled!(
            "All effects combined",
            fg = Rgb(138, 43, 226),
            bold,
            italic,
            underline,
            reversed
        )
    );

    println!("\n=== Usage Notes ===");
    println!("• Basic colors (Red, Green, etc.) use the terminal's color palette");
    println!("• Color256(n) uses the standard 256-color palette (0-255)");
    println!("• Rgb(r, g, b) uses true color values (requires TrueColor terminal)");
    println!("• \"#rrggbb\" hex colors are converted to RGB at compile time");
    println!("• Effects can be combined: bold, italic, underline, reversed");
    println!("• RGB/Hex colors provide the most control but require modern terminals");
    println!("• 256-color is widely supported and offers good color variety");
    println!("• Basic ANSI colors work everywhere but are limited to 8 colors");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_macro_formats() {
        // Test that different color formats compile and produce strings
        let basic = styled!("test", fg = Red);
        let color256 = styled!("test", fg = Color256(196));
        let rgb = styled!("test", fg = Rgb(255, 0, 0));
        let hex = styled!("test", fg = "#ff0000");

        assert!(!basic.is_empty());
        assert!(!color256.is_empty());
        assert!(!rgb.is_empty());
        assert!(!hex.is_empty());

        // Test effects
        let effects = styled!("test", bold, italic, underline);
        assert!(!effects.is_empty());
    }

    #[test]
    fn test_combined_styling() {
        let combined = styled!("test", fg = Rgb(255, 100, 0), bold, italic);
        assert!(combined.contains("test"));
        assert!(combined.contains("\x1b[")); // Contains ANSI codes
    }

    #[test]
    fn test_hex_colors() {
        let red_hex = styled!("test", fg = "#ff0000");
        let green_hex = styled!("test", fg = "#00ff00");
        let blue_hex = styled!("test", fg = "#0000ff");

        assert!(!red_hex.is_empty());
        assert!(!green_hex.is_empty());
        assert!(!blue_hex.is_empty());
    }
}
