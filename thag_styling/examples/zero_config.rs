//! Zero-configuration setup example for thag_styling
//!
//! This example demonstrates how thag_styling can be used with zero configuration
//! while automatically detecting terminal capabilities and choosing appropriate themes.
//!
//! Run with:
//! ```bash
//! cargo run --example zero_config --features "full"
//! ```

use std::io::{self};
use thag_styling::{paint_for_role, Role, TermAttributes, ThemedStyle};

fn main() -> io::Result<()> {
    println!("🎨 Thag Styling - Zero Configuration Demo\n");

    // Zero config step 1: Just use it! Terminal detection happens automatically
    show_automatic_detection();

    // Zero config step 2: Cross-library consistency without setup
    show_cross_library_consistency()?;

    // Zero config step 3: Interactive prompts work out of the box
    #[cfg(feature = "inquire_theming")]
    show_interactive_prompts()?;

    // Zero config step 4: Advanced features just work
    show_advanced_features();

    Ok(())
}

fn show_automatic_detection() {
    println!("📡 Automatic Terminal Detection:");
    println!("  (No configuration required - everything detected automatically)\n");

    // Display what was detected
    let term_attrs = TermAttributes::get_or_init();
    println!("  🖥️  Terminal Capabilities:");
    println!("    Color Support: {:?}", term_attrs.color_support);
    println!("    Background:    {:?}", term_attrs.term_bg_luma);
    println!("    Theme:         {}", term_attrs.theme.name);

    if let Some(rgb) = term_attrs.term_bg_rgb {
        println!("    BG Color:      RGB({}, {}, {})", rgb.0, rgb.1, rgb.2);
    }

    println!("    How Set:       {:?}", term_attrs.how_initialized);
    println!();
}

fn show_cross_library_consistency() -> io::Result<()> {
    println!("🔗 Cross-Library Consistency:");
    println!("  (Same colors across all libraries, zero setup)\n");

    let messages = [
        (Role::Success, "✓ Operation completed successfully"),
        (Role::Error, "✗ Critical error occurred"),
        (Role::Warning, "⚠ Warning: proceed with caution"),
        (Role::Info, "ℹ Informational message"),
        (Role::Code, "let result = perform_operation();"),
        (Role::Emphasis, "This text is emphasized"),
        (Role::Heading1, "# Main Section Header"),
        (Role::Heading2, "## Subsection Header"),
        (Role::Normal, "Regular paragraph text"),
        (Role::Subtle, "Less important details"),
    ];

    // Show the same styling works across different contexts
    println!("  📝 Standard Output:");
    for (role, message) in &messages {
        println!(
            "    {}: {}",
            format!("{role:12}"),
            paint_for_role(*role, message)
        );
    }
    println!();

    // Crossterm integration example
    #[cfg(feature = "crossterm_support")]
    {
        use crossterm::style::ContentStyle;
        use crossterm::{execute, style::Print};

        println!("  🔧 Crossterm Integration:");
        let mut stdout = io::stdout();

        execute!(
            stdout,
            Print("    Success: "),
            Print(crossterm::style::StyledContent::new(
                ContentStyle::themed(Role::Success),
                "Crossterm themed content"
            )),
            Print("\n")
        )?;

        execute!(
            stdout,
            Print("    Error:   "),
            Print(crossterm::style::StyledContent::new(
                ContentStyle::themed(Role::Error),
                "Error message in crossterm"
            )),
            Print("\n")
        )?;
        println!();
    }

    // Ratatui integration example
    #[cfg(feature = "ratatui_support")]
    {
        use ratatui::style::Style;

        println!("  📊 Ratatui Integration:");
        let success_style = Style::themed(Role::Success);
        let error_style = Style::themed(Role::Error);

        println!("    Success Style: {:?}", success_style);
        println!("    Error Style:   {:?}", error_style);
        println!();
    }

    // Nu-ANSI-Term integration example
    #[cfg(feature = "nu_ansi_term_support")]
    {
        use nu_ansi_term::Style;

        println!("  🐚 Nu-ANSI-Term Integration:");
        let success_style = Style::themed(Role::Success);
        let error_style = Style::themed(Role::Error);

        println!(
            "    {}",
            success_style.paint("Success: Nu-ANSI-Term themed content")
        );
        println!(
            "    {}",
            error_style.paint("Error: Error message in nu-ansi-term")
        );
        println!();
    }

    Ok(())
}

