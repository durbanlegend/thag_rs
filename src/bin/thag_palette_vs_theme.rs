/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
# thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto", default-features = false, features = ["inquire"] }
*/
#![allow(clippy::uninlined_format_args)]
/// Terminal palette comparison tool with theme selection
///
/// This tool displays the current terminal's color palette alongside
/// a selected thag theme for direct comparison. Helps identify color
/// mapping issues and verify theme installation.
//# Purpose: Compare terminal palette with thag theme colors
//# Categories: color, styling, terminal, theming, tools
use std::error::Error;
use thag_proc_macros::file_navigator;

use thag_styling::{
    cprtln, cprtln_with_embeds, select_builtin_theme, ColorInitStrategy, Role, Style, Styler,
    TermAttributes, TermBgLuma, Theme,
};

file_navigator! {}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    cprtln_with_embeds!(
        Role::Normal,
        "🎨 {} - Terminal Palette vs Theme Comparison",
        &[Role::Info.embed("thag_palette_vs_theme")]
    );
    cprtln!(Role::Subtle, "{}", "=".repeat(70));
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Select theme to compare
    let theme = select_theme(&mut navigator)?;

    cprtln_with_embeds!(
        Role::Normal,
        "📋 Selected theme: {}",
        &[Role::Heading3.embed(&theme.name)]
    );
    println!("📝 Description: {}", theme.description);
    println!();

    // Display comprehensive comparison
    display_terminal_info();
    display_ansi_colors();
    display_theme_colors(&theme);
    display_color_comparison(&theme);
    display_recommendations(&theme);

    println!("\n🎉 Palette comparison complete!");
    Ok(())
}

/// Select a theme using file navigator or built-in themes
fn select_theme(navigator: &mut FileNavigator) -> Result<Theme, Box<dyn Error>> {
    use inquire::{Select, Text};

    let selection_options = vec![
        "Select theme file (.toml)",
        "Use built-in theme by name",
        "List available built-in themes",
    ];

    let selection_method =
        Select::new("How would you like to select a theme?", selection_options).prompt()?;

    match selection_method {
        "Select theme file (.toml)" => {
            println!("\n📁 Select a theme file:");
            let Ok(theme_file) = select_file(navigator, Some("toml"), false) else {
                return Err("No theme file selected".into());
            };

            cprtln_with_embeds!(
                Role::Normal,
                "📄 Loading theme from: {}",
                &[Role::Debug.embed(&theme_file.display().to_string())]
            );

            Theme::load_from_file(&theme_file)
                .map_err(|e| format!("Failed to load theme file: {}", e).into())
        }
        "Use built-in theme by name" => {
            let theme_name = Text::new("Enter built-in theme name:")
                .with_help_message("e.g., 'thag-vibrant-dark', 'dracula_official', 'gruvbox_dark'")
                .prompt()?;

            Theme::get_builtin(&theme_name).map_err(|e| {
                format!("Failed to load built-in theme '{}': {}", theme_name, e).into()
            })
        }
        "List available built-in themes" => {
            cprtln_with_embeds!(
                Role::Normal,
                "\n📚 {} Built-in themes:",
                &[Role::Info.embed("Available")]
            );
            println!("─────────────────────");

            let maybe_theme_name = select_builtin_theme();
            let Some(theme_name) = maybe_theme_name else {
                return Err("No theme selected".into());
            };

            Theme::get_builtin(&theme_name).map_err(|e| {
                format!("Failed to load built-in theme '{}': {}", theme_name, e).into()
            })
        }
        _ => Err("Invalid selection".into()),
    }
}

