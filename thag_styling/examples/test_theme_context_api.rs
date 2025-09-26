//! Comprehensive test for the new theme context API
//!
//! This demonstrates both new approaches for using guest themes:
//! 1. Direct theme methods: theme.error("text")
//! 2. Context switching: theme.with_context(|| { "text".error() })

use thag_styling::{ColorInitStrategy, Styleable, StyledPrint, TermAttributes, Theme};

fn main() {
    println!("🎨 Testing Theme Context API\n");

    // Initialize styling
    let strategy = ColorInitStrategy::Default;
    let _attrs = TermAttributes::get_or_init_with_strategy(&strategy);

    // Get themes for demonstration - use fallback if specific themes not available
    let dark_theme = Theme::get_builtin("basic_dark")
        .or_else(|_| Theme::get_builtin("Basic Dark"))
        .unwrap_or_else(|_| _attrs.theme.clone());
    let light_theme = Theme::get_builtin("basic_light")
        .or_else(|_| Theme::get_builtin("Basic Light"))
        .unwrap_or_else(|_| _attrs.theme.clone());

    println!("Active theme: {}", _attrs.theme.name);
    println!(
        "Guest themes: {} and {}\n",
        dark_theme.name, light_theme.name
    );

    // Demonstrate the difference between active theme and guest theme usage
    println!("=== 1. Using Active Theme (Normal Methods) ===");
    "This uses the active theme".error().println();
    "This also uses the active theme".success().println();
    "Active theme normal text".normal().println();

    println!("\n=== 2. Direct Theme Methods (Guest Theme) ===");
    dark_theme
        .error("This uses the dark theme directly")
        .println();
    dark_theme.success("Dark theme success message").println();
    dark_theme.normal("Dark theme normal text").println();

    light_theme
        .error("This uses the light theme directly")
        .println();
    light_theme.success("Light theme success message").println();
    light_theme.normal("Light theme normal text").println();

    println!("\n=== 3. Context Switching API (Guest Theme) ===");

    // Using dark theme context
    dark_theme.with_context(|| {
        "Inside dark theme context - error".error().println();
        "Inside dark theme context - success".success().println();
        "Inside dark theme context - normal".normal().println();

        // Complex formatting still works
        format!(
            "Complex: {} | {} | {}",
            "Success".success(),
            "Warning".warning(),
            "Error".error()
        )
        .normal()
        .println();
    });

    // Using light theme context
    light_theme.with_context(|| {
        "Inside light theme context - error".error().println();
        "Inside light theme context - success".success().println();
        "Inside light theme context - normal".normal().println();
    });

    println!("\n=== 4. Nested Context Example ===");

    // Show that contexts can be nested
    dark_theme.with_context(|| {
        "Outer context (dark theme)".heading1().println();

        light_theme.with_context(|| {
            "  Inner context (light theme)".heading2().println();
            "  This uses light theme".info().println();
        });

        "Back to outer context (dark theme)".heading2().println();
        "This uses dark theme again".info().println();
    });

    println!("\n=== 5. Mixed Usage Example ===");

    // You can mix both approaches
    dark_theme.with_context(|| {
        format!(
            "Status: {} | File: {} | {}",
            "OK".success(),                  // Uses context (dark theme)
            light_theme.code("config.toml"), // Direct light theme method
            "Complete".emphasis()            // Uses context (dark theme)
        )
        .normal()
        .println(); // Uses context (dark theme)
    });

    println!("\n=== 6. Practical Example: thag_palette_vs_theme Style ===");

    // This shows how you could rewrite thag_palette_vs_theme sections
    let guest_theme = &dark_theme;

    // Old verbose way
    println!("Old way:");
    format!(
        "🖥️  Detected: {}",
        "WezTerm".style_with(guest_theme.style_for(thag_styling::Role::Emphasis))
    )
    .style_with(guest_theme.style_for(thag_styling::Role::Normal))
    .println();

    // New direct method way
    println!("New direct way:");
    guest_theme
        .normal(&format!(
            "🖥️  Detected: {}",
            guest_theme.emphasis("WezTerm")
        ))
        .println();

    // New context way
    println!("New context way:");
    guest_theme.with_context(|| {
        format!("🖥️  Detected: {}", "WezTerm".emphasis())
            .normal()
            .println();
    });

    println!("\n=== 7. Performance Note ===");
    println!("The direct methods (theme.error()) are slightly more efficient");
    println!("because they don't require thread-local storage lookups.");
    println!("The context methods are more ergonomic for sections with lots of styling.");

    println!("\n✅ Theme context API test complete!");
    println!("\nSummary of new options:");
    println!("1. theme.role(text) - Direct, efficient, works everywhere");
    println!("2. theme.with_context(|| {{ text.role() }}) - Ergonomic for styling blocks");
    println!("3. Can mix both approaches as needed");
    println!("4. Guest theme styling is now as easy as active theme styling!");
}
