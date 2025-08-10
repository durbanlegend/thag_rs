//! Diagnostic example to troubleshoot thag_styling issues
//!
//! This example helps diagnose environment differences and feature detection issues.
//!
//! Run with:
//! ```bash
//! # Test with minimal features
//! cargo run --example diagnostic --features "basic"
//!
//! # Test with color detection
//! cargo run --example diagnostic --features "color_detect"
//!
//! # Test with integrations
//! cargo run --example diagnostic --features "color_detect,crossterm_support,ratatui_support"
//!
//! # Test with full features
//! cargo run --example diagnostic --features "full"
//! ```

use thag_styling::{Role, Style, TermAttributes};

fn main() {
    println!("🔧 Thag Styling Diagnostic Tool\n");

    // Section 1: Environment Info
    print_environment_info();

    // Section 2: Feature Detection
    print_feature_detection();

    // Section 3: Raw Color Detection
    #[cfg(feature = "color_detect")]
    print_raw_color_detection();

    // Section 4: Theme Analysis
    print_theme_analysis();

    // Section 5: Style Analysis
    print_style_analysis();

    // Section 6: Integration Tests
    #[cfg(feature = "crossterm_support")]
    test_crossterm_integration();

    #[cfg(feature = "ratatui_support")]
    test_ratatui_integration();

    #[cfg(feature = "nu_ansi_term_support")]
    test_nu_ansi_term_integration();

    // Section 7: Strategy Comparison
    #[cfg(feature = "color_detect")]
    test_strategy_comparison();

    println!("\n✅ Diagnostic complete!");
    println!("\n💡 If you're seeing basic colors instead of rich themed colors,");
    println!("   make sure to include the 'color_detect' feature:");
    println!("   cargo run --example diagnostic --features \"color_detect,your_integration\"");
}

fn print_environment_info() {
    println!("1. 🌍 Environment Information:");
    println!("   OS: {}", std::env::consts::OS);
    println!("   Architecture: {}", std::env::consts::ARCH);

    if let Ok(term) = std::env::var("TERM") {
        println!("   TERM: {}", term);
    } else {
        println!("   TERM: Not set");
    }

    if let Ok(colorterm) = std::env::var("COLORTERM") {
        println!("   COLORTERM: {}", colorterm);
    } else {
        println!("   COLORTERM: Not set");
    }

    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        println!("   TERM_PROGRAM: {}", term_program);
    }

    println!();
}

fn print_feature_detection() {
    println!("2. 🎚️ Feature Detection:");

    #[cfg(feature = "basic")]
    println!("   ✅ basic");
    #[cfg(not(feature = "basic"))]
    println!("   ❌ basic");

    #[cfg(feature = "color_detect")]
    println!("   ✅ color_detect");
    #[cfg(not(feature = "color_detect"))]
    println!("   ❌ color_detect");

    #[cfg(feature = "config")]
    println!("   ✅ config");
    #[cfg(not(feature = "config"))]
    println!("   ❌ config");

    #[cfg(feature = "crossterm_support")]
    println!("   ✅ crossterm_support");
    #[cfg(not(feature = "crossterm_support"))]
    println!("   ❌ crossterm_support");

    #[cfg(feature = "ratatui_support")]
    println!("   ✅ ratatui_support");
    #[cfg(not(feature = "ratatui_support"))]
    println!("   ❌ ratatui_support");

    #[cfg(feature = "nu_ansi_term_support")]
    println!("   ✅ nu_ansi_term_support");
    #[cfg(not(feature = "nu_ansi_term_support"))]
    println!("   ❌ nu_ansi_term_support");

    println!();
}

#[cfg(feature = "color_detect")]
fn print_raw_color_detection() {
    use thag_common::terminal;

    println!("3. 🎨 Raw Color Detection:");

    let (color_support, term_bg_rgb) = terminal::detect_term_capabilities();
    println!("   Color Support: {:?}", color_support);
    println!("   Background RGB: {:?}", term_bg_rgb);

    let is_light = thag_common::terminal::is_light_color(*term_bg_rgb);
    println!("   Is Light Background: {}", is_light);

    let hex = format!(
        "{:02x}{:02x}{:02x}",
        term_bg_rgb.0, term_bg_rgb.1, term_bg_rgb.2
    );
    println!("   Background Hex: #{}", hex);

    println!();
}