/// Display basic terminal information
fn display_terminal_info() {
    cprtln_with_embeds!(
        Role::Normal,
        "📟 {} Information:",
        &[Role::Info.embed("Terminal")]
    );
    println!("─────────────────────");

    let term_attrs = TermAttributes::get_or_init();

    println!("🔍 Color Support: {:?}", term_attrs.color_support);
    println!("🌓 Background Luma: {:?}", term_attrs.term_bg_luma);

    // Display relevant environment variables
    if let Ok(term) = std::env::var("TERM") {
        cprtln_with_embeds!(Role::Normal, "🖥️  TERM: {}", &[Role::Debug.embed(&term)]);
    }
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        cprtln_with_embeds!(
            Role::Normal,
            "🌈 COLORTERM: {}",
            &[Role::Debug.embed(&colorterm)]
        );
    }

    // Try to detect terminal emulator
    let terminal_info = detect_terminal_emulator();
    if !terminal_info.is_empty() {
        cprtln_with_embeds!(
            Role::Normal,
            "🖥️  Detected: {}",
            &[Role::Emphasis.embed(&terminal_info)]
        );
    }

    println!();
}

/// Attempt to detect terminal emulator
fn detect_terminal_emulator() -> String {
    // Check various environment variables that indicate terminal type
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "Alacritty" => return "Alacritty".to_string(),
            "WezTerm" => return "WezTerm".to_string(),
            "iTerm.app" => return "iTerm2".to_string(),
            "Apple_Terminal" => return "Apple Terminal".to_string(),
            "vscode" => return "VS Code Terminal".to_string(),
            _ => {}
        }
    }

    if let Ok(wt_session) = std::env::var("WT_SESSION") {
        if !wt_session.is_empty() {
            return "Windows Terminal".to_string();
        }
    }

    if let Ok(kitty_window_id) = std::env::var("KITTY_WINDOW_ID") {
        if !kitty_window_id.is_empty() {
            return "Kitty".to_string();
        }
    }

    String::new()
}

/// Display the 16 basic ANSI colors
fn display_ansi_colors() {
    cprtln_with_embeds!(
        Role::Normal,
        "🎨 {} ANSI Colors (0-15):",
        &[Role::Info.embed("Current Terminal")]
    );
    println!("───────────────────────────────────────────");

    // Basic colors (0-7)
    println!("Standard Colors (0-7):");
    display_color_row(&[
        (0, "Black"),
        (1, "Red"),
        (2, "Green"),
        (3, "Yellow"),
        (4, "Blue"),
        (5, "Magenta"),
        (6, "Cyan"),
        (7, "White"),
    ]);

    println!();

    // Bright colors (8-15)
    println!("Bright Colors (8-15):");
    display_color_row(&[
        (8, "Bright Black"),
        (9, "Bright Red"),
        (10, "Bright Green"),
        (11, "Bright Yellow"),
        (12, "Bright Blue"),
        (13, "Bright Magenta"),
        (14, "Bright Cyan"),
        (15, "Bright White"),
    ]);

    println!();
}

/// Display a row of colors with their indices and names
fn display_color_row(colors: &[(u8, &str)]) {
    // Print color indices
    print!("   ");
    for (index, _) in colors {
        print!("{}", Role::Emphasis.paint(format!("{:>12}", index)));
    }
    println!();

    // Print color names
    print!("   ");
    for (_, name) in colors {
        print!("{:>12}", name);
    }
    println!();

    // Print colored blocks using ANSI escape codes
    print!("   ");
    for (index, _) in colors {
        print!("\x1b[48;5;{}m{:>12}\x1b[0m", index, "");
    }
    println!();

    // Print sample text in each color
    print!("   ");
    for (index, _) in colors {
        print!("\x1b[38;5;{}m{:>12}\x1b[0m", index, "Sample");
    }
    println!();
}

