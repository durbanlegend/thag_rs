//! Test the new concise theme styling API
//!
//! This demonstrates the new Theme methods for styling text with specific roles,
//! making it easy to use a "guest" theme instead of the active theme.

use thag_styling::{ColorInitStrategy, Role, Styleable, StyledStringExt, TermAttributes, Theme};

fn main() {
    println!("🎨 Testing Theme Styling API\n");

    // Initialize styling
    let strategy = ColorInitStrategy::Default;
    let _attrs = TermAttributes::initialize(&strategy);

    // Get a theme to use for demonstration
    let theme = match Theme::get_builtin("Basic Dark") {
        Ok(theme) => theme,
        Err(_) => {
            println!("Could not load Basic Dark theme, using fallback");
            return;
        }
    };

    println!("Using theme: {}\n", theme.name);

    // Test all the new theme styling methods
    println!("=== New Theme Styling Methods ===");

    theme.heading1("This is a Heading 1").println();
    theme.heading2("This is a Heading 2").println();
    theme.heading3("This is a Heading 3").println();

    println!();
    theme.error("This is an error message").println();
    theme.warning("This is a warning message").println();
    theme.success("This is a success message").println();
    theme.info_text("This is an info message").println();

    println!();
    theme.emphasis("This text is emphasized").println();
    theme.code("println!(\"Hello, world!\")").println();
    theme.normal("This is normal text").println();
    theme.subtle("This is subtle text").println();
    theme.hint("This is a hint").println();
    theme.debug("This is debug text").println();
    theme.link("https://example.com").println();
    theme.quote("\"This is a quoted text\"").println();
    theme.commentary("// This is a comment").println();

    println!();

    // Test embedded usage (the main benefit)
    println!("=== Embedded Usage Examples ===");

    theme
        .normal(&format!(
            "Status: {} | Errors: {} | Warnings: {}",
            theme.success("OK"),
            theme.error("0"),
            theme.warning("2")
        ))
        .println();

    theme
        .normal(&format!(
            "File: {} contains {} function calls",
            theme.code("main.rs"),
            theme.emphasis("42")
        ))
        .println();

    theme
        .heading2(&format!("Section: {}", theme.info_text("Configuration")))
        .println();

    println!();

    // Compare with the old verbose way
    println!("=== Comparison: Old vs New Syntax ===");

    // Old way (verbose)
    let old_way = format!(
        "Status: {}",
        "OK".style_with(theme.style_for(Role::Success))
    );
    old_way.style_with(theme.style_for(Role::Normal)).println();

    // New way (concise)
    theme
        .normal(&format!("Status: {}", theme.success("OK")))
        .println();

    println!("\n✨ The new API is much more concise and readable!");

    // Test that different themes produce different output
    if let Ok(light_theme) = Theme::get_builtin("Basic Light") {
        println!("\n=== Same Text, Different Theme ===");
        println!("Dark theme:");
        theme.error("This is an error").println();

        println!("Light theme:");
        light_theme.error("This is an error").println();
    }

    println!("\n✅ Theme styling API test complete!");
}
