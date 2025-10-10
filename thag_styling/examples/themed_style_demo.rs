//! Demonstration of the ThemedStyle trait across multiple styling crates
//!
//! This example shows how to use thag_styling's ThemedStyle trait to create
//! consistent, theme-aware styling across different terminal UI libraries.
/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/
//!
//! Run with different features to see different integrations:
//! ```bash
//! # Basic demo with all integrations (REQUIRES color_detect for rich colors!)
//! cargo run --example themed_style_demo --features "color_detect,crossterm_support,console_support,ratatui_support,nu_ansi_term_support"
//!
//! # Just crossterm (with rich colors)
//! cargo run --example themed_style_demo --features "color_detect,crossterm_support"
//!
//! # Just ratatui (with rich colors)
//! cargo run --example themed_style_demo --features "color_detect,ratatui_support"
//!
//! # Without color_detect (falls back to basic ANSI colors)
//! cargo run --example themed_style_demo --features "crossterm_support"
//! ```

use thag_styling::{Role, ThemedStyle};

fn main() {
    println!("🎨 Thag Styling ThemedStyle Trait Demo\n");

    // Check theme status
    check_theme_status();

    // Demonstrate role-based theming
    demonstrate_roles();

    // Show integration-specific features
    #[cfg(feature = "crossterm_support")]
    demonstrate_crossterm();

    #[cfg(feature = "console_support")]
    demonstrate_console();

    #[cfg(feature = "ratatui_support")]
    demonstrate_ratatui();

    #[cfg(feature = "nu_ansi_term_support")]
    demonstrate_nu_ansi_term();
}

fn demonstrate_roles() {
    println!("📝 Role-based styling across libraries:\n");

    let roles = [
        (Role::Success, "✓ Operation successful"),
        (Role::Error, "✗ Something went wrong"),
        (Role::Warning, "⚠ Proceed with caution"),
        (Role::Info, "ℹ Information message"),
        (Role::Code, "fn main() {{ println!(\"Hello!\"); }}"),
        (Role::Emphasis, "This is important"),
        (Role::Subtle, "Less important details"),
        (Role::Normal, "Regular text content"),
    ];

    for (role, message) in roles {
        println!(
            "  {role:12}: {}",
            thag_styling::paint_for_role(role, message)
        );
    }
    println!();
}

/// Check what theme was selected and provide guidance
fn check_theme_status() {
    use thag_styling::{ColorSupport, TermAttributes};

    let term_attrs = TermAttributes::get_or_init();

    println!("🎭 Theme Status:");
    println!(
        "   Theme: {} ({})",
        term_attrs.theme.name, term_attrs.color_support
    );

    // If we ended up with basic theme despite having good color support, explain why
    if term_attrs.theme.name.contains("Basic") && term_attrs.color_support >= ColorSupport::Color256
    {
        println!(
            "   ⚠️  Note: You have {:?} color support but got a basic theme.",
            term_attrs.color_support
        );
        println!("      This is likely because the 'color_detect' feature is not enabled.");
        println!("      Without 'color_detect', thag_styling falls back to basic themes.");
        println!("      The integration colors will work, but they'll be basic ANSI colors");
        println!("      instead of rich themed colors.");
        println!();
        println!("   💡 Solution: Add required feature to Cargo.toml:");
        println!("      [dependencies.thag_styling]");
        println!("      features = [\"color_detect\", \"your_integration_support\"]");
        println!();
        println!("      Then run: cargo run --example themed_style_demo --features \"color_detect,crossterm_support\"");
    } else {
        println!("   ✅ Good theme detected!");
    }
    println!();
}

#[cfg(feature = "crossterm_support")]
fn demonstrate_crossterm() {
    use crossterm::style::ContentStyle;
    use thag_styling::integrations::crossterm_integration::{crossterm_helpers, ThemedStylize};

    println!("🔧 Crossterm Integration:\n");

    // Using ThemedStyle trait
    let success_style = ContentStyle::themed(Role::Success);
    let error_style = ContentStyle::themed(Role::Error);

    println!("  Direct ThemedStyle usage:");
    println!("    Success: {:?}", success_style);
    println!("    Error:   {:?}", error_style);

    // Using helper functions
    println!("\n  Helper functions:");
    println!("    Success: {:?}", crossterm_helpers::success_style());
    println!("    Warning: {:?}", crossterm_helpers::warning_style());
    println!("    Code:    {:?}", crossterm_helpers::code_style());

    // Using ThemedStylize trait
    println!("\n  ThemedStylize extension:");
    let styled_content = "Themed content".role(Role::Emphasis);
    println!("    Content: {:?}", styled_content);

    println!();
}

#[cfg(feature = "console_support")]
fn demonstrate_console() {
    println!("🖥️  Console Integration:\n");

    // Note: This would require implementing console integration
    // For now, just show the concept
    println!("  (Console integration would be implemented similarly)");
    println!("  Example: console::Style::themed(Role::Success)");
    println!();
}

#[cfg(feature = "ratatui_support")]
fn demonstrate_ratatui() {
    use ratatui::style::{Color, Style, Stylize};
    use thag_styling::{integrations::ratatui_integration::RatatuiStyleExt, ThemedStyle};

    println!("📊 Ratatui Integration:\n");

    // Using ThemedStyle trait
    let success_style = Style::themed(Role::Success);
    let error_color = Color::themed(Role::Error);

    println!("  Direct ThemedStyle usage:");
    println!("    Success style: {:?}", success_style);
    println!("    Error color:   {:?}", error_color);

    // Using extension trait
    let base_style = Style::default().bold();
    let themed_style = base_style.with_role(Role::Warning);

    println!("\n  RatatuiStyleExt usage:");
    println!("    Base style:   {:?}", base_style);
    println!("    Themed style: {:?}", themed_style);

    println!();
}

#[cfg(feature = "nu_ansi_term_support")]
fn demonstrate_nu_ansi_term() {
    use nu_ansi_term::{Color, Style};
    use thag_styling::integrations::nu_ansi_term_integration::{
        reedline_helpers, NuAnsiTermStyleExt,
    };

    println!("🐚 Nu-ANSI-Term Integration:\n");

    // Using ThemedStyle trait
    let success_style = Style::themed(Role::Success);
    let error_color = Color::themed(Role::Error);

    println!("  Direct ThemedStyle usage:");
    println!("    Success style: {:?}", success_style);
    println!("    Error color:   {:?}", error_color);

    // Using reedline helpers
    println!("\n  Reedline helper functions:");
    println!("    Prompt:     {:?}", reedline_helpers::prompt_style());
    println!("    Selection:  {:?}", reedline_helpers::selection_style());
    println!("    Error:      {:?}", reedline_helpers::error_style());

    // Using extension trait
    let base_style = Style {
        is_bold: true,
        ..Default::default()
    };
    let themed_style = base_style.with_role(Role::Info);

    println!("\n  NuAnsiTermStyleExt usage:");
    println!("    Base style:   {:?}", base_style);
    println!("    Themed style: {:?}", themed_style);

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_runs() {
        // Just ensure the demo doesn't panic
        demonstrate_roles();
    }

    #[cfg(feature = "crossterm_support")]
    #[test]
    fn test_crossterm_demo() {
        demonstrate_crossterm();
    }

    #[cfg(feature = "ratatui_support")]
    #[test]
    fn test_ratatui_demo() {
        demonstrate_ratatui();
    }

    #[cfg(feature = "nu_ansi_term_support")]
    #[test]
    fn test_nu_ansi_term_demo() {
        demonstrate_nu_ansi_term();
    }
}
