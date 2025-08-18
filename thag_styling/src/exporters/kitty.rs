//! Kitty terminal theme exporter
//!
//! Exports thag themes to Kitty's configuration format.
//! Kitty uses a simple key-value configuration format for color schemes.

use crate::{
    exporters::{brighten_color, ThemeExporter},
    ColorValue, StylingResult, Theme,
};

/// Kitty theme exporter
pub struct KittyExporter;

impl ThemeExporter for KittyExporter {
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!(
            "# Kitty Color Scheme: {}\n# Generated from thag theme\n# {}\n\n",
            theme.name, theme.description
        ));

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Basic colors
        output.push_str("# Basic colors\n");
        output.push_str(&format!(
            "background #{:02x}{:02x}{:02x}\n",
            bg_color.0, bg_color.1, bg_color.2
        ));

        if let Some(fg_color) = get_rgb_from_style(&theme.palette.normal) {
            output.push_str(&format!(
                "foreground #{:02x}{:02x}{:02x}\n",
                fg_color.0, fg_color.1, fg_color.2
            ));
        }

        output.push_str("\n");

        // Selection colors
        output.push_str("# Selection colors\n");
        let selection_bg = adjust_color_brightness(bg_color, 1.4);
        output.push_str(&format!(
            "selection_background #{:02x}{:02x}{:02x}\n",
            selection_bg.0, selection_bg.1, selection_bg.2
        ));

        if let Some(selection_fg) = get_rgb_from_style(&theme.palette.normal) {
            output.push_str(&format!(
                "selection_foreground #{:02x}{:02x}{:02x}\n",
                selection_fg.0, selection_fg.1, selection_fg.2
            ));
        }

        output.push_str("\n");

        // Cursor colors
        output.push_str("# Cursor colors\n");
        if let Some(cursor_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            output.push_str(&format!(
                "cursor #{:02x}{:02x}{:02x}\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            ));

            // Cursor text should contrast with cursor color
            let cursor_text = if is_light_color(cursor_color) {
                bg_color // Use background color for contrast
            } else {
                get_rgb_from_style(&theme.palette.normal).unwrap_or((255, 255, 255))
            };
            output.push_str(&format!(
                "cursor_text_color #{:02x}{:02x}{:02x}\n",
                cursor_text.0, cursor_text.1, cursor_text.2
            ));
        }

        output.push_str("\n");

        // URL colors
        output.push_str("# URL underline color when hovering with mouse\n");
        if let Some(url_color) = get_rgb_from_style(&theme.palette.info) {
            output.push_str(&format!(
                "url_color #{:02x}{:02x}{:02x}\n",
                url_color.0, url_color.1, url_color.2
            ));
        }

        output.push_str("\n");

        // Visual bell
        output.push_str("# Visual bell color\n");
        if let Some(bell_color) = get_rgb_from_style(&theme.palette.warning) {
            output.push_str(&format!(
                "visual_bell_color #{:02x}{:02x}{:02x}\n",
                bell_color.0, bell_color.1, bell_color.2
            ));
        }

        output.push_str("\n");

        // Active border color
        output.push_str("# Border colors\n");
        if let Some(active_border) = get_rgb_from_style(&theme.palette.emphasis) {
            output.push_str(&format!(
                "active_border_color #{:02x}{:02x}{:02x}\n",
                active_border.0, active_border.1, active_border.2
            ));
        }

        if let Some(inactive_border) = get_rgb_from_style(&theme.palette.subtle)
            .or_else(|| Some(adjust_color_brightness(bg_color, 1.3)))
        {
            output.push_str(&format!(
                "inactive_border_color #{:02x}{:02x}{:02x}\n",
                inactive_border.0, inactive_border.1, inactive_border.2
            ));
        }

        output.push_str("\n");

        // Tab colors
        output.push_str("# Tab bar colors\n");
        let tab_bg = adjust_color_brightness(bg_color, 0.9);
        output.push_str(&format!(
            "tab_bar_background #{:02x}{:02x}{:02x}\n",
            tab_bg.0, tab_bg.1, tab_bg.2
        ));

        if let Some(active_tab_fg) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            output.push_str(&format!(
                "active_tab_foreground #{:02x}{:02x}{:02x}\n",
                active_tab_fg.0, active_tab_fg.1, active_tab_fg.2
            ));
        }

        output.push_str(&format!(
            "active_tab_background #{:02x}{:02x}{:02x}\n",
            bg_color.0, bg_color.1, bg_color.2
        ));

        if let Some(inactive_tab_fg) = get_rgb_from_style(&theme.palette.subtle)
            .or_else(|| get_rgb_from_style(&theme.palette.normal).map(dim_color))
        {
            output.push_str(&format!(
                "inactive_tab_foreground #{:02x}{:02x}{:02x}\n",
                inactive_tab_fg.0, inactive_tab_fg.1, inactive_tab_fg.2
            ));
        }

        output.push_str(&format!(
            "inactive_tab_background #{:02x}{:02x}{:02x}\n",
            tab_bg.0, tab_bg.1, tab_bg.2
        ));

        output.push_str("\n");

        // Mark colors (for marks and text search)
        output.push_str("# Mark colors (for text search highlighting)\n");
        if let Some(mark1_bg) = get_rgb_from_style(&theme.palette.warning) {
            output.push_str(&format!(
                "mark1_background #{:02x}{:02x}{:02x}\n",
                mark1_bg.0, mark1_bg.1, mark1_bg.2
            ));
            output.push_str(&format!(
                "mark1_foreground #{:02x}{:02x}{:02x}\n",
                bg_color.0, bg_color.1, bg_color.2
            ));
        }

        if let Some(mark2_bg) = get_rgb_from_style(&theme.palette.info) {
            output.push_str(&format!(
                "mark2_background #{:02x}{:02x}{:02x}\n",
                mark2_bg.0, mark2_bg.1, mark2_bg.2
            ));
            output.push_str(&format!(
                "mark2_foreground #{:02x}{:02x}{:02x}\n",
                bg_color.0, bg_color.1, bg_color.2
            ));
        }

        if let Some(mark3_bg) = get_rgb_from_style(&theme.palette.success) {
            output.push_str(&format!(
                "mark3_background #{:02x}{:02x}{:02x}\n",
                mark3_bg.0, mark3_bg.1, mark3_bg.2
            ));
            output.push_str(&format!(
                "mark3_foreground #{:02x}{:02x}{:02x}\n",
                bg_color.0, bg_color.1, bg_color.2
            ));
        }

        output.push_str("\n");

        // Black (0-7: normal colors)
        output.push_str("# The color table\n");
        output.push_str("#\n");
        output.push_str("# black\n");
        if let Some((r, g, b)) = get_best_dark_color(theme) {
            output.push_str(&format!("color0 #{:02x}{:02x}{:02x}\n", r, g, b));
        }
        if let Some((r, g, b)) =
            get_rgb_from_style(&theme.palette.subtle).or_else(|| Some((64, 64, 64)))
        {
            output.push_str(&format!("color8 #{:02x}{:02x}{:02x}\n", r, g, b));
        }

        output.push_str("\n# red\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.error) {
            output.push_str(&format!("color1 #{:02x}{:02x}{:02x}\n", r, g, b));
        }
        if let Some((r, g, b)) =
            get_rgb_from_style(&theme.palette.trace).or_else(|| Some((64, 64, 64)))
        {
            output.push_str(&format!("color8 #{:02x}{:02x}{:02x}\n", r, g, b));
        }

        output.push_str("\n# green\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.success) {
            output.push_str(&format!("color2 #{:02x}{:02x}{:02x}\n", r, g, b));
        }
        if let Some((r, g, b)) =
            get_rgb_from_style(&theme.palette.debug).or_else(|| Some((64, 64, 64)))
        {
            output.push_str(&format!("color8 #{:02x}{:02x}{:02x}\n", r, g, b));
        }

        output.push_str("\n# yellow\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.warning) {
            output.push_str(&format!("color3 #{:02x}{:02x}{:02x}\n", r, g, b));
        }
        if let Some((r, g, b)) =
            get_rgb_from_style(&theme.palette.emphasis).or_else(|| Some((64, 64, 64)))
        {
            output.push_str(&format!("color8 #{:02x}{:02x}{:02x}\n", r, g, b));
        }

        output.push_str("\n# blue\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.info) {
            output.push_str(&format!("color4 #{:02x}{:02x}{:02x}\n", r, g, b));
            let bright = brighten_color((r, g, b));
            output.push_str(&format!(
                "color12 #{:02x}{:02x}{:02x}\n",
                bright.0, bright.1, bright.2
            ));
        }

        output.push_str("\n# magenta\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.heading1) {
            output.push_str(&format!("color5 #{:02x}{:02x}{:02x}\n", r, g, b));
            let bright = brighten_color((r, g, b));
            output.push_str(&format!(
                "color13 #{:02x}{:02x}{:02x}\n",
                bright.0, bright.1, bright.2
            ));
        }

        output.push_str("\n# cyan\n");
        let cyan_normal = get_rgb_from_style(&theme.palette.heading3).unwrap_or((64, 192, 192));
        output.push_str(&format!(
            "color6 #{:02x}{:02x}{:02x}\n",
            cyan_normal.0, cyan_normal.1, cyan_normal.2
        ));
        let cyan_bright = get_rgb_from_style(&theme.palette.hint).unwrap_or((64, 192, 192));
        output.push_str(&format!(
            "color14 #{:02x}{:02x}{:02x}\n",
            cyan_bright.0, cyan_bright.1, cyan_bright.2
        ));

        output.push_str("\n# white\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.normal) {
            output.push_str(&format!("color7 #{:02x}{:02x}{:02x}\n", r, g, b));
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.normal).map(brighten_color) {
            output.push_str(&format!("color15 #{:02x}{:02x}{:02x}\n", r, g, b));
        }

        output.push_str("\n");

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "conf"
    }

    fn format_name() -> &'static str {
        "Kitty"
    }
}

