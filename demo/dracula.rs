/*[toml]
[dependencies]
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_detect", "config", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_detect", "config", "simplelog"] }
*/

/// Claude-generated demo of `Dracula` theme.
//# Purpose: Confirm style implementation and demo the theme.
//# Categories: basic, educational, testing
use thag_rs::styling::{Role, Theme};

fn main() {
    let theme = Theme::dracula();
    println!("\nDracula Theme Demo");
    println!("=================\n");

    // Structural
    println!("Structural Elements:");
    println!(
        "{}",
        theme
            .style_for(Role::Heading1)
            .paint("Main Heading (Bold Pink)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Heading2)
            .paint("Secondary Heading (Bold Purple)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Heading3)
            .paint("Tertiary Heading (Bold Cyan)")
    );
    println!();

    // Status/Alerts
    println!("Status and Alerts:");
    println!(
        "{}",
        theme
            .style_for(Role::Error)
            .paint("Error: Critical system failure (Red)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Warning)
            .paint("Warning: Approaching limit (Yellow)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Success)
            .paint("Success: Operation completed (Green)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Info)
            .paint("Info: System status normal (Cyan)")
    );
    println!();

    // Emphasis levels
    println!("Emphasis Levels:");
    println!(
        "{}",
        theme
            .style_for(Role::Emphasis)
            .paint("Emphasized text (Bold Purple)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Code)
            .paint("let x = 42; // Code snippet (Green)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Normal)
            .paint("Normal text (Light Gray)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Subtle)
            .paint("Background information (Medium Gray)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Hint)
            .paint("Type 'help' for more... (Italic Light Gray)")
    );
    println!();

    // Development
    println!("Development Information:");
    println!(
        "{}",
        theme
            .style_for(Role::Debug)
            .paint("Debug: Variable state = ready (Italic Medium Gray)")
    );
    println!(
        "{}",
        theme
            .style_for(Role::Trace)
            .paint("Trace: Entering main() (Dim Light Gray)")
    );
}