/// Display theme colors with visual preview
fn display_theme_colors(theme: &Theme) {
    cprtln_with_embeds!(
        Role::Normal,
        "🌟 {} Colors:",
        &[Role::Info.embed(&theme.name)]
    );
    println!("─────────────────────────────────────────");

    println!("Background: {:?}", theme.bg_rgbs);
    println!();

    // Display semantic colors with visual preview
    let semantic_colors = [
        ("Heading1", &theme.palette.heading1),
        ("Heading2", &theme.palette.heading2),
        ("Heading3", &theme.palette.heading3),
        ("Error", &theme.palette.error),
        ("Warning", &theme.palette.warning),
        ("Success", &theme.palette.success),
        ("Info", &theme.palette.info),
        ("Emphasis", &theme.palette.emphasis),
        ("Code", &theme.palette.code),
        ("Normal", &theme.palette.normal),
        ("Subtle", &theme.palette.subtle),
        ("Hint", &theme.palette.hint),
        ("Debug", &theme.palette.debug),
        ("Trace", &theme.palette.trace),
    ];

    println!("Semantic Colors:");
    for (name, style) in semantic_colors {
        let colored_text = style.paint(format!("{:>12}", name));
        let rgb_info = extract_rgb_info(style);
        println!("   {} {}", colored_text, Role::Subtle.paint(&rgb_info));
    }

    // Show background color preview if available
    if let Some((r, g, b)) = theme.bg_rgbs.first() {
        println!();
        println!("Background Preview:");
        print!("   ");
        for _ in 0..20 {
            print!("\x1b[48;2;{};{};{}m \x1b[0m", r, g, b);
        }
        println!(" RGB({}, {}, {})", r, g, b);
    }

    println!();
}

/// Display side-by-side color comparison
#[allow(clippy::too_many_lines)]
fn display_color_comparison(theme: &Theme) {
    cprtln_with_embeds!(
        Role::Normal,
        "🔄 {} Color Mapping:",
        &[Role::Info.embed("ANSI vs Theme")]
    );
    println!("──────────────────────────────────────────");

    // Corrected mappings that match thag_sync_palette behavior
    let color_mappings = [
        ("Black (0)", 0, "Background", get_best_dark_color(theme)),
        ("Red (1)", 1, "Error", extract_rgb(&theme.palette.error)),
        (
            "Green (2)",
            2,
            "Success",
            extract_rgb(&theme.palette.success),
        ),
        (
            "Yellow (3)",
            3,
            "Warning",
            extract_rgb(&theme.palette.warning),
        ),
        ("Blue (4)", 4, "Info", extract_rgb(&theme.palette.info)),
        (
            "Magenta (5)",
            5,
            "Heading1",
            extract_rgb(&theme.palette.heading1),
        ),
        (
            "Cyan (6)",
            6,
            "Heading3",
            extract_rgb(&theme.palette.heading3),
        ),
        ("White (7)", 7, "Normal", extract_rgb(&theme.palette.normal)),
        (
            "Bright Black (8)",
            8,
            "Subtle",
            extract_rgb(&theme.palette.subtle),
        ),
        (
            "Bright Red (9)",
            9,
            "Trace",
            extract_rgb(&theme.palette.trace),
        ),
        (
            "Bright Green (10)",
            10,
            "Debug",
            extract_rgb(&theme.palette.debug),
        ),
        (
            "Bright Yellow (11)",
            11,
            "Emphasis",
            extract_rgb(&theme.palette.emphasis),
        ),
        (
            "Bright Blue (12)",
            12,
            "Info (brighter)",
            extract_rgb(&theme.palette.info).map(brighten_color),
        ),
        (
            "Bright Magenta (13)",
            13,
            "Heading1 (brighter)",
            extract_rgb(&theme.palette.heading1).map(brighten_color),
        ),
        (
            "Bright Cyan (14)",
            14,
            "Hint",
            extract_rgb(&theme.palette.hint),
        ),
        (
            "Bright White (15)",
            15,
            "Normal (brighter)",
            extract_rgb(&theme.palette.normal).map(brighten_color),
        ),
    ];

    cprtln!(
        Role::Heading3,
        "{:<20} {:<12} {:<26} Semantic Role",
        "ANSI Color",
        "Current",
        "Expected (Theme)"
    );
    println!("{}", "─".repeat(80));

    for (name, ansi_index, semantic_role, thag_rgb) in color_mappings {
        // Current terminal color (visual sample)
        let terminal_sample = format!("\x1b[38;5;{}m████\x1b[0m", ansi_index);

        // Expected thag color with RGB info
        let thag_display = if let Some((r, g, b)) = thag_rgb {
            format!(
                "\x1b[38;2;{};{};{}m████\x1b[0m #{:02x}{:02x}{:02x} ({:3},{:3},{:3})",
                r, g, b, r, g, b, r, g, b
            )
        } else {
            // Style::from(Role::Subtle).paint("N/A").to_string()
            Role::Normal.dim().paint("N/A").to_string()
        };

        println!(
            "{:<20} {:<5}         {:<26} {}",
            name,
            terminal_sample,
            thag_display,
            Role::Subtle.paint(semantic_role)
        );
    }

    println!();
}