/// Extract RGB values from a Style's foreground color
fn get_rgb_from_style(style: &crate::Style) -> Option<(u8, u8, u8)> {
    style.foreground.as_ref().and_then(|color_info| {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            ColorValue::Color256 { color256 } => {
                // Convert 256-color index to approximate RGB
                Some(color_256_to_rgb(*color256))
            }
            ColorValue::Basic { index, .. } => {
                // Convert basic color index to RGB
                Some(basic_color_to_rgb(*index))
            }
        }
    })
}

/// Convert 256-color index to RGB
fn color_256_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        // Standard colors (0-15)
        0 => (0, 0, 0),        // Black
        1 => (128, 0, 0),      // Red
        2 => (0, 128, 0),      // Green
        3 => (128, 128, 0),    // Yellow
        4 => (0, 0, 128),      // Blue
        5 => (128, 0, 128),    // Magenta
        6 => (0, 128, 128),    // Cyan
        7 => (192, 192, 192),  // White
        8 => (128, 128, 128),  // Bright Black
        9 => (255, 0, 0),      // Bright Red
        10 => (0, 255, 0),     // Bright Green
        11 => (255, 255, 0),   // Bright Yellow
        12 => (0, 0, 255),     // Bright Blue
        13 => (255, 0, 255),   // Bright Magenta
        14 => (0, 255, 255),   // Bright Cyan
        15 => (255, 255, 255), // Bright White

        // 216 color cube (16-231)
        16..=231 => {
            let n = index - 16;
            let r = (n / 36) * 51;
            let g = ((n % 36) / 6) * 51;
            let b = (n % 6) * 51;
            (r, g, b)
        }

        // Grayscale (232-255)
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            (gray, gray, gray)
        }
    }
}

