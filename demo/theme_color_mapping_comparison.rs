/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto" }
*/

/// Theme Color Mapping Comparison Tool
///
/// This tool shows exactly how the source thag-vibrant-dark theme colors
/// map to the exported Alacritty format, helping debug color differences.
//# Purpose: Test color mapping.
//# Categories: color, testing, theming
use thag_styling::{ExportFormat, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Theme Color Mapping Comparison");
    println!("==================================\n");

    // Load the source theme
    let theme = Theme::get_builtin("thag-vibrant-dark")?;

    println!("📋 Source Theme: {}", theme.name);
    println!("📝 Description: {}", theme.description);
    let first_bg_rgb = theme.bg_rgbs.first().unwrap_or(&[0, 0, 0]);
    let [r, g, b] = first_bg_rgb;
    println!("🖼️  Background: {first_bg_rgb:?} = #{r:02x}{g:02x}{b:02x}\n");

    // Show source semantic colors
    println!("🎯 Source thag-vibrant-dark Semantic Colors:");
    println!("==============================================");
    display_source_colors(&theme);

    println!("\n🔄 Mapping to Alacritty ANSI Colors:");
    println!("=====================================");
    display_mapping_logic(&theme);

    println!("\n📄 Generated Alacritty Theme Preview:");
    println!("======================================");
    let alacritty_content = ExportFormat::Alacritty.export_theme(&theme)?;

    // Show the colors section of the generated file
    let lines: Vec<&str> = alacritty_content.lines().collect();
    let mut in_colors = false;
    let mut line_count = 0;

    for line in lines {
        if line.contains("[colors]") {
            in_colors = true;
        }

        if in_colors {
            println!("{}", line);
            line_count += 1;
            if line_count > 50 {
                // Limit output
                break;
            }
        }
    }

    println!("\n💡 Color Mapping Explanation:");
    println!("==============================");
    println!("The differences you see are due to semantic → ANSI color mapping:");
    println!("• Normal text (light green #90e090) → both 'green' and 'white' ANSI slots");
    println!("• Error/Warning (peachy #f0d0b0) → 'red' and 'yellow' ANSI slots");
    println!("• Info (blue #2080b0) → 'blue' and 'cyan' ANSI slots");
    println!("• Code (blue-violet #6f8cc5) → 'magenta' ANSI slot");
    println!("• Background (#202020) → 'black' ANSI slot");
    println!("• Subtle (yellow-green #caca92) → 'bright black' ANSI slot");

    println!("\n🔧 To verify colors match, check if:");
    println!("• Your terminal background is #202020 (dark gray)");
    println!("• Normal text appears as #90e090 (light green)");
    println!("• Error messages appear as #f0d0b0 (peachy/tan)");

    Ok(())
}

/// Display the source theme's semantic colors with RGB values
fn display_source_colors(theme: &Theme) {
    let semantic_colors = [
        ("Normal", &theme.palette.normal),
        ("Error", &theme.palette.error),
        ("Warning", &theme.palette.warning),
        ("Success", &theme.palette.success),
        ("Info", &theme.palette.info),
        ("Code", &theme.palette.code),
        ("Emphasis", &theme.palette.emphasis),
        ("Subtle", &theme.palette.subtle),
        ("Heading1", &theme.palette.heading1),
        ("Heading2", &theme.palette.heading2),
        ("Heading3", &theme.palette.heading3),
    ];

    for (name, style) in semantic_colors {
        let colored_text = style.paint(format!("{:>12}", name));
        let rgb_info = extract_rgb_info(style);
        println!("   {} - {}", colored_text, rgb_info);
    }
}

/// Show the mapping logic from semantic to ANSI colors
fn display_mapping_logic(theme: &Theme) {
    let mappings = [
        (
            "ANSI Black (0)",
            "Background",
            theme.bg_rgbs.first().copied(),
        ),
        ("ANSI Red (1)", "Error", extract_rgb(&theme.palette.error)),
        (
            "ANSI Green (2)",
            "Success",
            extract_rgb(&theme.palette.success),
        ),
        (
            "ANSI Yellow (3)",
            "Warning",
            extract_rgb(&theme.palette.warning),
        ),
        ("ANSI Blue (4)", "Info", extract_rgb(&theme.palette.info)),
        ("ANSI Magenta (5)", "Code", extract_rgb(&theme.palette.code)),
        (
            "ANSI Cyan (6)",
            "Info (reused)",
            extract_rgb(&theme.palette.info),
        ),
        (
            "ANSI White (7)",
            "Normal",
            extract_rgb(&theme.palette.normal),
        ),
        (
            "ANSI Bright Black (8)",
            "Subtle",
            extract_rgb(&theme.palette.subtle),
        ),
        (
            "ANSI Bright White (15)",
            "Emphasis",
            extract_rgb(&theme.palette.emphasis),
        ),
    ];

    for (ansi_slot, semantic_role, rgb_opt) in mappings {
        if let Some([r, g, b]) = rgb_opt {
            let color_sample = format!("\x1b[38;2;{};{};{}m██████\x1b[0m", r, g, b);
            println!(
                "   {:20} ← {:12} {} #{:02x}{:02x}{:02x}",
                ansi_slot, semantic_role, color_sample, r, g, b
            );
        }
    }
}

/// Extract RGB values from a style
fn extract_rgb(style: &thag_styling::Style) -> Option<[u8; 3]> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some(rgb),
            _ => None,
        })
        .copied()
}

/// Extract RGB information from a style for display
fn extract_rgb_info(style: &thag_styling::Style) -> String {
    match &style.foreground {
        Some(color_info) => match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => {
                format!(
                    "RGB({:3}, {:3}, {:3}) = #{:02x}{:02x}{:02x}",
                    rgb[0], rgb[1], rgb[2], rgb[0], rgb[1], rgb[2]
                )
            }
            thag_styling::ColorValue::Color256 { color256 } => {
                format!("256-Color({})", color256)
            }
            thag_styling::ColorValue::Basic { index, .. } => {
                format!("Basic({})", index)
            }
        },
        None => "No color".to_string(),
    }
}
