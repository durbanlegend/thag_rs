//! iTerm2 terminal theme exporter
//!
//! Exports thag themes to iTerm2's .itermcolors XML format.
//! iTerm2 uses .itermcolors files (plist XML) for color presets that can be imported through the UI.

use crate::{
    exporters::{
        adjust_color_brightness, brighten_color, get_best_dark_color, get_rgb_from_style,
        ThemeExporter,
    },
    StylingResult, Theme,
};

use std::fmt::Write as _;

/// iTerm2 theme exporter
pub struct ITerm2Exporter;

impl ThemeExporter for ITerm2Exporter {
    #[allow(clippy::too_many_lines)]
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // XML declaration and plist header
        writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            output,
            r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#
        )?;
        writeln!(output, r#"<plist version="1.0">"#)?;
        writeln!(output, "<dict>")?;

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // ANSI Colors 0-15
        write_color_entry(&mut output, "Ansi 0 Color", get_best_dark_color(theme))?;
        write_color_entry(
            &mut output,
            "Ansi 1 Color",
            get_rgb_from_style(&theme.palette.error),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 2 Color",
            get_rgb_from_style(&theme.palette.success),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 3 Color",
            get_rgb_from_style(&theme.palette.warning),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 4 Color",
            get_rgb_from_style(&theme.palette.info),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 5 Color",
            get_rgb_from_style(&theme.palette.heading1),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 6 Color",
            get_rgb_from_style(&theme.palette.heading3),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 7 Color",
            get_rgb_from_style(&theme.palette.normal),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 8 Color",
            get_rgb_from_style(&theme.palette.subtle),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 9 Color",
            get_rgb_from_style(&theme.palette.trace),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 10 Color",
            get_rgb_from_style(&theme.palette.debug),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 11 Color",
            get_rgb_from_style(&theme.palette.emphasis),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 12 Color",
            get_rgb_from_style(&theme.palette.info).map(brighten_color),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 13 Color",
            get_rgb_from_style(&theme.palette.heading1),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 14 Color",
            get_rgb_from_style(&theme.palette.hint),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 15 Color",
            get_rgb_from_style(&theme.palette.normal).map(brighten_color),
        )?;

        // Background and foreground
        write_color_entry(&mut output, "Background Color", Some(bg_color))?;
        write_color_entry(
            &mut output,
            "Foreground Color",
            get_rgb_from_style(&theme.palette.normal),
        )?;

        // Bold color
        write_color_entry(
            &mut output,
            "Bold Color",
            get_rgb_from_style(&theme.palette.emphasis)
                .or_else(|| get_rgb_from_style(&theme.palette.normal)),
        )?;

        // Cursor colors
        write_color_entry(
            &mut output,
            "Cursor Color",
            get_rgb_from_style(&theme.palette.emphasis)
                .or_else(|| get_rgb_from_style(&theme.palette.normal)),
        )?;
        write_color_entry(&mut output, "Cursor Text Color", Some(bg_color))?;

        // Selection colors
        write_color_entry(
            &mut output,
            "Selection Color",
            Some(adjust_color_brightness(bg_color, 1.4)),
        )?;
        write_color_entry(
            &mut output,
            "Selected Text Color",
            get_rgb_from_style(&theme.palette.normal),
        )?;

        // Close the plist
        writeln!(output, "</dict>")?;
        writeln!(output, "</plist>")?;

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "itermcolors"
    }

    fn format_name() -> &'static str {
        "iTerm2"
    }
}

/// Write a color entry to the XML output
fn write_color_entry(
    output: &mut String,
    key: &str,
    rgb_opt: Option<(u8, u8, u8)>,
) -> Result<(), std::fmt::Error> {
    let (r, g, b) = rgb_opt.unwrap_or((128, 128, 128));

    // Convert to normalized float values (0.0 - 1.0)
    let red = f64::from(r) / 255.0;
    let green = f64::from(g) / 255.0;
    let blue = f64::from(b) / 255.0;

    writeln!(output, "\t<key>{}</key>", key)?;
    writeln!(output, "\t<dict>")?;
    writeln!(output, "\t\t<key>Blue Component</key>")?;
    writeln!(output, "\t\t<real>{}</real>", blue)?;
    writeln!(output, "\t\t<key>Green Component</key>")?;
    writeln!(output, "\t\t<real>{}</real>", green)?;
    writeln!(output, "\t\t<key>Red Component</key>")?;
    writeln!(output, "\t\t<real>{}</real>", red)?;
    writeln!(output, "\t</dict>")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        exporters::{basic_color_to_rgb, color_256_to_rgb},
        ColorSupport, Palette, TermBgLuma,
    };
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
    fn test_iterm2_export() {
        let theme = create_test_theme();
        let result = ITerm2Exporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check for XML structure
        assert!(content.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(content.contains("<!DOCTYPE plist"));
        assert!(content.contains("<plist version=\"1.0\">"));
        assert!(content.contains("<dict>"));
        assert!(content.contains("</dict>"));
        assert!(content.contains("</plist>"));

        // Check for required iTerm2 color keys
        assert!(content.contains("<key>Background Color</key>"));
        assert!(content.contains("<key>Foreground Color</key>"));
        assert!(content.contains("<key>Ansi 0 Color</key>"));
        assert!(content.contains("<key>Ansi 15 Color</key>"));
        assert!(content.contains("<key>Cursor Color</key>"));
        assert!(content.contains("<key>Selection Color</key>"));

        // Check for color components
        assert!(content.contains("<key>Red Component</key>"));
        assert!(content.contains("<key>Green Component</key>"));
        assert!(content.contains("<key>Blue Component</key>"));
        assert!(content.contains("<real>"));
    }

    #[test]
    fn test_write_color_entry() {
        let mut output = String::new();
        let result = write_color_entry(&mut output, "Test Color", Some((255, 128, 64)));

        assert!(result.is_ok());
        assert!(output.contains("<key>Test Color</key>"));
        assert!(output.contains("<real>1</real>")); // Red component (255/255 = 1.0)
        assert!(output.contains("<real>0.25098039215686274</real>")); // Blue component (64/255)
    }

    #[test]
    fn test_color_conversions() {
        assert_eq!(color_256_to_rgb(0), (0, 0, 0));
        assert_eq!(color_256_to_rgb(15), (255, 255, 255));
        assert_eq!(basic_color_to_rgb(1), (128, 0, 0));
    }
}