fn print_theme_analysis() {
    println!("4. 🎭 Theme Analysis:");

    let term_attrs = TermAttributes::get_or_init();
    println!("   How Initialized: {:?}", term_attrs.how_initialized);
    println!("   Color Support: {:?}", term_attrs.color_support);
    println!("   Background Luma: {:?}", term_attrs.term_bg_luma);
    println!("   Theme Name: {}", term_attrs.theme.name);
    println!("   Theme Is Builtin: {}", term_attrs.theme.is_builtin);
    println!(
        "   Theme Min Color Support: {:?}",
        term_attrs.theme.min_color_support
    );

    if let Some(rgb) = term_attrs.term_bg_rgb {
        println!("   Term BG RGB: RGB({}, {}, {})", rgb.0, rgb.1, rgb.2);
    } else {
        println!("   Term BG RGB: None");
    }

    println!();
}

fn print_style_analysis() {
    println!("5. 🎨 Style Analysis:");

    let test_roles = [
        Role::Success,
        Role::Error,
        Role::Warning,
        Role::Info,
        Role::Code,
        Role::Normal,
    ];

    for role in test_roles {
        let style = Style::from(role);

        print!("   {:12}: ", format!("{:?}", role));

        if let Some(color_info) = &style.foreground {
            match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => {
                    println!("TrueColor RGB({}, {}, {})", rgb[0], rgb[1], rgb[2]);
                }
                thag_styling::ColorValue::Color256 { color256 } => {
                    println!("Color256({})", color256);
                }
                thag_styling::ColorValue::Basic { basic } => {
                    println!("Basic({:?})", basic);
                }
            }
        } else {
            println!("No foreground color");
        }
    }

    println!();
}

#[cfg(feature = "crossterm_support")]
fn test_crossterm_integration() {
    use crossterm::style::Color as CrossColor;
    use thag_styling::ThemedStyle;

    println!("6. 🔧 Crossterm Integration Test:");

    let success_color = CrossColor::themed(Role::Success);
    let error_color = CrossColor::themed(Role::Error);

    println!("   Success: {:?}", success_color);
    println!("   Error: {:?}", error_color);
    println!();
}

#[cfg(feature = "ratatui_support")]
fn test_ratatui_integration() {
    use ratatui::style::Color as RataColor;
    use thag_styling::ThemedStyle;

    println!("7. 📊 Ratatui Integration Test:");

    let success_color = RataColor::themed(Role::Success);
    let error_color = RataColor::themed(Role::Error);

    println!("   Success: {:?}", success_color);
    println!("   Error: {:?}", error_color);
    println!();
}

#[cfg(feature = "nu_ansi_term_support")]
fn test_nu_ansi_term_integration() {
    use nu_ansi_term::Color as NuColor;
    use thag_styling::ThemedStyle;

    println!("8. 🐚 Nu-ANSI-Term Integration Test:");

    let success_color = NuColor::themed(Role::Success);
    let error_color = NuColor::themed(Role::Error);

    println!("   Success: {:?}", success_color);
    println!("   Error: {:?}", error_color);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_runs() {
        // Just ensure the diagnostic doesn't panic
        print_environment_info();
        print_feature_detection();
        print_theme_analysis();
        print_style_analysis();
    }

    #[cfg(feature = "color_detect")]
    #[test]
    fn test_color_detection() {
        print_raw_color_detection();
    }
}