/// Convert basic color index to RGB
fn basic_color_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0 => (0, 0, 0),        // Black
        1 => (128, 0, 0),      // Red
        2 => (0, 128, 0),      // Green
        3 => (128, 128, 0),    // Yellow
        4 => (0, 0, 128),      // Blue
        5 => (128, 0, 128),    // Magenta
        6 => (0, 128, 128),    // Cyan
        7 => (192, 192, 192),  // White
        8 => (128, 128, 128),  // Bright Black
        9 => (255, 0, 0),      // Bright Red
        10 => (0, 255, 0),     // Bright Green
        11 => (255, 255, 0),   // Bright Yellow
        12 => (0, 0, 255),     // Bright Blue
        13 => (255, 0, 255),   // Bright Magenta
        14 => (0, 255, 255),   // Bright Cyan
        15 => (255, 255, 255), // Bright White
        _ => (128, 128, 128),  // Default gray
    }
}

/// Dim a color by reducing its components
fn dim_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let factor = 0.6;
    (
        (r as f32 * factor) as u8,
        (g as f32 * factor) as u8,
        (b as f32 * factor) as u8,
    )
}

/// Check if a color is considered light
fn is_light_color((r, g, b): (u8, u8, u8)) -> bool {
    // Calculate relative luminance
    let r_linear = if r <= 10 {
        r as f32 / 3294.6
    } else {
        ((r as f32 + 14.025) / 269.025).powf(2.4)
    };
    let g_linear = if g <= 10 {
        g as f32 / 3294.6
    } else {
        ((g as f32 + 14.025) / 269.025).powf(2.4)
    };
    let b_linear = if b <= 10 {
        b as f32 / 3294.6
    } else {
        ((b as f32 + 14.025) / 269.025).powf(2.4)
    };

    let luminance = 0.2126 * r_linear + 0.7152 * g_linear + 0.0722 * b_linear;
    luminance > 0.5
}

