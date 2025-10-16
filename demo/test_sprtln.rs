/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Test script to verify sprtln macro works with both Style and Role
//# Purpose: Testing
//# Categories: macros, styling, technique, testing
use thag_styling::sprtln;
use thag_styling::styling::{Color, ColorInitStrategy, Role, Style, TermAttributes};

fn main() {
    // Initialize styling
    let strategy = ColorInitStrategy::determine();
    TermAttributes::get_or_init_with_strategy(&strategy);

    println!("Testing sprtln! macro with different input types:\n");

    // Test with Style directly (existing logic)
    sprtln!(
        Style::from(Role::Code),
        "This uses Style::from(Role::Code): {}",
        "hello world"
    );

    // Test with Role directly (new logic)
    sprtln!(
        Role::Code,
        "This uses Role::Code directly: {}",
        "hello world"
    );

    // Test with modified styles (existing logic should still work)
    sprtln!(
        Style::from(Role::Normal).bold(),
        "This uses Style::from(Role::Normal).bold(): {}",
        "hello world"
    );

    // Test with Color styles (existing logic should still work)
    sprtln!(
        &Color::yellow().bold(),
        "This uses &Color::yellow().bold(): {}",
        "hello world"
    );

    // Test with different roles
    sprtln!(Role::Error, "Error message: {}", "something went wrong");
    sprtln!(Role::Success, "Success message: {}", "operation completed");
    sprtln!(Role::Warning, "Warning message: {}", "please be careful");
    sprtln!(Role::Info, "Info message: {}", "just so you know");

    // Test with references to roles
    let role = Role::Emphasis;
    sprtln!(&role, "Using reference to role: {}", "emphasized text");
    sprtln!(role, "Using owned role: {}", "emphasized text");

    println!("\nAll tests completed successfully!");
}
