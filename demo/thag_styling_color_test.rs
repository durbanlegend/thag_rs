/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

/// Thag Styling Color Output Test
///
/// This script tests what thag_styling actually outputs when THAG_COLOR_MODE
/// is set. Unlike the diagnostic comparison scripts, this shows the real
/// escape sequences that thag_styling generates based on the detected
/// color support mode.
//# Purpose: Test actual thag_styling color output with THAG_COLOR_MODE environment variable
//# Categories: terminal, color, testing, styling
use thag_styling::{Style, TermAttributes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Thag Styling Color Output Test");
    println!("==================================");
    println!("Testing what thag_styling actually outputs with different color modes");
    println!();

    // Show environment
    display_environment();

    // Get terminal attributes
    let term_attrs = TermAttributes::get_or_init();
    println!("🔍 Detected Color Support: {:?}", term_attrs.color_support);
    println!();

    // Test thag_styling's actual color output
    test_thag_styling_colors(&term_attrs);

    // Show what escape sequences are actually being generated
    analyze_escape_sequences(&term_attrs);

    Ok(())
}

fn display_environment() {
    println!("📊 Environment:");
    let vars = [
        "THAG_THEME",
        "THAG_COLOR_MODE",
        "FORCE_COLOR",
        "NO_COLOR",
        "TERM_PROGRAM",
        "COLORTERM",
    ];
    for var in &vars {
        match std::env::var(var) {
            Ok(value) => println!("   {}: {}", var, value),
            Err(_) => println!("   {}: <not set>", var),
        }
    }
    println!();
}

fn test_thag_styling_colors(term_attrs: &TermAttributes) {
    println!("🎨 Thag Styling Color Output:");
    println!("=============================");
    println!("These are the actual colors thag_styling generates:");
    println!();

    let theme = &term_attrs.theme;

    // Test semantic colors using thag_styling
    let test_styles = [
        (&theme.palette.error, "Error", "Should be red/crimson"),
        (&theme.palette.warning, "Warning", "Should be yellow/orange"),
        (&theme.palette.success, "Success", "Should be green"),
        (&theme.palette.info, "Info", "Should be blue/cyan"),
        (&theme.palette.emphasis, "Emphasis", "Should be bright/bold"),
        (&theme.palette.subtle, "Subtle", "Should be muted/dim"),
        (&theme.palette.code, "Code", "Should be distinct"),
        (&theme.palette.heading1, "Heading1", "Should be prominent"),
        (&theme.palette.heading2, "Heading2", "Should be secondary"),
        (&theme.palette.normal, "Normal", "Should be default text"),
    ];

    for (style, name, description) in &test_styles {
        // Use thag_styling to render the color
        let colored_text = style.paint(format!("██████ {}", name));
        println!("   {} - {}", colored_text, description);
    }
    println!();
}

fn analyze_escape_sequences(term_attrs: &TermAttributes) {
    println!("🔬 Escape Sequence Analysis:");
    println!("=============================");
    println!("Analyzing what escape sequences thag_styling generates:");
    println!();

    let theme = &term_attrs.theme;

    // Analyze a few key colors
    let colors_to_analyze = [
        (&theme.palette.error, "Error"),
        (&theme.palette.success, "Success"),
        (&theme.palette.info, "Info"),
        (&theme.palette.warning, "Warning"),
    ];

    for (style, name) in &colors_to_analyze {
        println!("🔍 {} color implementation:", name);

        if let Some(fg) = &style.foreground {
            match &fg.value {
                thag_styling::ColorValue::TrueColor { rgb } => {
                    println!("   Type: TrueColor RGB");
                    println!("   RGB Values: ({}, {}, {})", rgb[0], rgb[1], rgb[2]);
                    println!(
                        "   Escape Sequence: ESC[38;2;{};{};{}m",
                        rgb[0], rgb[1], rgb[2]
                    );
                    println!("   💡 This uses 24-bit RGB color");
                }
                thag_styling::ColorValue::Color256 { color256 } => {
                    println!("   Type: 256-Color Palette");
                    println!("   Color Index: {}", color256);
                    println!("   Escape Sequence: ESC[38;5;{}m", color256);
                    println!("   💡 This uses the 256-color palette");
                }
                thag_styling::ColorValue::Basic { index, .. } => {
                    println!("   Type: Basic ANSI Color");
                    println!("   Color Index: {}", index);
                    println!("   Escape Sequence: ESC[38;5;{}m", index);
                    println!("   💡 This uses basic 16-color ANSI");
                }
            }
        } else {
            println!("   No foreground color defined");
        }

        // Show the actual rendered output
        let sample = style.paint("████ SAMPLE");
        println!("   Rendered: {}", sample);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_analysis() {
        let term_attrs = TermAttributes::get_or_init();

        // Test that we can analyze colors without panicking
        let error_style = &term_attrs.theme.palette.error;
        assert!(error_style.foreground.is_some());
    }
}
