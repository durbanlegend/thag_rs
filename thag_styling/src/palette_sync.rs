//! Runtime terminal palette synchronization using OSC escape sequences
//!
//! This module provides functionality to update a terminal emulator's color palette
//! in real-time using OSC (Operating System Command) escape sequences. This allows
//! thag_styling themes to be applied directly to the terminal's base palette,
//! ensuring visual consistency between thag-styled output and regular terminal content.
//!
//! # Supported Terminals
//!
//! Most modern terminal emulators support OSC sequences for palette updates:
//! - WezTerm, Alacritty, iTerm2, Kitty, Gnome Terminal, Windows Terminal, etc.
//!
//! # Example
//!
//! ```rust
//! use thag_styling::palette_sync::PaletteSync;
//! use thag_styling::{Theme, TermAttributes};
//!
//! // Load a theme
//! let theme = Theme::get_builtin("thag-botticelli-birth-of-venus").unwrap();
//!
//! // Apply to terminal
//! PaletteSync::apply_theme(&theme)?;
//!
//! // Later, reset to defaults
//! PaletteSync::reset_palette()?;
//! ```

use crate::{Role, Theme};
use std::io::{self, Write};

/// Terminal palette synchronization using OSC sequences
pub struct PaletteSync;

impl PaletteSync {
    /// Apply a thag_styling theme to the terminal's color palette
    ///
    /// This updates both the 16-color ANSI palette (colors 0-15) and the
    /// terminal's default foreground/background colors using OSC sequences.
    ///
    /// # Arguments
    /// * `theme` - The theme to apply to the terminal palette
    ///
    /// # Returns
    /// * `Ok(())` if the palette was updated successfully
    /// * `Err(io::Error)` if there was an error writing to stdout
    ///
    /// # Note
    /// Changes are applied immediately but may not persist after the terminal session ends.
    /// The actual visual change depends on the terminal emulator's OSC support.
    pub fn apply_theme(theme: &Theme) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Map thag_styling roles to standard ANSI color indices
        let color_map = Self::build_ansi_color_map(theme);

        // Apply ANSI colors 0-15
        for (ansi_index, rgb) in color_map.iter().enumerate() {
            let osc = format!(
                "\x1b]4;{};rgb:{:02x}/{:02x}/{:02x}\x07",
                ansi_index, rgb[0], rgb[1], rgb[2]
            );
            stdout.write_all(osc.as_bytes())?;
        }

        // Set default background color (OSC 11)
        if let Some(bg_rgb) = theme.bg_rgbs.first() {
            let osc = format!(
                "\x1b]11;rgb:{:02x}/{:02x}/{:02x}\x07",
                bg_rgb.0, bg_rgb.1, bg_rgb.2
            );
            stdout.write_all(osc.as_bytes())?;
        }

        // Set default foreground color (OSC 10) - use normal role
        let normal_rgb = theme
            .style_for(Role::Normal)
            .foreground
            .as_ref()
            .map(|color_info| match &color_info.value {
                crate::ColorValue::TrueColor { rgb } => *rgb,
                crate::ColorValue::Color256 { color256: _ } => {
                    [color_info.index, color_info.index, color_info.index]
                } // Approximation
                crate::ColorValue::Basic { .. } => {
                    [color_info.index, color_info.index, color_info.index]
                } // Approximation
            })
            .unwrap_or([192, 192, 192]); // Default light gray

        let osc = format!(
            "\x1b]10;rgb:{:02x}/{:02x}/{:02x}\x07",
            normal_rgb[0], normal_rgb[1], normal_rgb[2]
        );
        stdout.write_all(osc.as_bytes())?;