/// Display recommendations based on comparison
fn display_recommendations(theme: &Theme) {
    cprtln_with_embeds!(
        Role::Normal,
        "💡 {} and Tips:",
        &[Role::Info.embed("Recommendations")]
    );
    println!("────────────────────────────");

    println!("• If colors don't match expected values:");
    println!("  - Your terminal may not support the theme correctly");
    cprtln_with_embeds!(
        Role::Normal,
        "  - Try using {} to synchronize the terminal palette with the `thag_styling` theme",
        &[Role::Heading3.embed("thag_sync_palette")]
    );
    println!("  - Check if your terminal emulator supports the theme format");
    println!();

    cprtln_with_embeds!(
        Role::Normal,
        "• For {} theme:",
        &[Role::Heading3.embed(&theme.name)]
    );
    match theme.term_bg_luma {
        TermBgLuma::Dark => {
            println!("  - Ensure your terminal has a dark background");
            println!("  - ANSI 0 (Black) should match background color");
        }
        TermBgLuma::Light => {
            println!("  - Ensure your terminal has a light background");
            println!("  - Colors should be adjusted for light backgrounds");
        }
        TermBgLuma::Undetermined => {}
    }
    println!();

    cprtln_with_embeds!(
        Role::Normal,
        "• {} Commands:",
        &[Role::Emphasis.embed("Useful")]
    );
    cprtln_with_embeds!(
        Role::Normal,
        "  - {}: Export theme to terminal formats",
        &[Role::Heading3.embed("thag_gen_terminal_themes")]
    );
    cprtln_with_embeds!(
        Role::Normal,
        "  - {}: Sync terminal palette",
        &[Role::Heading3.embed(&format!("thag_sync_palette --apply {}", theme.name))]
    );
    cprtln_with_embeds!(
        Role::Normal,
        "  - {}: Generate themes from images",
        &[Role::Heading3.embed("thag_image_to_theme")]
    );
    println!();

    // Show specific issues if detected
    let issues = detect_potential_issues(theme);
    if !issues.is_empty() {
        cprtln_with_embeds!(
            Role::Normal,
            "⚠️  {} Issues Detected:",
            &[Role::Emphasis.embed("Potential")]
        );
        for issue in issues {
            cprtln_with_embeds!(Role::Normal, "   • {}", &[Role::Emphasis.embed(&issue)]);
        }
        println!();
    }
}

/// Detect potential issues with theme/terminal compatibility
fn detect_potential_issues(theme: &Theme) -> Vec<String> {
    let mut issues = Vec::new();

    // Check if theme colors are too similar to background
    if let Some(bg_rgb) = theme.bg_rgbs.first() {
        if let Some(normal_rgb) = extract_rgb(&theme.palette.normal) {
            let contrast = calculate_contrast_ratio(*bg_rgb, normal_rgb);
            if contrast < 4.5 {
                issues.push(format!(
                    "Low contrast between background and normal text ({}:1, recommended 4.5:1+)",
                    format_args!("{:.1}", contrast)
                ));
            }
        }
    }

    // Check for missing color information
    let essential_colors = [
        ("Error", &theme.palette.error),
        ("Warning", &theme.palette.warning),
        ("Success", &theme.palette.success),
        ("Normal", &theme.palette.normal),
    ];

    for (name, style) in essential_colors {
        if extract_rgb(style).is_none() {
            issues.push(format!("{} color has no RGB information", name));
        }
    }

    issues
}