/// Adjust color brightness by a factor
fn adjust_color_brightness((r, g, b): (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    (
        ((r as f32 * factor).min(255.0).max(0.0)) as u8,
        ((g as f32 * factor).min(255.0).max(0.0)) as u8,
        ((b as f32 * factor).min(255.0).max(0.0)) as u8,
    )
}

/// Get the best dark color from the theme for black mapping
fn get_best_dark_color(theme: &Theme) -> Option<(u8, u8, u8)> {
    // Try background first, then subtle, then create a dark color
    theme
        .bg_rgbs
        .first()
        .copied()
        .or_else(|| get_rgb_from_style(&theme.palette.subtle))
        .or_else(|| Some((16, 16, 16)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSupport, Palette, TermBgLuma};
    use std::path::PathBuf;

    fn create_test_theme() -> Theme {
        Theme {
            name: "Test Theme".to_string(),
            filename: PathBuf::from("test.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette: Palette::default(),
            backgrounds: vec!["#1e1e2e".to_string()],
            bg_rgbs: vec![(30, 30, 46)],
            description: "A test theme".to_string(),
        }
    }

    #[test]
    fn test_kitty_export() {
        let theme = create_test_theme();
        let result = KittyExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check that the content contains expected configuration keys
        assert!(content.contains("background"));
        assert!(content.contains("foreground"));
        assert!(content.contains("selection_background"));
        assert!(content.contains("cursor"));
        assert!(content.contains("color0"));
        assert!(content.contains("color15"));
        assert!(content.contains("active_tab_foreground"));
        assert!(content.contains("mark1_background"));
    }

    #[test]
    fn test_color_conversions() {
        assert_eq!(color_256_to_rgb(0), (0, 0, 0));
        assert_eq!(color_256_to_rgb(15), (255, 255, 255));
        assert_eq!(basic_color_to_rgb(1), (128, 0, 0));

        assert_eq!(brighten_color((100, 100, 100)), (130, 130, 130));
        assert_eq!(dim_color((100, 100, 100)), (60, 60, 60));
    }

    #[test]
    fn test_color_brightness_detection() {
        assert!(is_light_color((255, 255, 255))); // White should be light
        assert!(!is_light_color((0, 0, 0))); // Black should be dark
        assert!(!is_light_color((64, 64, 64))); // Dark gray should be dark
    }
}