        stdout.flush()?;
        Ok(())
    }

    /// Reset the terminal palette to its default colors
    ///
    /// This uses OSC 104 to reset all colors and OSC 110/111 to reset
    /// the default foreground and background colors.
    pub fn reset_palette() -> io::Result<()> {
        let mut stdout = io::stdout();

        // Reset all palette colors (OSC 104)
        stdout.write_all(b"\x1b]104\x07")?;

        // Reset default foreground (OSC 110)
        stdout.write_all(b"\x1b]110\x07")?;

        // Reset default background (OSC 111)
        stdout.write_all(b"\x1b]111\x07")?;

        stdout.flush()?;
        Ok(())
    }

    /// Preview a theme by applying it temporarily with a reset option
    ///
    /// This applies the theme and provides instructions for resetting.
    /// Useful for testing themes interactively.
    pub fn preview_theme(theme: &Theme) -> io::Result<()> {
        println!("🎨 Applying theme: {}", theme.name);
        println!("📝 Description: {}", theme.description);

        Self::apply_theme(theme)?;

        println!("✅ Theme applied! Your terminal colors have been updated.");
        println!("💡 To reset colors, run: thag reset-palette");
        println!("🔄 Or call PaletteSync::reset_palette() from code");

        Ok(())
    }

    /// Build a mapping from ANSI color indices (0-15) to RGB values based on theme roles
    ///
    /// Maps thag_styling semantic roles to traditional ANSI color meanings:
    /// - 0: Black (subtle)
    /// - 1: Red (error)
    /// - 2: Green (success)
    /// - 3: Yellow (warning)
    /// - 4: Blue (code)
    /// - 5: Magenta (emphasis)
    /// - 6: Cyan (info)
    /// - 7: White (normal)
    /// - 8: Bright Black (debug)
    /// - 9: Bright Red (error, brighter)
    /// - 10: Bright Green (success, brighter)
    /// - 11: Bright Yellow (heading3)
    /// - 12: Bright Blue (heading2)
    /// - 13: Bright Magenta (heading1)
    /// - 14: Bright Cyan (hint)
    /// - 15: Bright White (normal, brighter)
    fn build_ansi_color_map(theme: &Theme) -> [[u8; 3]; 16] {
        let extract_rgb = |role: Role| -> [u8; 3] {
            theme
                .style_for(role)
                .foreground
                .as_ref()
                .map(|color_info| match &color_info.value {
                    crate::ColorValue::TrueColor { rgb } => *rgb,
                    crate::ColorValue::Color256 { .. } => {
                        // Convert color256 index to approximate RGB
                        let (r, g, b) = crate::styling::get_rgb(color_info.index);
                        [r, g, b]
                    }
                    crate::ColorValue::Basic { .. } => {
                        // Use basic color mapping
                        let (r, g, b) = crate::styling::get_rgb(color_info.index);
                        [r, g, b]
                    }
                })
                .unwrap_or([128, 128, 128]) // Default gray
        };

        // Get background color for black (0)
        let bg_rgb = theme
            .bg_rgbs
            .first()
            .copied()
            .map(|(r, g, b)| [r, g, b])
            .unwrap_or([0, 0, 0]);

        [
            bg_rgb,                                          // 0: Black (use theme background)
            extract_rgb(Role::Error),                        // 1: Red
            extract_rgb(Role::Success),                      // 2: Green
            extract_rgb(Role::Warning),                      // 3: Yellow
            extract_rgb(Role::Code),                         // 4: Blue
            extract_rgb(Role::Emphasis),                     // 5: Magenta
            extract_rgb(Role::Info),                         // 6: Cyan
            extract_rgb(Role::Normal),                       // 7: White
            extract_rgb(Role::Subtle), // 8: Bright Black (use Subtle instead of Debug)
            extract_rgb(Role::Trace),  // 9: Bright Red (use Trace instead of brightened Error)
            extract_rgb(Role::Debug),  // 10: Bright Green (use Debug instead of brightened Success)
            extract_rgb(Role::Heading3), // 11: Bright Yellow
            extract_rgb(Role::Heading2), // 12: Bright Blue
            extract_rgb(Role::Heading1), // 13: Bright Magenta
            extract_rgb(Role::Hint),   // 14: Bright Cyan
            Self::brighten_color(extract_rgb(Role::Normal)), // 15: Bright White
        ]
    }

    /// Brighten a color by increasing its lightness
    fn brighten_color(rgb: [u8; 3]) -> [u8; 3] {
        let factor = 1.3f32;
        [
            ((rgb[0] as f32 * factor).min(255.0)) as u8,
            ((rgb[1] as f32 * factor).min(255.0)) as u8,
            ((rgb[2] as f32 * factor).min(255.0)) as u8,
        ]
    }

    /// Print a color demonstration using the current palette
    ///
    /// This shows how the ANSI colors look after applying a theme,
    /// useful for verifying the palette update worked correctly.
    pub fn demonstrate_palette() -> io::Result<()> {
        println!("🎨 Current Terminal Palette:");
        println!();

        // Show standard ANSI colors
        for i in 0..8 {
            print!("\x1b[3{};40m  {:<12}\x1b[0m", i, Self::color_name(i));
        }
        println!();

        // Show bright ANSI colors
        for i in 0..8 {
            print!("\x1b[9{};40m  {:<12}\x1b[0m", i, Self::bright_color_name(i));
        }
        println!();
        println!();

        // Show thag roles mapped to ANSI colors
        println!("🏷️  Thag Styling Roles (using updated ANSI palette):");
        println!("\x1b[31m● Error\x1b[0m - Something went wrong (ANSI 1: Red)");
        println!("\x1b[33m● Warning\x1b[0m - Pay attention to this (ANSI 3: Yellow)");
        println!("\x1b[32m● Success\x1b[0m - Everything worked! (ANSI 2: Green)");
        println!("\x1b[36m● Info\x1b[0m - Informational message (ANSI 6: Cyan)");
        println!("\x1b[35m● Emphasis\x1b[0m - Important content (ANSI 5: Magenta)");
        println!("\x1b[34m● Code\x1b[0m - `code blocks and filenames` (ANSI 4: Blue)");
        println!("\x1b[37m● Normal\x1b[0m - Regular text (ANSI 7: White)");
        println!("\x1b[90m● Subtle\x1b[0m - Secondary information (ANSI 8: Bright Black)");
        println!("\x1b[94m● Hint\x1b[0m - Helpful suggestions (ANSI 14: Bright Cyan)");
        println!("\x1b[95m● Heading1\x1b[0m - Major sections (ANSI 13: Bright Magenta)");
        println!("\x1b[94m● Heading2\x1b[0m - Subsections (ANSI 12: Bright Blue)");
        println!("\x1b[93m● Heading3\x1b[0m - Minor sections (ANSI 11: Bright Yellow)");
        println!("\x1b[92m● Debug\x1b[0m - Development info (ANSI 10: Bright Green)");
        println!("\x1b[91m● Trace\x1b[0m - Detailed diagnostic (ANSI 9: Bright Red)");

        Ok(())
    }

    /// Display current background color information
    pub fn show_background_info(theme: &Theme) -> io::Result<()> {
        println!();
        println!("🖼️  Background Information:");
        if let Some(bg_rgb) = theme.bg_rgbs.first() {
            println!("   RGB: ({}, {}, {})", bg_rgb.0, bg_rgb.1, bg_rgb.2);
            println!("   Hex: #{:02x}{:02x}{:02x}", bg_rgb.0, bg_rgb.1, bg_rgb.2);

            // Show a sample with the background color
            let osc = format!(
                "\x1b]11;rgb:{:02x}/{:02x}/{:02x}\x07",
                bg_rgb.0, bg_rgb.1, bg_rgb.2
            );
            print!("{}", osc);
            println!("   Sample: \x1b[37m█████\x1b[0m (background should match theme)");
        }
        Ok(())
    }

    /// Show hybrid demonstration: both palette colors and proper styling with attributes
    pub fn demonstrate_hybrid_styling(theme: &Theme) -> io::Result<()> {
        println!();
        println!("🎨 Hybrid Color Demonstration:");
        println!("   📋 Left: ANSI palette colors | Right: Thag styling with attributes");
        println!();

        use crate::{Role, Style, StyleLike};

        let roles_and_messages = [
            (
                Role::Heading1,
                "# Heading 1 - Major sections",
                "\x1b[95m",
                "ANSI 13: Bright Magenta",
            ),
            (
                Role::Heading2,
                "## Heading 2 - Subsections",
                "\x1b[94m",
                "ANSI 12: Bright Blue",
            ),
            (
                Role::Heading3,
                "### Heading 3 - Minor sections",
                "\x1b[93m",
                "ANSI 11: Bright Yellow",
            ),
            (
                Role::Error,
                "❌ Error - Something went wrong",
                "\x1b[31m",
                "ANSI 1: Red",
            ),
            (
                Role::Warning,
                "⚠️  Warning - Pay attention",
                "\x1b[33m",
                "ANSI 3: Yellow",
            ),
            (
                Role::Success,
                "✅ Success - Everything worked!",
                "\x1b[32m",
                "ANSI 2: Green",
            ),
            (
                Role::Info,
                "ℹ️  Info - Informational message",
                "\x1b[36m",
                "ANSI 6: Cyan",
            ),
            (
                Role::Emphasis,
                "⭐ Emphasis - Important content",
                "\x1b[35m",
                "ANSI 5: Magenta",
            ),
            (
                Role::Code,
                "💻 Code - `filenames and code blocks`",
                "\x1b[34m",
                "ANSI 4: Blue",
            ),
            (
                Role::Normal,
                "📄 Normal - Regular text content",
                "\x1b[37m",
                "ANSI 7: White",
            ),
            (
                Role::Subtle,
                "🔍 Subtle - Secondary information",
                "\x1b[90m",
                "ANSI 8: Bright Black",
            ),
            (
                Role::Hint,
                "💡 Hint - Helpful suggestions",
                "\x1b[96m",
                "ANSI 14: Bright Cyan",
            ),
            (
                Role::Debug,
                "🐛 Debug - Development info",
                "\x1b[92m",
                "ANSI 10: Bright Green",
            ),
            (
                Role::Trace,
                "🔍 Trace - Detailed diagnostic",
                "\x1b[91m",
                "ANSI 9: Bright Red",
            ),
        ];

        for (role, message, ansi_code, ansi_desc) in roles_and_messages {
            let style = Style::from(role);

            // Left side: ANSI palette color only
            print!("{}{}  {}\x1b[0m", ansi_code, "●", ansi_desc);

            // Right side: Thag styling with attributes
            print!("  │  ");
            style.prtln(format_args!("{}", message));
        }

        println!();
        println!("🎯 Left colors come from updated terminal palette");
        println!("🎯 Right colors/attributes come from thag styling system");
        println!("💡 The combination gives you consistent theming everywhere!");

        Ok(())
    }

    fn color_name(index: u8) -> &'static str {
        match index {
            0 => "Black",
            1 => "Red",
            2 => "Green",
            3 => "Yellow",
            4 => "Blue",
            5 => "Magenta",
            6 => "Cyan",
            7 => "White",
            _ => "Unknown",
        }
    }

    fn bright_color_name(index: u8) -> &'static str {
        match index {
            0 => "Bright Black",
            1 => "Bright Red",
            2 => "Bright Green",
            3 => "Bright Yellow",
            4 => "Bright Blue",
            5 => "Bright Magenta",
            6 => "Bright Cyan",
            7 => "Bright White",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::styling::TermAttributes;

    #[test]
    fn test_ansi_color_mapping() {
        // Initialize with a basic theme for testing
        TermAttributes::initialize(crate::ColorInitStrategy::Configure {
            theme: Some("thag-dark".to_string()),
            color_support: Some(crate::ColorSupport::TrueColor),
            backgrounds: Some(vec!["#000000".to_string()]),
        })
        .unwrap();

        let theme = crate::Theme::get_builtin("thag-dark").unwrap();
        let color_map = PaletteSync::build_ansi_color_map(&theme);

        // Verify we got 16 colors
        assert_eq!(color_map.len(), 16);

        // Verify colors are not all the same (basic sanity check)
        let first_color = color_map[0];
        let has_different = color_map.iter().any(|&color| color != first_color);
        assert!(has_different, "All colors should not be identical");
    }

    #[test]
    fn test_brighten_color() {
        let dark_red = [100, 0, 0];
        let bright_red = PaletteSync::brighten_color(dark_red);

        // Should be brighter
        assert!(bright_red[0] > dark_red[0]);

        // Should not exceed 255
        let white = [255, 255, 255];
        let bright_white = PaletteSync::brighten_color(white);
        assert!(bright_white[0] <= 255);
        assert!(bright_white[1] <= 255);
        assert!(bright_white[2] <= 255);
    }
}
