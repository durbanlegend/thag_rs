/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

//! Demo script showcasing the color distance optimization for inquire prompts
//!
//! This script demonstrates how the enhanced RenderConfig uses color distance
//! calculations to optimize contrast between selected and normal list options.
//! It works with different themes and terminal capabilities.

use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet};
use inquire::Select;
use thag_styling::{ColorValue, Role, TermAttributes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Color Distance Optimization Demo");
    println!("═══════════════════════════════════════");
    println!();

    // Show current theme analysis
    show_color_analysis();

    // Apply the optimized render config
    inquire::set_global_render_config(get_optimized_render_config());

    // Demo with different types of selections
    demo_basic_selection()?;
    demo_theme_roles()?;
    demo_long_list()?;

    println!("\n✨ Demo complete! The color optimization ensures maximum contrast");
    println!("   between selected and normal list entries across different themes.");

    Ok(())
}

fn show_color_analysis() {
    let term_attrs = TermAttributes::get_or_init();
    let theme = &term_attrs.theme;

    println!("📊 Current Theme Analysis:");
    println!("─────────────────────────");

    let normal_rgb = get_rgb_from_role(Role::Normal, theme);
    // let candidates = [Role::Emphasis, Role::Code, Role::Heading3];
    let candidates = [Role::Emphasis, Role::Heading3, Role::Info, Role::Success];

    if let Some(normal_color) = normal_rgb {
        println!("Normal role (regular entries): RGB {:?}", normal_color);
        println!();

        let mut best_distance = 0.0;
        let mut best_role = Role::Code;

        for &role in &candidates {
            if let Some(role_rgb) = get_rgb_from_role(role, theme) {
                let distance = color_distance(normal_color, role_rgb);
                println!(
                    "{:12?}: RGB {:?} → Distance: {:.1}",
                    role, role_rgb, distance
                );

                if distance > best_distance {
                    best_distance = distance;
                    best_role = role;
                }
            }
        }

        println!();
        println!(
            "🎯 Optimal choice: {:?} (distance: {:.1})",
            best_role, best_distance
        );
    } else {
        println!("⚠️  Could not extract RGB from Normal role - using fallback");
    }

    println!();
}

fn demo_basic_selection() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Basic Selection Demo:");
    println!("Notice the contrast between the highlighted option and normal entries");
    println!();

    let options = vec![
        "Regular list entry #1",
        "Regular list entry #2",
        "Regular list entry #3",
        "Regular list entry #4",
    ];

    let _choice = Select::new("Choose an option:", options)
        .with_help_message("↑↓ to navigate - observe the color contrast optimization")
        .prompt()?;

    println!();
    Ok(())
}

fn demo_theme_roles() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Theme Roles Demo:");
    println!("Showing how different role types would appear in lists");
    println!();

    let role_options = vec![
        "Emphasis - emphasized text role",
        "Heading3 - tertiary heading role",
        "Info - informational text role",
        "Normal - standard text role",
        "Success - success text role",
        // "Code - code snippet role",
    ];

    let _choice = Select::new("Select a role type:", role_options)
        .with_help_message("Each role has different colors - selected option uses optimal contrast")
        .prompt()?;

    println!();
    Ok(())
}

fn demo_long_list() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Long List Demo:");
    println!("Testing contrast optimization with many options");
    println!();

    let long_options: Vec<String> = (1..=20)
        .map(|i| format!("List item {} - normal entry color", i))
        .collect();

    let _choice = Select::new("Navigate through this long list:", long_options)
        .with_help_message("Scroll through options - contrast remains optimal throughout")
        .with_page_size(8)
        .prompt()?;

    println!();
    Ok(())
}

