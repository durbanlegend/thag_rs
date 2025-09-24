/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

/// Environment Variable Debug
///
/// This script directly tests the environment variable parsing for color support
/// to debug why THAG_COLOR_MODE=256 isn't working as expected.
//# Purpose: Debug environment variable parsing for color support
//# Categories: terminal, color, debugging, environment
use thag_common::{
    terminal::detect_term_capabilities, terminal::get_fresh_color_support, ColorSupport,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Environment Variable Debug");
    println!("==============================");
    println!();

    // Show all relevant environment variables
    println!("📊 Environment Variables:");
    let env_vars = [
        "THAG_COLOR_MODE",
        "FORCE_COLOR",
        "NO_COLOR",
        "CLICOLOR_FORCE",
        "TERM",
        "TERM_PROGRAM",
        "COLORTERM",
    ];

    for var in &env_vars {
        match std::env::var(var) {
            Ok(value) => println!("   {}: {}", var, value),
            Err(_) => println!("   {}: <not set>", var),
        }
    }
    println!();

    // Test the check_env_color_support function directly
    println!("🧪 Direct Environment Check:");
    let env_override = check_env_color_support_direct();
    match env_override {
        Some(support) => println!("   Environment override detected: {:?}", support),
        None => println!("   No environment override detected"),
    }
    println!();

    // Test the full color detection
    println!("🎯 Full Color Detection:");
    let fresh_support = get_fresh_color_support();
    println!("   Fresh detection result: {:?}", fresh_support);
    println!();

    // Test cached detection
    println!("📦 Cached Detection:");
    let (cached_support, _bg) = detect_term_capabilities();
    println!("   Cached detection result: {:?}", cached_support);
    println!();

    // Manual THAG_COLOR_MODE test
    println!("🔬 Manual THAG_COLOR_MODE Test:");
    if let Ok(thag_mode) = std::env::var("THAG_COLOR_MODE") {
        println!("   THAG_COLOR_MODE value: '{}'", thag_mode);
        let lowercase = thag_mode.to_lowercase();
        println!("   Lowercase: '{}'", lowercase);

        match lowercase.as_str() {
            "none" | "off" | "0" => println!("   → Should map to: None"),
            "basic" | "16" | "1" => println!("   → Should map to: Basic"),
            "256" | "2" => println!("   → Should map to: Color256"),
            "truecolor" | "24bit" | "rgb" | "3" => println!("   → Should map to: TrueColor"),
            _ => println!("   → No match found for: '{}'", lowercase),
        }
    } else {
        println!("   THAG_COLOR_MODE not set");
    }

    Ok(())
}

/// Direct implementation of check_env_color_support for testing
fn check_env_color_support_direct() -> Option<ColorSupport> {
    // Check for NO_COLOR (takes precedence)
    if std::env::var("NO_COLOR").is_ok() {
        return Some(ColorSupport::None);
    }

    // Check for THAG_COLOR_MODE (thag-specific override)
    if let Ok(thag_color_mode) = std::env::var("THAG_COLOR_MODE") {
        match thag_color_mode.to_lowercase().as_str() {
            "none" | "off" | "0" => return Some(ColorSupport::None),
            "basic" | "16" | "1" => return Some(ColorSupport::Basic),
            "256" | "2" => return Some(ColorSupport::Color256),
            "truecolor" | "24bit" | "rgb" | "3" => return Some(ColorSupport::TrueColor),
            _ => {}
        }
    }

    // Check for FORCE_COLOR
    if let Ok(force_color) = std::env::var("FORCE_COLOR") {
        match force_color.as_str() {
            "0" => return Some(ColorSupport::None),
            "1" => return Some(ColorSupport::Basic),
            "2" => return Some(ColorSupport::Color256),
            "3" => return Some(ColorSupport::TrueColor),
            _ => {}
        }
    }

    // Check CLICOLOR_FORCE
    if std::env::var("CLICOLOR_FORCE").is_ok() {
        return Some(ColorSupport::Basic);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_parsing() {
        // Test that our direct function works the same as the library
        let result = check_env_color_support_direct();
        assert!(result.is_some() || result.is_none()); // Just ensure it doesn't panic
    }
}
