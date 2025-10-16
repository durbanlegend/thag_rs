/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/
/// TermAttributes context pattern demo.
//# Purpose: Demonstrate TermAttributes context pattern for testing and temporary overrides
//# Categories: styling, terminal, testing
use thag_styling::{ColorSupport, Style, TermAttributes, TermBgLuma, Theme};

fn main() {
    println!("=== TermAttributes Context Pattern Demo ===\n");

    // 1. Normal global initialization
    println!("1. Global TermAttributes (auto-detected):");
    let global_attrs = TermAttributes::get_or_init();
    println!("   Color Support: {:?}", global_attrs.color_support);
    println!("   Theme: {}", global_attrs.theme.name);
    println!("   Background Luma: {:?}", global_attrs.term_bg_luma);
    println!("   How Initialized: {:?}", global_attrs.how_initialized);
    println!("   Init Strategy: {:?}", global_attrs.init_strategy);

    // Show a styled message with global context
    println!(
        "   Styled message: {}",
        Style::for_role(thag_styling::Role::Success).paint("Global context works!")
    );
    println!();

    // 2. Create a custom context for testing
    println!("2. Creating custom context for testing:");
    let custom_theme = Theme::get_builtin("basic_dark").unwrap();
    let custom_attrs = TermAttributes::for_testing(
        ColorSupport::Basic,
        Some([64, 64, 64]),
        TermBgLuma::Dark,
        custom_theme,
    );

    println!("   Custom Color Support: {:?}", custom_attrs.color_support);
    println!("   Custom Theme: {}", custom_attrs.theme.name);
    println!("   Custom Background RGB: {:?}", custom_attrs.term_bg_rgb);

    // 3. Use the custom context
    println!("\n3. Running code within custom context:");
    custom_attrs.with_context(|| {
        let context_attrs = TermAttributes::current();
        println!(
            "   Inside context - Color Support: {:?}",
            context_attrs.color_support
        );
        println!("   Inside context - Theme: {}", context_attrs.theme.name);
        println!(
            "   Inside context - Background RGB: {:?}",
            context_attrs.term_bg_rgb
        );

        // Styling within context uses the custom attributes
        println!(
            "   Styled message: {}",
            Style::for_role(thag_styling::Role::Error).paint("Context-specific styling!")
        );

        // Nested context test
        println!("\n   3a. Nested context test:");
        let nested_theme = Theme::get_builtin("solarized-dark").unwrap();
        let nested_attrs = TermAttributes::for_testing(
            ColorSupport::TrueColor,
            Some([248, 248, 248]),
            TermBgLuma::Light,
            nested_theme,
        );

        nested_attrs.with_context(|| {
            let nested_context = TermAttributes::current();
            println!(
                "      Nested context - Color Support: {:?}",
                nested_context.color_support
            );
            println!(
                "      Nested context - Theme: {}",
                nested_context.theme.name
            );
            println!(
                "      Nested context - Background Luma: {:?}",
                nested_context.term_bg_luma
            );
            println!(
                "      Styled message: {}",
                Style::for_role(thag_styling::Role::Info).paint("Nested context!")
            );
        });

        println!(
            "   Back to outer context - Theme: {}",
            TermAttributes::current().theme.name
        );
    });

    // 4. Verify global context is restored
    println!("\n4. Back to global context:");
    let restored_attrs = TermAttributes::current();
    println!("   Color Support: {:?}", restored_attrs.color_support);
    println!("   Theme: {}", restored_attrs.theme.name);
    println!("   Background Luma: {:?}", restored_attrs.term_bg_luma);
    println!(
        "   Styled message: {}",
        Style::for_role(thag_styling::Role::Success).paint("Global context restored!")
    );

    println!("\n=== Context Demo Complete ===");
}
