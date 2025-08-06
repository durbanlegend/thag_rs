/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect", "config"] }
*/

/// Test script to verify cprtln macro works with both Style and Role
//# Purpose: Testing
//# Categories: macros, styling, technique, testing
use thag_styling::cprtln;
use thag_styling::styling::{Color, ColorInitStrategy, Role, Style, TermAttributes};

fn main() {
    // Initialize styling
    let strategy = ColorInitStrategy::determine();
    TermAttributes::initialize(strategy);

    println!("Testing cprtln! macro with different input types:\n");

    // Test with Style directly (existing functionality)
    cprtln!(
        Style::from(Role::Code),
        "This uses Style::from(Role::Code): {}",
        "hello world"
    );

    // Test with Role directly (new functionality)
    cprtln!(
        Role::Code,
        "This uses Role::Code directly: {}",
        "hello world"
    );

    // Test with modified styles (existing functionality should still work)
    cprtln!(
        Style::from(Role::Normal).bold(),
        "This uses Style::from(Role::Normal).bold(): {}",
        "hello world"
    );

    // Test with Color styles (existing functionality should still work)
    cprtln!(
        &Color::yellow().bold(),
        "This uses &Color::yellow().bold(): {}",
        "hello world"
    );

    // Test with different roles
    cprtln!(Role::Error, "Error message: {}", "something went wrong");
    cprtln!(Role::Success, "Success message: {}", "operation completed");
    cprtln!(Role::Warning, "Warning message: {}", "please be careful");
    cprtln!(Role::Info, "Info message: {}", "just so you know");

    // Test with references to roles
    let role = Role::Emphasis;
    cprtln!(&role, "Using reference to role: {}", "emphasized text");
    cprtln!(role, "Using owned role: {}", "emphasized text");

    println!("\nAll tests completed successfully!");
}
