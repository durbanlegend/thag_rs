//! Simple test to verify Phase 2 implementation works

use thag_common::{ColorSupport, TermBgLuma, Verbosity, V};
use thag_styling::{Color, ColorInitStrategy, Role, Style, TermAttributes, Theme};

fn main() {
    println!("Testing Phase 2 implementation...");

    // Test thag_common functionality
    println!("Testing thag_common...");
    let verbosity = V::Normal;
    println!("Verbosity: {:?}", verbosity);

    let color_support = ColorSupport::Color256;
    println!("Color support: {:?}", color_support);

    let term_bg_luma = TermBgLuma::Dark;
    println!("Terminal background: {:?}", term_bg_luma);

    // Test thag_styling functionality
    println!("\nTesting thag_styling...");

    // Test basic color creation
    let red = Color::red();
    println!("Red color: {:?}", red);

    // Test style creation
    let style = Style::new().fg(red).bold();
    println!("Bold red style: {:?}", style);

    // Test style painting
    let painted = style.paint("Hello, World!");
    println!("Painted text: {}", painted);

    // Test role-based styling
    let error_text = Style::for_role(Role::Error).paint("Error message");
    println!("Error styled text: {}", error_text);

    // Test terminal attributes initialization
    let strategy = ColorInitStrategy::Default;
    let attrs = TermAttributes::get_or_init_with_strategy(&strategy);
    println!("Terminal attributes initialized successfully");
    println!("Color support: {:?}", attrs.color_support);
    println!("Background luminance: {:?}", attrs.term_bg_luma);

    // Test theme creation
    let bg_rgb = (32, 32, 32); // Dark background RGB
    let theme = Theme::auto_detect(ColorSupport::Color256, TermBgLuma::Dark, Some(&bg_rgb));
    match theme {
        Ok(theme) => {
            println!("Theme created: {}", theme.name);
            println!("Theme description: {}", theme.description);
        }
        Err(e) => {
            println!("Failed to create theme: {}", e);
        }
    }

    println!("\nPhase 2 test completed!");
}
