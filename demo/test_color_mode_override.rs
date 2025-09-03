/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

//! Test Color Mode Override
//!
//! This script tests the THAG_COLOR_MODE environment variable override
//! functionality to force specific color modes in thag_styling.
//! This is particularly useful for working around terminal issues
//! like Zed's RGB truecolor handling problems.

//# Purpose: Test THAG_COLOR_MODE environment variable functionality
//# Categories: terminal, colors, testing, configuration

use thag_styling::TermAttributes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎛️  Color Mode Override Test");
    println!("============================");
    println!("Testing THAG_COLOR_MODE environment variable");
    println!();

    // Display current environment
    display_environment_info();

    // Get terminal attributes and display detected capabilities
    let term_attrs = TermAttributes::get_or_init();
    println!("🔍 Detected Color Support: {:?}", term_attrs.color_support);
    println!("🌓 Background Luma: {:?}", term_attrs.term_bg_luma);
    println!();

    // Test color rendering with current settings
    test_color_rendering(&term_attrs);

    // Show configuration examples
    show_configuration_examples();

    Ok(())
}

fn display_environment_info() {
    println!("📊 Current Environment:");
    println!("=======================");

    let env_vars = [
        ("TERM", "Terminal type"),
        ("TERM_PROGRAM", "Terminal program"),
        ("COLORTERM", "Color terminal capability"),
        ("THAG_COLOR_MODE", "Thag color mode override"),
        ("FORCE_COLOR", "Force color override"),
        ("NO_COLOR", "Disable colors"),
    ];

    for (var, description) in &env_vars {
        match std::env::var(var) {
            Ok(value) => println!("   {:<15}: {} ({})", var, value, description),
            Err(_) => println!("   {:<15}: <not set> ({})", var, description),
        }
    }
    println!();
}

fn test_color_rendering(term_attrs: &TermAttributes) {
    println!("🎨 Color Rendering Test:");
    println!("========================");
    println!("Colors rendered using detected color support mode:");
    println!();

    let theme = &term_attrs.theme;

    // Test semantic colors
    let test_styles = [
        (&theme.palette.error, "Error", "Should be red/crimson"),
        (&theme.palette.warning, "Warning", "Should be yellow/orange"),
        (&theme.palette.success, "Success", "Should be green"),
        (&theme.palette.info, "Info", "Should be blue/cyan"),
        (&theme.palette.emphasis, "Emphasis", "Should be bright/bold"),
        (&theme.palette.subtle, "Subtle", "Should be muted/dim"),
        (&theme.palette.code, "Code", "Should be distinct"),
        (&theme.palette.heading1, "Heading1", "Should be prominent"),
    ];

    for (style, name, description) in &test_styles {
        let colored_text = style.paint(format!("██████ {}", name));
        println!("   {} - {}", colored_text, description);
    }

    println!();

    // Test RGB vs palette comparison if we can determine the color values
    println!("🔬 Color Implementation Analysis:");
    println!("=================================");

    let error_color = &theme.palette.error;
    println!("Error color implementation:");
    if let Some(fg) = &error_color.foreground {
        match &fg.value {
            thag_styling::ColorValue::TrueColor { rgb } => {
                println!("   Mode: TrueColor RGB({}, {}, {})", rgb[0], rgb[1], rgb[2]);
                println!("   Escape: ESC[38;2;{};{};{}m", rgb[0], rgb[1], rgb[2]);
            }
            thag_styling::ColorValue::Color256 { color256 } => {
                println!("   Mode: 256-Color Index {}", color256);
                println!("   Escape: ESC[38;5;{}m", color256);
            }
            thag_styling::ColorValue::Basic { index, .. } => {
                println!("   Mode: Basic ANSI Color {}", index);
                println!("   Escape: ESC[38;5;{}m", index);
            }
        }
    } else {
        println!("   No foreground color set");
    }
}

fn show_configuration_examples() {
    println!("⚙️  Configuration Examples:");
    println!("============================");
    println!("Use these environment variables to override color detection:");
    println!();

    println!("🔧 Force specific color modes:");
    println!("   export THAG_COLOR_MODE=none      # Disable all colors");
    println!("   export THAG_COLOR_MODE=basic     # 16 basic ANSI colors");
    println!("   export THAG_COLOR_MODE=256       # 256-color palette");
    println!("   export THAG_COLOR_MODE=truecolor # 24-bit RGB colors");
    println!();

    println!("🐛 Workaround for Zed terminal RGB issues:");
    println!("   export THAG_COLOR_MODE=256");
    println!("   # This forces palette colors which work correctly in Zed");
    println!();

    println!("🚫 Disable colors entirely:");
    println!("   export NO_COLOR=1");
    println!("   # This disables all color output");
    println!();

    println!("💡 Testing different modes:");
    println!("   THAG_COLOR_MODE=basic cargo run your_program");
    println!("   THAG_COLOR_MODE=256 cargo run your_program");
    println!("   THAG_COLOR_MODE=truecolor cargo run your_program");
    println!();

    println!("🔍 Current recommendation based on your terminal:");
    match std::env::var("TERM_PROGRAM").ok().as_deref() {
        Some("zed") => {
            println!("   ⚠️  Zed detected: Use THAG_COLOR_MODE=256");
            println!("   Reason: Zed has issues with RGB truecolor sequences");
        }
        Some("iTerm.app") => {
            println!("   ✅ iTerm2 detected: TrueColor should work fine");
        }
        Some("Apple_Terminal") => {
            println!("   📱 Apple Terminal: 256-color recommended");
        }
        Some("WezTerm") => {
            println!("   🚀 WezTerm detected: Excellent color support");
        }
        _ => {
            println!("   🤔 Unknown terminal: Try 256-color if RGB looks wrong");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        // Test that we can read environment variables
        let term = std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string());
        assert!(!term.is_empty());
    }
}
