/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["owo_colors_support"] }
owo-colors = "4.1"
*/

/// Demo and test script for owo-colors integration with `thag_styling`.
///
//# Purpose: Demonstrate and test the owo-colors integration with `thag`'s theming system.
//# Categories: styling, colors, terminal, integration, demo
use owo_colors::{DynColors, OwoColorize, Style as OwoStyle};
use thag_styling::{
    integrations::{
        owo_colors_integration::{helpers, OwoColorsStyleExt},
        ThemedStyle,
    },
    Role, Style,
};

fn main() {
    println!("🎨 owo-colors Integration Demo\n");

    // Test basic themed styling
    println!("=== Basic Themed Styling ===");
    test_themed_styles();
    println!();

    // Test color conversions
    println!("=== Color Conversions ===");
    test_color_conversions();
    println!();

    // Test style extensions
    println!("=== Style Extensions ===");
    test_style_extensions();
    println!();

    // Test practical examples
    println!("=== Practical Examples ===");
    test_practical_examples();
    println!();

    // Test all roles
    println!("=== All Role Styles ===");
    test_all_roles();
    println!();

    println!("✅ All tests completed successfully!");
}

fn test_themed_styles() {
    println!("Creating themed styles from roles:");

    // Test ThemedStyle trait implementation
    let error_style = OwoStyle::themed(Role::Error);
    let success_style = OwoStyle::themed(Role::Success);
    let warning_style = OwoStyle::themed(Role::Warning);
    let info_style = OwoStyle::themed(Role::Info);

    println!(
        "  Error:   {}",
        "This is an error message".style(error_style)
    );
    println!(
        "  Success: {}",
        "This is a success message".style(success_style)
    );
    println!(
        "  Warning: {}",
        "This is a warning message".style(warning_style)
    );
    println!("  Info:    {}", "This is an info message".style(info_style));

    // Test colored text directly
    let error_color = DynColors::themed(Role::Error);
    let success_color = DynColors::themed(Role::Success);

    println!(
        "  Colored: {} and {}",
        "Error".color(error_color),
        "Success".color(success_color)
    );
}

fn test_color_conversions() {
    println!("Testing From trait implementations:");

    // Test Role -> OwoColor conversion
    let roles = [
        Role::Error,
        Role::Success,
        Role::Warning,
        Role::Info,
        Role::Normal,
    ];

    for role in &roles {
        let color: DynColors = role.into();
        println!("  {:?} -> {}", role, "Sample text".color(color));
    }

    // Test Role -> OwoStyle conversion
    println!("\nTesting Role -> OwoStyle conversions:");
    for role in &roles {
        let style: OwoStyle = role.into();
        println!("  {:?} -> {}", role, "Styled text".style(style));
    }
}

fn test_style_extensions() {
    println!("Testing OwoColorsStyleExt trait:");

    // Create a base style with some attributes
    let base_style = OwoStyle::new().bold().underline();

    // Extend it with theme-aware colors
    let error_themed = base_style.with_role(Role::Error);
    let success_themed = base_style.with_role(Role::Success);

    println!(
        "  Base + Error:   {}",
        "Bold underlined error".style(error_themed)
    );
    println!(
        "  Base + Success: {}",
        "Bold underlined success".style(success_themed)
    );

    // Test with_thag_style method
    let custom_style = Style::from(Role::Warning);
    let extended_style = base_style.with_thag_style(&custom_style);

    println!(
        "  Base + Custom:  {}",
        "Bold underlined custom".style(extended_style)
    );
}

fn test_practical_examples() {
    println!("Practical usage examples:");

    // Simulate a CLI application output
    simulate_cli_output();
    println!();

    // Simulate log messages
    simulate_log_messages();
    println!();

    // Simulate status indicators
    simulate_status_indicators();
}

