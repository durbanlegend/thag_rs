/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["full"] }
*/

/// Debug demo to check specific theme loading and color values
///
/// This demonstrates:
/// 1. Explicit theme selection instead of auto-detection
/// 2. Color values for each role with specific theme
/// 3. RGB values and indices for the monet theme
/// 4. Comparison with auto-detected theme
//# Purpose: Debug specific theme loading vs auto-detection
//# Categories: debugging, styling, colors, themes
use thag_styling::{ColorInitStrategy, Role, Styleable, TermAttributes, Theme};

fn main() {
    println!("=== Specific Theme Debug Demo ===\n");

    println!("1. AUTO-DETECTED THEME:");
    // Initialize with auto-detection
    TermAttributes::initialize(&ColorInitStrategy::Match);
    let auto_attrs = TermAttributes::get_or_init();
    println!("   Theme: {}", auto_attrs.theme.name);
    print_role_colors("auto-detected", &auto_attrs.theme);

    println!("\n2. EXPLICIT THEME - thag-monet-woman-with-parasol-dark:");
    // Try to load the specific theme
    match Theme::get_builtin("thag-monet-woman-with-parasol-dark") {
        Ok(monet_theme) => {
            println!("   Theme: {}", monet_theme.name);
            print_role_colors("monet", &monet_theme);

            println!("\n3. COMPARISON:");
            compare_colors(&auto_attrs.theme, &monet_theme);
        }
        Err(e) => {
            println!("   Error loading monet theme: {}", e);
            println!("   Available themes:");
            // List available themes
            let themes = [
                "thag-dark",
                "thag-light",
                "thag-monet-woman-with-parasol-dark",
                "thag-monet-woman-with-parasol-light",
                "thag-morning-coffee-dark",
                "thag-morning-coffee-light",
            ];
            for theme_name in themes {
                match Theme::get_builtin(theme_name) {
                    Ok(_) => println!("     ✓ {}", theme_name),
                    Err(_) => println!("     ✗ {}", theme_name),
                }
            }
        }
    }

    println!("\n4. VISUAL TEST WITH EXPLICIT THEME:");
    if let Ok(monet_theme) = Theme::get_builtin("thag-monet-woman-with-parasol-dark") {
        // Test with explicit theme colors
        println!("   Using monet theme colors directly:");
        let roles = [
            ("Error", Role::Error),
            ("Warning", Role::Warning),
            ("Success", Role::Success),
            ("Info", Role::Info),
            ("Heading1", Role::Heading1),
            ("Heading2", Role::Heading2),
        ];

        for (name, role) in &roles {
            let style = monet_theme.style_for(*role);
            if let Some(color_info) = &style.foreground {
                let styled_text = style.paint(format!("{} message", name));
                println!("     {}", styled_text);
            }
        }
    }

    println!("\n5. CURRENT STYLEABLE METHODS (should use auto-detected):");
    println!("   {}", "Error message".error());
    println!("   {}", "Warning message".warning());
    println!("   {}", "Success message".success());
    println!("   {}", "Info message".info());
    println!("   {}", "Heading1 text".heading1());
    println!("   {}", "Heading2 text".heading2());
}

fn print_role_colors(label: &str, theme: &Theme) {
    let roles = [
        ("Error", Role::Error),
        ("Warning", Role::Warning),
        ("Success", Role::Success),
        ("Info", Role::Info),
        ("Heading1", Role::Heading1),
        ("Heading2", Role::Heading2),
        ("Heading3", Role::Heading3),
    ];

    for (name, role) in &roles {
        let style = theme.style_for(*role);
        if let Some(color_info) = &style.foreground {
            if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
                println!(
                    "     {:>10} [{}]: RGB({:3},{:3},{:3})",
                    name, label, rgb[0], rgb[1], rgb[2]
                );
            }
        }
    }
}

fn compare_colors(auto_theme: &Theme, monet_theme: &Theme) {
    println!("   Color differences:");
    let roles = [("Error", Role::Error), ("Heading1", Role::Heading1)];

    for (name, role) in &roles {
        let auto_style = auto_theme.style_for(*role);
        let monet_style = monet_theme.style_for(*role);

        let auto_rgb = extract_rgb(&auto_style);
        let monet_rgb = extract_rgb(&monet_style);

        println!("     {}:", name);
        println!("       Auto:  {:?}", auto_rgb);
        println!("       Monet: {:?}", monet_rgb);

        if auto_rgb == monet_rgb {
            println!("       ❌ SAME - This explains the visual similarity!");
        } else {
            println!("       ✅ Different");
        }
    }
}

fn extract_rgb(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    if let Some(color_info) = &style.foreground {
        if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
            Some((rgb[0], rgb[1], rgb[2]))
        } else {
            None
        }
    } else {
        None
    }
}