#[cfg(feature = "inquire_theming")]
fn show_interactive_prompts() -> io::Result<()> {
    use inquire::{Confirm, Select, Text};

    println!("💬 Interactive Prompts:");
    println!("  (Automatically themed to match your terminal)\n");

    // Use the themed inquire config - completely automatic
    let config = thag_styling::themed_inquire_config();

    // Simple text input with theming
    if let Ok(name) = Text::new("What's your name?")
        .with_render_config(config.clone())
        .prompt()
    {
        println!("  Hello, {}! 👋", paint_for_role(Role::Emphasis, &name));
    }

    // Selection with themed options
    let options = vec!["Light Theme", "Dark Theme", "Auto-detect"];
    if let Ok(choice) = Select::new("Theme preference:", options)
        .with_render_config(config.clone())
        .prompt()
    {
        println!("  You chose: {}", paint_for_role(Role::Success, choice));
    }

    // Confirmation with theming
    if let Ok(confirmed) = Confirm::new("Continue with zero-config demo?")
        .with_default(true)
        .with_render_config(config)
        .prompt()
    {
        if confirmed {
            println!("  {}", paint_for_role(Role::Success, "✓ Continuing..."));
        } else {
            println!("  {}", paint_for_role(Role::Info, "Demo stopped by user"));
        }
    }

    println!();
    Ok(())
}

fn show_advanced_features() {
    println!("🚀 Advanced Features (Zero Config):");
    println!("  (All features work automatically)\n");

    // Show color adaptation
    println!("  🎭 Automatic Color Adaptation:");
    let term_attrs = TermAttributes::get_or_init();
    match term_attrs.term_bg_luma {
        thag_styling::TermBgLuma::Light => {
            println!("    Light background detected → Using dark colors for contrast");
        }
        thag_styling::TermBgLuma::Dark => {
            println!("    Dark background detected → Using light colors for contrast");
        }
        _ => {
            println!("    Background auto-detected → Colors automatically optimized");
        }
    }

    // Show capability matching
    println!("\n  🎨 Capability Matching:");
    match term_attrs.color_support {
        thag_styling::ColorSupport::TrueColor => {
            println!("    True color support → Using RGB colors for maximum fidelity");
        }
        thag_styling::ColorSupport::Color256 => {
            println!("    256-color support → Using optimized 256-color palette");
        }
        thag_styling::ColorSupport::Basic => {
            println!("    Basic color support → Using safe 8-color palette");
        }
        _ => {
            println!("    Color support auto-detected → Best available colors selected");
        }
    }

    // Show theme selection
    println!("\n  🌈 Smart Theme Selection:");
    println!(
        "    Current theme: {}",
        paint_for_role(Role::Emphasis, &term_attrs.theme.name)
    );
    println!(
        "    Theme family: {}",
        term_attrs.theme.name.split('_').next().unwrap_or("unknown")
    );
    println!("    Automatically chosen for your terminal setup");

    // Performance info
    println!("\n  ⚡ Performance:");
    println!("    • Zero runtime detection overhead (cached on first use)");
    println!("    • Optimized color calculations");
    println!("    • Minimal memory footprint");
    println!("    • Feature-gated dependencies");

    println!();
}

/// Example of a typical CLI application using zero-config thag_styling
fn example_cli_app() {
    println!("📋 Example CLI Application Output:\n");

    // Simulate a typical CLI tool
    println!("{}", paint_for_role(Role::Heading1, "MyTool v1.0.0"));
    println!(
        "{}",
        paint_for_role(
            Role::Subtle,
            "A sample CLI application with automatic theming"
        )
    );
    println!();

    // Status messages
    println!("{} Initializing...", paint_for_role(Role::Info, "ℹ"));
    println!(
        "{} Loading configuration",
        paint_for_role(Role::Normal, "→")
    );
    println!(
        "{} Configuration loaded",
        paint_for_role(Role::Success, "✓")
    );
    println!();

    // Processing steps
    let steps = [
        ("Validating input", Role::Info),
        ("Processing data", Role::Normal),
        ("Generating output", Role::Normal),
        ("Writing results", Role::Success),
    ];

    for (step, role) in steps {
        println!("{} {}", paint_for_role(role, "→"), step);
    }
    println!();

    // Results
    println!("{}", paint_for_role(Role::Heading2, "## Results"));
    println!(
        "{} Files processed: {}",
        paint_for_role(Role::Normal, "•"),
        paint_for_role(Role::Emphasis, "1,234")
    );
    println!(
        "{} Errors: {}",
        paint_for_role(Role::Normal, "•"),
        paint_for_role(Role::Error, "0")
    );
    println!(
        "{} Warnings: {}",
        paint_for_role(Role::Normal, "•"),
        paint_for_role(Role::Warning, "3")
    );
    println!();

    println!(
        "{}",
        paint_for_role(Role::Success, "✓ Operation completed successfully!")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_config_demo() {
        // Ensure the demo functions don't panic
        show_automatic_detection();
        show_advanced_features();
        example_cli_app();
    }

    #[test]
    fn test_cross_library_consistency() {
        // Test that cross-library demo runs without errors
        assert!(show_cross_library_consistency().is_ok());
    }
}