/// Creates an optimized RenderConfig using color distance calculations
fn get_optimized_render_config() -> RenderConfig<'static> {
    let mut render_config = RenderConfig::default();
    let term_attrs = TermAttributes::get_or_init();
    let theme = &term_attrs.theme;

    // Helper to convert thag roles to inquire colors
    let convert_color = |role: Role| -> Color {
        let style = theme.style_for(role);
        if let Some(color_info) = &style.foreground {
            match &color_info.value {
                ColorValue::TrueColor { rgb } => Color::Rgb {
                    r: rgb[0],
                    g: rgb[1],
                    b: rgb[2],
                },
                ColorValue::Color256 { color256 } => Color::AnsiValue(*color256),
                ColorValue::Basic { .. } => Color::AnsiValue(u8::from(&role)),
            }
        } else {
            Color::AnsiValue(u8::from(&role))
        }
    };

    // Color distance optimization for selected_option
    let normal_rgb = get_rgb_from_role(Role::Normal, theme);
    let candidate_roles = [Role::Emphasis, Role::Heading3, Role::Info, Role::Success];

    let optimal_role = if let Some(normal_color) = normal_rgb {
        candidate_roles
            .iter()
            .filter_map(|&role| {
                get_rgb_from_role(role, theme).map(|rgb| (role, color_distance(normal_color, rgb)))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(role, _)| role)
            .unwrap_or(Role::Code)
    } else {
        Role::Code // Fallback
    };

    // Apply optimized configuration
    render_config.selected_option = Some(
        StyleSheet::new()
            .with_fg(convert_color(optimal_role))
            .with_attr(Attributes::BOLD),
    );

    // Regular options use Normal role for consistent baseline
    render_config.option = StyleSheet::empty().with_fg(convert_color(Role::Subtle));

    // Other UI elements
    render_config.help_message = StyleSheet::empty().with_fg(convert_color(Role::Info));
    render_config.error_message = inquire::ui::ErrorMessageRenderConfig::default_colored()
        .with_message(StyleSheet::empty().with_fg(convert_color(Role::Error)));
    render_config.prompt = StyleSheet::empty().with_fg(convert_color(Role::Normal));
    render_config.answer = StyleSheet::empty().with_fg(convert_color(Role::Success));
    render_config.placeholder = StyleSheet::empty().with_fg(convert_color(Role::Subtle));

    render_config
}

/// Extract RGB values from a thag role for color distance calculations
fn get_rgb_from_role(role: Role, theme: &thag_styling::Theme) -> Option<(u8, u8, u8)> {
    let style = theme.style_for(role);
    if let Some(color_info) = &style.foreground {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            ColorValue::Color256 { color256 } => {
                // Convert 256-color index to approximate RGB
                let index = *color256 as usize;
                if index < 16 {
                    // Standard 16 colors
                    let standard_colors = [
                        (0, 0, 0),
                        (128, 0, 0),
                        (0, 128, 0),
                        (128, 128, 0),
                        (0, 0, 128),
                        (128, 0, 128),
                        (0, 128, 128),
                        (192, 192, 192),
                        (128, 128, 128),
                        (255, 0, 0),
                        (0, 255, 0),
                        (255, 255, 0),
                        (0, 0, 255),
                        (255, 0, 255),
                        (0, 255, 255),
                        (255, 255, 255),
                    ];
                    standard_colors.get(index).copied()
                } else if index < 232 {
                    // 216-color cube (6x6x6)
                    let n = index - 16;
                    let r = (n / 36) * 51;
                    let g = ((n % 36) / 6) * 51;
                    let b = (n % 6) * 51;
                    Some((r as u8, g as u8, b as u8))
                } else {
                    // 24 grayscale colors
                    let gray = 8 + (index - 232) * 10;
                    Some((gray as u8, gray as u8, gray as u8))
                }
            }
            ColorValue::Basic { .. } => {
                // Approximate RGB values for basic color roles
                match role {
                    Role::Error => Some([255, 0, 0]),
                    Role::Success => Some([0, 255, 0]),
                    Role::Warning => Some([255, 255, 0]),
                    Role::Info => Some([0, 255, 255]),
                    Role::Code => Some([255, 0, 255]),
                    Role::Emphasis => Some([255, 128, 0]),
                    Role::Heading3 => Some([128, 255, 128]),
                    _ => Some([192, 192, 192]),
                }
            }
        }
    } else {
        None
    }
}

/// Calculate Euclidean distance between two RGB colors
fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
    let dr = (f32::from(c1.0) - f32::from(c2.0)).powi(2);
    let dg = (f32::from(c1.1) - f32::from(c2.1)).powi(2);
    let db = (f32::from(c1.2) - f32::from(c2.2)).powi(2);
    (dr + dg + db).sqrt()
}