#[cfg(feature = "color_detect")]
fn test_strategy_comparison() {
    use thag_styling::ColorInitStrategy;

    println!("9. 🔄 Strategy Comparison:");

    // Test what determine() returns
    let determine_strategy = ColorInitStrategy::determine();
    println!(
        "   ColorInitStrategy::determine() = {:?}",
        determine_strategy
    );

    // Debug the color detection steps in detail
    println!("   Detailed color detection breakdown:");

    // Check environment variables that could affect detection
    println!("     Environment checks:");
    if std::env::var("TEST_ENV").is_ok() {
        println!("       ⚠️ TEST_ENV is set - this forces fallback colors!");
    } else {
        println!("       ✅ TEST_ENV not set");
    }

    if let Ok(term) = std::env::var("TERM") {
        println!("       TERM = {}", term);
    }

    if let Ok(colorterm) = std::env::var("COLORTERM") {
        println!("       COLORTERM = {}", colorterm);
    }

    // Check raw mode status
    println!("     Raw mode status:");
    match crossterm::terminal::is_raw_mode_enabled() {
        Ok(raw_mode) => println!("       Raw mode enabled: {}", raw_mode),
        Err(e) => {
            println!("       ⚠️ Raw mode check failed: {:?}", e);
            println!("       This causes fallback to (0,0,0) background!");
        }
    }

    // Test color support detection specifically
    println!("     Color support detection:");
    #[cfg(feature = "color_detect")]
    {
        use supports_color::{on, Stream};
        match on(Stream::Stdout) {
            Some(level) => {
                println!(
                    "       Color level detected: has_16m={}, has_256={}, has_basic={}",
                    level.has_16m, level.has_256, level.has_basic
                );
                let support = if level.has_16m {
                    thag_styling::ColorSupport::TrueColor
                } else if level.has_256 {
                    thag_styling::ColorSupport::Color256
                } else {
                    thag_styling::ColorSupport::Basic
                };
                println!("       Mapped to: {:?}", support);
            }
            None => {
                println!("       ⚠️ No color support detected (returns None)");
                println!("       This maps to ColorSupport::None");
            }
        }
    }

    // Test fresh detection without caching
    println!("   Fresh color detection (bypassing cache):");
    let (fresh_color_support, fresh_bg_rgb) = thag_common::terminal::detect_term_capabilities();
    println!("     - fresh_color_support: {:?}", fresh_color_support);
    println!("     - fresh_bg_rgb: {:?}", fresh_bg_rgb);
    let fresh_hex = format!(
        "{:02x}{:02x}{:02x}",
        fresh_bg_rgb.0, fresh_bg_rgb.1, fresh_bg_rgb.2
    );
    println!("     - fresh_hex: #{}", fresh_hex);
    let fresh_is_light = thag_common::terminal::is_light_color(*fresh_bg_rgb);
    println!("     - fresh_is_light: {}", fresh_is_light);

    // Test if TermAttributes is using stale data
    println!("   TermAttributes initialization status:");
    println!(
        "     - is_initialized: {}",
        TermAttributes::is_initialized()
    );

    // Test forced Match strategy
    println!("   Testing ColorInitStrategy::Match:");
    let match_attrs = TermAttributes::initialize(&ColorInitStrategy::Match);
    println!("     - how_initialized: {:?}", match_attrs.how_initialized);
    println!("     - color_support: {:?}", match_attrs.color_support);
    println!("     - theme.name: {}", match_attrs.theme.name);
    if let Some(rgb) = match_attrs.term_bg_rgb {
        println!("     - term_bg_rgb: RGB({}, {}, {})", rgb.0, rgb.1, rgb.2);
        let hex = format!("{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2);
        println!("     - background_hex: #{}", hex);
    } else {
        println!("     - term_bg_rgb: None");
    }

    // Compare with current (determine-based) initialization
    let current_attrs = TermAttributes::get_or_init();
    println!("   Current (determine-based) initialization:");
    println!(
        "     - how_initialized: {:?}",
        current_attrs.how_initialized
    );
    println!("     - color_support: {:?}", current_attrs.color_support);
    println!("     - theme.name: {}", current_attrs.theme.name);
    if let Some(rgb) = current_attrs.term_bg_rgb {
        println!("     - term_bg_rgb: RGB({}, {}, {})", rgb.0, rgb.1, rgb.2);
        let hex = format!("{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2);
        println!("     - background_hex: #{}", hex);
    } else {
        println!("     - term_bg_rgb: None");
    }

    // Check for discrepancies
    let fresh_matches_attrs = *fresh_bg_rgb == current_attrs.term_bg_rgb.unwrap_or((0, 0, 0));
    if !fresh_matches_attrs {
        println!("   ⚠️  MISMATCH: Fresh detection differs from TermAttributes!");
        println!(
            "      Fresh detection: RGB({}, {}, {})",
            fresh_bg_rgb.0, fresh_bg_rgb.1, fresh_bg_rgb.2
        );
        println!("      TermAttributes: {:?}", current_attrs.term_bg_rgb);
        println!("      This suggests TermAttributes caching stale data.");
    }

    // Show if strategies are different
    if match_attrs.theme.name != current_attrs.theme.name {
        println!("   ⚠️  STRATEGY MISMATCH: Different themes selected!");
        println!("      Match strategy: {}", match_attrs.theme.name);
        println!("      Determine strategy: {}", current_attrs.theme.name);
    } else {
        println!("   ✅ Both strategies selected the same theme");
    }

    println!();
}