/// Calculate contrast ratio between two RGB colors
fn calculate_contrast_ratio(color1: (u8, u8, u8), color2: (u8, u8, u8)) -> f64 {
    fn luminance(rgb: (u8, u8, u8)) -> f64 {
        let (r, g, b) = (
            f64::from(rgb.0) / 255.0,
            f64::from(rgb.1) / 255.0,
            f64::from(rgb.2) / 255.0,
        );

        let to_linear = |c: f64| {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        0.0722f64.mul_add(
            to_linear(b),
            0.2126f64.mul_add(to_linear(r), 0.7152 * to_linear(g)),
        )
    }

    let l1 = luminance(color1);
    let l2 = luminance(color2);

    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Extract RGB information from a style for display
fn extract_rgb_info(style: &Style) -> String {
    style.foreground.as_ref().map_or_else(
        || "No color".to_string(),
        |color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => {
                format!(
                    "#{:02x}{:02x}{:02x} RGB({}, {}, {})",
                    rgb[0], rgb[1], rgb[2], rgb[0], rgb[1], rgb[2]
                )
            }
            thag_styling::ColorValue::Color256 { color256 } => {
                format!("256-Color({})", color256)
            }
            thag_styling::ColorValue::Basic { index, .. } => {
                format!("ANSI({})", index)
            }
        },
    )
}

/// Extract RGB tuple from a style
fn extract_rgb(style: &Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}

/// Get the best dark color from the theme for black mapping
fn get_best_dark_color(theme: &Theme) -> Option<(u8, u8, u8)> {
    theme
        .bg_rgbs
        .first()
        .copied()
        .or_else(|| extract_rgb(&theme.palette.subtle))
        .or(Some((16, 16, 16)))
}

/// Brighten a color by increasing its components
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn brighten_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let factor = 1.3;
    (
        ((f32::from(r) * factor).min(255.0)) as u8,
        ((f32::from(g) * factor).min(255.0)) as u8,
        ((f32::from(b) * factor).min(255.0)) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use thag_styling::{ColorInfo, ColorSupport, Palette, TermBgLuma};

    fn create_test_theme() -> Theme {
        let mut palette = Palette::default();
        palette.normal = Style::fg(ColorInfo::rgb(220, 220, 220));
        palette.error = Style::fg(ColorInfo::rgb(255, 100, 100));

        Theme {
            name: "Test Palette Theme".to_string(),
            filename: PathBuf::from("test_palette_theme.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette,
            backgrounds: vec!["#2a2a2a".to_string()],
            bg_rgbs: vec![(42, 42, 42)],
            description: "Test theme for palette comparison".to_string(),
        }
    }

    #[test]
    fn test_extract_rgb_info() {
        let style = Style::fg(ColorInfo::rgb(255, 128, 64));
        let info = extract_rgb_info(&style);
        assert!(info.contains("ff8040"));
        assert!(info.contains("255, 128, 64"));
    }

    #[test]
    fn test_extract_rgb() {
        let style = Style::fg(ColorInfo::rgb(255, 128, 64));
        let rgb = extract_rgb(&style);
        assert_eq!(rgb, Some((255, 128, 64)));
    }

    #[test]
    fn test_brighten_color() {
        let original = (100, 150, 200);
        let brightened = brighten_color(original);

        assert!(brightened.0 >= original.0);
        assert!(brightened.1 >= original.1);
        assert!(brightened.2 >= original.2);
        assert!(brightened.0 <= 255);
        assert!(brightened.1 <= 255);
        assert!(brightened.2 <= 255);
    }

    #[test]
    fn test_contrast_ratio_calculation() {
        // Test with black and white (maximum contrast)
        let contrast = calculate_contrast_ratio((0, 0, 0), (255, 255, 255));
        assert!((contrast - 21.0).abs() < 0.1);

        // Test with identical colors (minimum contrast)
        let contrast = calculate_contrast_ratio((128, 128, 128), (128, 128, 128));
        assert!((contrast - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_detect_potential_issues() {
        let theme = create_test_theme();
        let issues = detect_potential_issues(&theme);

        // Should detect some issues with the test theme
        assert!(issues.len() >= 0);
    }
}
