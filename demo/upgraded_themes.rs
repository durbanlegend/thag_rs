/*[toml]
[dependencies]
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_detect", "config", "simplelog"] }
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_detect", "config", "simplelog"] }
*/

/// Claude-generated demo of styling's legacy themes upgraded to the new structure.
//# Purpose: Confirm style conversion and demo the themes.
//# Categories: basic, educational, testing
use thag_rs::styling::{Role, Theme};

fn display_theme(theme: &Theme) {
    println!("\n{} Theme", theme.config().description);
    println!("{}", "=".repeat(theme.config().description.len() + 6));

    // Structural
    println!("\nStructural:");
    print!("  ");
    println!("{}", theme.style_for(Role::Heading1).paint("Heading 1"));
    print!("  ");
    println!("{}", theme.style_for(Role::Heading2).paint("Heading 2"));
    print!("  ");
    println!("{}", theme.style_for(Role::Heading3).paint("Heading 3"));

    // Status/Alerts
    println!("\nStatus/Alerts:");
    print!("  ");
    println!(
        "{}",
        theme.style_for(Role::Error).paint("Error: Critical issue")
    );
    print!("  ");
    println!(
        "{}",
        theme.style_for(Role::Warning).paint("Warning: Take care")
    );
    print!("  ");
    println!(
        "{}",
        theme
            .style_for(Role::Success)
            .paint("Success: Task completed")
    );
    print!("  ");
    println!(
        "{}",
        theme.style_for(Role::Info).paint("Info: Notable point")
    );

    // Emphasis levels
    println!("\nEmphasis Levels:");
    print!("  ");
    println!(
        "{}",
        theme.style_for(Role::Emphasis).paint("Emphasized text")
    );
    print!("  ");
    println!("{}", theme.style_for(Role::Code).paint("let x = 42;"));
    print!("  ");
    println!("{}", theme.style_for(Role::Normal).paint("Normal text"));
    print!("  ");
    println!(
        "{}",
        theme
            .style_for(Role::Subtle)
            .paint("Subtle background info")
    );
    print!("  ");
    println!(
        "{}",
        theme.style_for(Role::Hint).paint("Type 'help' for more...")
    );

    // Development
    println!("\nDevelopment:");
    print!("  ");
    println!(
        "{}",
        theme.style_for(Role::Debug).paint("Debug: Variable state")
    );
    print!("  ");
    println!(
        "{}",
        theme.style_for(Role::Trace).paint("Trace: Function entry")
    );
    println!();
}

fn main() {
    let themes = [
        Theme::basic_light(),
        Theme::basic_dark(),
        Theme::full_light(),
        Theme::full_dark(),
    ];

    for theme in themes {
        display_theme(&theme);
    }
}
