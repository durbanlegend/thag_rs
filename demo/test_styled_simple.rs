/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_styling = { version = "0.2, thag-auto" }
*/

/// Simple test for enhanced styled! macro
///
/// Tests the four color formats: Basic ANSI, 256-color, RGB, and Hex
//# Purpose: Simple test of enhanced styled! macro with all color formats
//# Categories: color, macros, styling, testing
use thag_proc_macros::{ansi_styling_support, styled};

// Enable the ANSI styling support
ansi_styling_support! {}

fn main() {
    println!("🎨 Simple styled! Macro Test\n");

    // Basic ANSI colors
    println!("=== Basic ANSI Colors ===");
    println!("{}", styled!("Red text", fg = Red));
    println!("{}", styled!("Green bold", fg = Green, bold));
    println!("{}", styled!("Blue italic", fg = Blue, italic));

    // 256-color palette
    println!("\n=== 256-Color Palette ===");
    println!("{}", styled!("Bright red", fg = Color256(196)));
    println!("{}", styled!("Orange bold", fg = Color256(214), bold));
    println!("{}", styled!("Purple italic", fg = Color256(93), italic));

    // True RGB colors
    println!("\n=== RGB Colors ===");
    println!("{}", styled!("Crimson", fg = Rgb(220, 20, 60)));
    println!("{}", styled!("Orange bold", fg = Rgb(255, 165, 0), bold));
    println!("{}", styled!("Lime italic", fg = Rgb(50, 205, 50), italic));

    // Hex colors
    println!("\n=== Hex Colors ===");
    println!("{}", styled!("Hex red", fg = "#ff0000"));
    println!("{}", styled!("Hex orange bold", fg = "#ffa500", bold));
    println!("{}", styled!("Hex purple italic", fg = "#800080", italic));

    // Effects
    println!("\n=== Text Effects ===");
    println!("{}", styled!("Bold text", bold));
    println!("{}", styled!("Italic text", italic));
    println!("{}", styled!("Underlined text", underline));
    println!("{}", styled!("Reversed text", reversed));

    // Combined
    println!("\n=== Combined ===");
    println!(
        "{}",
        styled!("All effects", fg = "#ff6347", bold, italic, underline)
    );

    println!("\n✅ All styled! macro formats working!");
}