fn simulate_cli_output() {
    println!("📋 CLI Application Output:");

    let header_style = helpers::emphasis_style();
    let normal_style = OwoStyle::themed(Role::Normal);
    let subtle_style = helpers::subtle_style();
    let success_style = helpers::success_style();
    let error_style = helpers::error_style();

    println!("  {}", "Project Analysis Results".style(header_style));
    println!("  {}", "─".repeat(25).style(subtle_style));

    println!(
        "  {} {}",
        "Files processed:".style(normal_style),
        "42".style(success_style)
    );
    println!(
        "  {} {}",
        "Warnings found: ".style(normal_style),
        "3".style((&Role::Warning).into())
    );
    println!(
        "  {} {}",
        "Errors found:   ".style(normal_style),
        "1".style(error_style)
    );

    println!("  {}", "Analysis complete!".style(success_style));
}

fn simulate_log_messages() {
    println!("📝 Log Messages:");

    let timestamp_style = OwoStyle::themed(Role::Subtle);
    let level_styles = [
        ("DEBUG", OwoStyle::themed(Role::Subtle)),
        ("INFO ", OwoStyle::themed(Role::Info)),
        ("WARN ", OwoStyle::themed(Role::Warning)),
        ("ERROR", OwoStyle::themed(Role::Error)),
    ];

    for (level, style) in &level_styles {
        println!(
            "  {} [{}] Sample {} message",
            "2024-01-15 10:30:45".style(timestamp_style),
            level.style(*style),
            level.to_lowercase()
        );
    }
}

fn simulate_status_indicators() {
    println!("📊 Status Indicators:");

    let statuses = [
        ("✓", "Service running", Role::Success),
        ("⚠", "Service degraded", Role::Warning),
        ("✗", "Service failed", Role::Error),
        ("ℹ", "Service info", Role::Info),
        ("⏸", "Service paused", Role::Subtle),
    ];

    for (icon, message, role) in &statuses {
        let icon_style = OwoStyle::themed(*role);
        let message_style = OwoStyle::themed(Role::Normal);

        println!(
            "  {} {}",
            icon.style(icon_style),
            message.style(message_style)
        );
    }
}

fn test_all_roles() {
    println!("Testing all available roles:");

    let all_roles = [
        Role::Normal,
        Role::Emphasis,
        Role::Subtle,
        Role::Error,
        Role::Warning,
        Role::Success,
        Role::Info,
        Role::Debug,
        Role::Link,
        Role::Code,
        Role::Quote,
        Role::Hint,
        Role::Commentary,
        Role::Heading1,
        Role::Heading2,
        Role::Heading3,
    ];

    for role in &all_roles {
        let style = OwoStyle::themed(*role);
        let color = DynColors::themed(*role);

        println!(
            "  {:>10}: {} | {}",
            format!("{:?}", role),
            "styled".style(style),
            "colored".color(color)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_basic() {
        // Test that basic integration works
        let _style = OwoStyle::themed(Role::Error);
        let _color = DynColors::themed(Role::Success);
    }

    #[test]
    fn test_from_conversions() {
        let role = Role::Warning;

        // Test From trait implementations
        let _color: DynColors = (&role).into();
        let _style: OwoStyle = (&role).into();

        // Test direct conversion
        let _color2 = DynColors::from(&role);
        let _style2 = OwoStyle::from(&role);
    }

    #[test]
    fn test_style_extensions() {
        let base_style = OwoStyle::new().bold();

        // Test extension methods
        let _extended1 = base_style.with_role(Role::Error);
        let _extended2 = base_style.with_thag_style(&Style::from(Role::Success));
    }

    #[test]
    fn test_all_roles_convert() {
        // Ensure all roles can be converted without panicking
        let roles = [
            Role::Normal,
            Role::Emphasis,
            Role::Subtle,
            Role::Error,
            Role::Warning,
            Role::Success,
            Role::Info,
            Role::Debug,
            Role::Link,
            Role::Code,
            Role::Quote,
            Role::Hint,
            Role::Commentary,
            Role::Heading1,
            Role::Heading2,
            Role::Heading3,
        ];

        for role in &roles {
            let _style = OwoStyle::themed(*role);
            let _color = DynColors::themed(*role);
        }
    }
}
