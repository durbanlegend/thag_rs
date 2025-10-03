//! iTerm2 terminal theme exporter
//!
//! Exports thag themes to iTerm2's .itermcolors XML format.
//! iTerm2 uses .itermcolors files (plist XML) for color presets that can be imported through the UI.

use crate::{
    exporters::{adjust_color_brightness, ThemeExporter},
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
        write_color_entry(&mut output, "Ansi 0 Color", Some(theme.bg_rgbs[0]))?;
        write_color_entry(
            &mut output,
            "Ansi 1 Color",
            theme
                .palette
                .emphasis
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 2 Color",
            theme
                .palette
                .success
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 3 Color",
            theme
                .palette
                .commentary
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 4 Color",
            theme.palette.info.rgb().map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 5 Color",
            theme
                .palette
                .heading1
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 6 Color",
            theme.palette.code.rgb().map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 7 Color",
            theme
                .palette
                .normal
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 8 Color",
            theme
                .palette
                .subtle
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 9 Color",
            theme
                .palette
                .error
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 10 Color",
            theme
                .palette
                .debug
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 11 Color",
            theme
                .palette
                .warning
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 12 Color",
            theme.palette.link.rgb().map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 13 Color",
            theme
                .palette
                .heading2
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 14 Color",
            theme.palette.hint.rgb().map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(
            &mut output,
            "Ansi 15 Color",
            theme
                .palette
                .quote
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;

        // Background and foreground
        write_color_entry(&mut output, "Background Color", Some(bg_color))?;
        write_color_entry(
            &mut output,
            "Foreground Color",
            theme
                .palette
                .normal
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;

        // Bold color
        write_color_entry(
            &mut output,
            "Bold Color",
            theme
                .palette
                .emphasis
                .rgb()
                .or_else(|| theme.palette.normal.rgb())
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;

        // Cursor colors
        write_color_entry(
            &mut output,
            "Cursor Color",
            theme
                .palette
                .emphasis
                .rgb()
                .or_else(|| theme.palette.normal.rgb())
                .map(|arr| (arr[0], arr[1], arr[2])),
        )?;
        write_color_entry(&mut output, "Cursor Text Color", Some(bg_color))?;

        // Selection colors for highlighted text
        // "Selection Color" = background color of selected text (using commentary for good contrast)
        // "Selected Text Color" = foreground color of selected text (using normal for readability)
        write_color_entry(
            &mut output,
            "Selection Color",
            theme
                .palette
                .commentary
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .or_else(|| Some(adjust_color_brightness(bg_color, 1.4))),
        )?;
        write_color_entry(
            &mut output,
            "Selected Text Color",
            theme
                .palette
                .normal
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2])),
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

/// Write a color entry to the XML output in iTerm2's native format
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
    writeln!(output, "\t\t<key>Alpha Component</key>")?;
    writeln!(output, "\t\t<real>1</real>")?;
    writeln!(output, "\t\t<key>Blue Component</key>")?;
    writeln!(output, "\t\t<real>{}</real>", blue)?;
    writeln!(output, "\t\t<key>Color Space</key>")?;
    writeln!(output, "\t\t<string>P3</string>")?;
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
    use crate::exporters::{basic_color_to_rgb, color_256_to_rgb, create_test_theme};

    #[test]
    fn test_iterm2_export() {
        let theme = create_test_theme();
        let result = ITerm2Exporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check for XML structure
        assert!(content.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(content.contains("<!DOCTYPE plist"));
        assert!(content.contains(r#"<plist version="1.0">"#));
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
