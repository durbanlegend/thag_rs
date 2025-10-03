//! Format-specific theme export implementations.
//!
//! Contains exporters for Alacritty, WezTerm, Windows Terminal, and other terminal emulators.
//! Each exporter converts thag themes to the appropriate configuration format.

use crate::{StylingError, StylingResult, Theme};
use std::path::Path;

pub mod alacritty;
pub mod iterm2;
pub mod kitty;
pub mod konsole;
pub mod mintty;
pub mod wezterm;
pub mod windows_terminal;

// pub use crate::Style;

/// Trait for exporting themes to different terminal emulator formats
pub trait ThemeExporter {
    /// Export a theme to the format-specific string representation
    ///
    /// # Errors
    ///
    /// Will bubble up any i/o  errors encountered.
    fn export_theme(theme: &Theme) -> StylingResult<String>;

    /// Get the recommended file extension for this format
    fn file_extension() -> &'static str;

    /// Get a human-readable name for this format
    fn format_name() -> &'static str;
}

/// Supported terminal emulator formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Alacritty terminal emulator (TOML/YAML)
    Alacritty,
    /// `WezTerm` terminal emulator (TOML)
    WezTerm,
    /// `iTerm2` terminal emulator (JSON/Plist)
    ITerm2,
    ///`Kitty` terminal emulator (Config format)
    Kitty,
    /// `Konsole` terminal emulator (colorscheme format)
    Konsole,
    /// `Mintty` terminal emulator (INI format)
    Mintty,
    /// `Windows Terminal` (JSON)
    WindowsTerminal,
}

impl ExportFormat {
    /// Get all available export formats
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Alacritty,
            Self::WezTerm,
            Self::ITerm2,
            Self::Kitty,
            Self::Konsole,
            Self::Mintty,
            Self::WindowsTerminal,
        ]
    }

    /// Get the file extension for this format
    #[must_use]
    pub fn file_extension(self) -> &'static str {
        match self {
            Self::Alacritty => alacritty::AlacrittyExporter::file_extension(),
            Self::WezTerm => wezterm::WezTermExporter::file_extension(),
            Self::ITerm2 => iterm2::ITerm2Exporter::file_extension(),
            Self::Kitty => kitty::KittyExporter::file_extension(),
            Self::Konsole => konsole::KonsoleExporter::file_extension(),
            Self::Mintty => mintty::MinttyExporter::file_extension(),
            Self::WindowsTerminal => windows_terminal::WindowsTerminalExporter::file_extension(),
        }
    }

    /// Get the format name
    #[must_use]
    pub fn format_name(self) -> &'static str {
        match self {
            Self::Alacritty => alacritty::AlacrittyExporter::format_name(),
            Self::WezTerm => wezterm::WezTermExporter::format_name(),
            Self::ITerm2 => iterm2::ITerm2Exporter::format_name(),
            Self::Kitty => kitty::KittyExporter::format_name(),
            Self::Konsole => konsole::KonsoleExporter::format_name(),
            Self::Mintty => mintty::MinttyExporter::format_name(),
            Self::WindowsTerminal => windows_terminal::WindowsTerminalExporter::format_name(),
        }
    }

    /// Export a theme using this format
    ///
    /// # Errors
    ///
    /// Will bubble up any i/o  errors encountered.
    pub fn export_theme(self, theme: &Theme) -> StylingResult<String> {
        match self {
            Self::Alacritty => alacritty::AlacrittyExporter::export_theme(theme),
            Self::WezTerm => wezterm::WezTermExporter::export_theme(theme),
            Self::ITerm2 => iterm2::ITerm2Exporter::export_theme(theme),
            Self::Kitty => kitty::KittyExporter::export_theme(theme),
            Self::Konsole => konsole::KonsoleExporter::export_theme(theme),
            Self::Mintty => mintty::MinttyExporter::export_theme(theme),
            Self::WindowsTerminal => windows_terminal::WindowsTerminalExporter::export_theme(theme),
        }
    }
}

/// Export a theme to all supported formats and save to files in the specified directory
///
/// # Arguments
/// * `theme` - The theme to export
/// * `output_dir` - Directory to save the exported theme files
/// * `base_filename` - Base filename (without extension) for the exported files
///
/// # Returns
/// A vector of successfully exported file paths
///
/// # Errors
///
/// Will bubble up any i/o  errors encountered.
pub fn export_all_formats<P: AsRef<Path>>(
    theme: &Theme,
    output_dir: P,
    base_filename: &str,
) -> StylingResult<Vec<std::path::PathBuf>> {
    let output_dir = output_dir.as_ref();
    let mut exported_files = Vec::new();

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir).map_err(StylingError::Io)?;

    for format in ExportFormat::all() {
        match format.export_theme(theme) {
            Ok(content) => {
                // Handle filename conflicts between formats with same extension
                let filename = match format {
                    ExportFormat::Alacritty => {
                        format!("{}_alacritty.{}", base_filename, format.file_extension())
                    }
                    ExportFormat::WezTerm => {
                        format!("{}_wezterm.{}", base_filename, format.file_extension())
                    }
                    ExportFormat::WindowsTerminal => format!(
                        "{}_windows_terminal.{}",
                        base_filename,
                        format.file_extension()
                    ),
                    ExportFormat::Mintty => {
                        if format.file_extension().is_empty() {
                            format!("{}_mintty", base_filename)
                        } else {
                            format!("{}_mintty.{}", base_filename, format.file_extension())
                        }
                    }
                    _ => format!("{}.{}", base_filename, format.file_extension()),
                };
                let file_path = output_dir.join(filename);

                match std::fs::write(&file_path, content) {
                    Ok(()) => {
                        exported_files.push(file_path);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to write {} theme file: {}",
                            format.format_name(),
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to export {} theme: {}",
                    format.format_name(),
                    e
                );
            }
        }
    }

    Ok(exported_files)
}

/// Export a single theme to a specific format and save to file
///
/// # Arguments
/// * `theme` - The theme to export
/// * `format` - The target export format
/// * `output_path` - Path to save the exported theme file
///
/// # Errors
///
/// Will bubble up any i/o errors encountered.
pub fn export_theme_to_file<P: AsRef<Path>>(
    theme: &Theme,
    format: ExportFormat,
    output_path: P,
) -> StylingResult<()> {
    let content = format.export_theme(theme)?;
    let path = output_path.as_ref();
    std::fs::write(path, content).map_err(StylingError::Io)?;

    // Convert iTerm2 XML plists to binary format for compatibility
    #[cfg(target_os = "macos")]
    if matches!(format, ExportFormat::ITerm2) {
        convert_xml_plist_to_binary(path)?;
    }

    Ok(())
}

/// Convert an XML plist file to binary format using macOS plutil
#[cfg(target_os = "macos")]
fn convert_xml_plist_to_binary<P: AsRef<Path>>(path: P) -> StylingResult<()> {
    use std::process::Command;

    let output = Command::new("plutil")
        .arg("-convert")
        .arg("binary1")
        .arg(path.as_ref())
        .output()
        .map_err(StylingError::Io)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(StylingError::FromStr(format!(
            "plutil conversion failed: {}",
            error_msg
        )));
    }

    Ok(())
}

/// Generate installation instructions for a specific format
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn generate_installation_instructions(format: ExportFormat, theme_filename: &str) -> String {
    match format {
        ExportFormat::Alacritty => {
            format!(
                r#"# Alacritty Theme Installation

To use this theme with Alacritty:

Programmatic installation: Run `thag_alacritty_add_theme` to select and install the theme.

Manual installation:

    1. Copy the generated terminal theme file to your Alacritty config directory:
    - Linux/macOS: `~/.config/alacritty/themes/.`
    - Windows: `%APPDATA%\alacritty\themes\.`

    2. Add this to your alacritty.yml or alacritty.toml:
    ```yaml
    general:
        import:
        - 'themes/{theme_filename}'

    ```
    ```toml
    general.import = ["themes/{theme_filename}"]
    ```

Restart Alacritty to apply the theme.
"#
            )
        }
        ExportFormat::Kitty => {
            format!(
                r"# Kitty Theme Installation

To use this theme with Kitty:

1. Copy the theme file to your Kitty config directory:
   - Linux/macOS: `~/.config/kitty/themes/.`
   - Windows: `%APPDATA%\kitty\themes\.`

2. Add this to your kitty.conf:
   ```
   include themes/{theme_filename}
   ```

3. Reload Kitty config with Ctrl+Shift+F5 or restart Kitty.
"
            )
        }
        ExportFormat::Mintty => {
            format!(
                r#"# Mintty (Git Bash) Theme Installation

To use this theme with Mintty (Windows: Git Bash or Cygwin):

Programmatic installation: Run `thag_mintty_add_theme` to select and install the theme.

Manual installation:

    1. Copy the generated terminal theme file to your Mintty themes directory:
   - Git Bash: `C:\Program Files\Git\usr\share\mintty\themes\`
   - Cygwin: `C:\Program Files\Git\usr\share\mintty\themes\`
   - Note: You may need administrator privileges

    2. Option A - Using the theme chooser:
    - Open Git Bash
    - Right-click on the title bar and select "Options..."
    - Go to "Looks" tab
    - Select your theme from the "Theme" dropdown

    3. Option B - Manual configuration:
    - Edit ~/.minttyrc and add: `ThemeFile={theme_filename}`
    - Restart Git Bash

The theme will be applied to all new Mintty windows.
"#
            )
        }
        ExportFormat::WezTerm => {
            format!(
                r"# WezTerm Theme Installation

To use this theme with WezTerm:

1. Copy the theme file to your WezTerm config directory:
   - Linux/macOS: `~/.config/wezterm/colors/.`
   - Windows: `%USERPROFILE%\.config\wezterm\colors\.`

2. Add this to your wezterm.lua config file:
   ```lua
   local config = wezterm.config_builder()
   config.color_scheme = '{theme_filename}'
   return config
   ```

3. Restart WezTerm to apply the theme.

Note: The theme name should match the filename without extension.
WezTerm will use this TOML file format, which is different from Alacritty's TOML structure.
"
            )
        }
        ExportFormat::ITerm2 => {
            format!(
                r#"# iTerm2 Theme Installation

To use this theme with iTerm2:

1. Open iTerm2
2. Go to Preferences > Profiles > Colors
3. Click "Color Presets..." dropdown
4. Select "Import..."
5. Choose the {theme_filename} file
6. Select the imported theme from the dropdown
7. For clear selected text, select contrasting colors for the Selection:Background and Foreground from
   their respective color wells and color pickers. Thag suggests using colours from the theme's palette
   for aesthetic reasons.

The theme will be applied to your current profile.

"#
            )
        }
        ExportFormat::WindowsTerminal => {
            format!(
                r#"# Windows Terminal Theme Installation

To use this theme with Windows Terminal:

TODO: Formalise: Run `thag demo/windows_terminal_add_theme.rs <json`
1. Open Windows Terminal
2. Open Settings (Ctrl+,)
3. Go to "Open JSON file" in the bottom left
4. Copy the color scheme from {theme_filename} into the "schemes" array
5. Add "colorScheme": "{theme_filename}" to your profile settings

Alternatively, you can merge the JSON content directly into your settings.json file.
"#
            )
        }
        ExportFormat::Konsole => {
            format!(
                r#"# Konsole Theme Installation

To use this theme with Konsole:

1. Copy the theme file to your Konsole color schemes directory:
   - Linux: `~/.local/share/konsole/` or `/usr/share/konsole/`,
     or for a Flatpak installation: `~/.var/app/org.kde.konsole/data/konsole/`.
     If in doubt, create a new profile from the Konsole settings menu,
     and use `find` with `-name "*.profile"` to find the directory.
   - The file should have a `.colorscheme` extension

2. Option A - Using Konsole Settings:
   - Open Konsole
   - Go to Settings > Edit Current Profile...
   - Click on "Appearance" tab
   - Select your theme from the "Color scheme" dropdown

3. Option B - Command line:
   - Use: `konsoleprofile ColorScheme={theme_filename}`
   - Note: Use the filename without the .colorscheme extension

4. Option C - For persistent changes:
   - Edit your profile configuration in `~/.local/share/konsole/`
   - Add `ColorScheme={theme_filename}` to the profile file

The theme will be applied immediately to new Konsole sessions.
"#
            )
        }
    }
}

/// Brighten a color by increasing its components
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, dead_code)]
fn brighten_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    adjust_color_brightness((r, g, b), 1.3)
}

/// Dim a color by reducing its components
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn dim_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    adjust_color_brightness((r, g, b), 0.6)
}

/// Adjust color brightness by a factor
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn adjust_color_brightness((r, g, b): (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    // For very dark colors, use additive brightening to ensure visibility
    if r < 50 && g < 50 && b < 50 && factor > 1.0 {
        // Add a minimum brightness boost for very dark backgrounds
        let min_boost = 80.0;
        (
            f32::from(r).mul_add(factor, min_boost).clamp(0.0, 255.0) as u8,
            f32::from(g).mul_add(factor, min_boost).clamp(0.0, 255.0) as u8,
            f32::from(b).mul_add(factor, min_boost).clamp(0.0, 255.0) as u8,
        )
    } else {
        // Use multiplicative for normal colors
        (
            (f32::from(r) * factor).clamp(0.0, 255.0) as u8,
            (f32::from(g) * factor).clamp(0.0, 255.0) as u8,
            (f32::from(b) * factor).clamp(0.0, 255.0) as u8,
        )
    }
}

// /// Get the best dark color from the theme for black mapping
// fn get_best_dark_color(theme: &Theme) -> Option<(u8, u8, u8)> {
//     // Try background first, then subtle, then create a dark color
//     theme
//         .bg_rgbs
//         .first()
//         .copied()
//         .or_else(|| &theme.palette.subtle).rgb()
//         .or(Some((16, 16, 16)))
// }

// /// Extract RGB values from a Style's foreground color
// fn get_rgb_from_style(style: &crate::Style) -> Option<(u8, u8, u8)> {
//     style.foreground.as_ref().map(|color_info| {
//         match &color_info.value {
//             ColorValue::TrueColor { rgb } => (rgb[0], rgb[1], rgb[2]),
//             ColorValue::Color256 { color256 } => {
//                 // Convert 256-color index to approximate RGB
//                 color_256_to_rgb(*color256)
//             }
//             ColorValue::Basic { index, .. } => {
//                 // Convert basic color index to RGB
//                 basic_color_to_rgb(*index)
//             }
//         }
//     })
// }

/// Convert basic color index to RGB
#[allow(clippy::match_same_arms)]
const fn basic_color_to_rgb(index: u8) -> (u8, u8, u8) {
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

/// Convert 256-color index to RGB
const fn color_256_to_rgb(index: u8) -> (u8, u8, u8) {
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

/// Check if a color is considered light
fn is_light_color((r, g, b): (u8, u8, u8)) -> bool {
    // Calculate relative luminance
    let r_linear = if r <= 10 {
        f32::from(r) / 3294.6
    } else {
        ((f32::from(r) + 14.025) / 269.025).powf(2.4)
    };
    let g_linear = if g <= 10 {
        f32::from(g) / 3294.6
    } else {
        ((f32::from(g) + 14.025) / 269.025).powf(2.4)
    };
    let b_linear = if b <= 10 {
        f32::from(b) / 3294.6
    } else {
        ((f32::from(b) + 14.025) / 269.025).powf(2.4)
    };

    let luminance = 0.0722f32.mul_add(b_linear, 0.2126f32.mul_add(r_linear, 0.7152 * g_linear));
    luminance > 0.5
}

#[cfg(test)]
fn create_test_theme() -> Theme {
    use crate::{ColorInfo, ColorSupport, Palette, Style, TermBgLuma};
    use std::path::PathBuf;

    let palette = Palette {
        normal: Style::fg(ColorInfo::rgb(220, 220, 220)),
        error: Style::fg(ColorInfo::rgb(255, 100, 100)),
        heading1: Style::fg(ColorInfo::rgb(55, 85, 206)),
        heading2: Style::fg(ColorInfo::rgb(106, 138, 203)),
        heading3: Style::fg(ColorInfo::rgb(255, 100, 100)),
        warning: Style::fg(ColorInfo::rgb(168, 120, 48)),
        success: Style::fg(ColorInfo::rgb(170, 170, 170)),
        info: Style::fg(ColorInfo::rgb(118, 157, 216)),
        emphasis: Style::fg(ColorInfo::rgb(201, 147, 66)),
        code: Style::fg(ColorInfo::rgb(66, 120, 201)),
        subtle: Style::fg(ColorInfo::rgb(144, 144, 144)),
        hint: Style::fg(ColorInfo::rgb(144, 144, 144)),
        debug: Style::fg(ColorInfo::rgb(144, 144, 144)),
        link: Style::fg(ColorInfo::rgb(160, 160, 160)),
        quote: Style::fg(ColorInfo::rgb(178, 161, 161)),
        commentary: Style::fg(ColorInfo::rgb(120, 98, 64)),
    };

    Theme {
        name: "Test Theme".to_string(),
        filename: PathBuf::from("test.toml"),
        is_builtin: false,
        term_bg_luma: TermBgLuma::Dark,
        min_color_support: ColorSupport::TrueColor,
        palette,
        backgrounds: vec!["#1e1e2e".to_string()],
        bg_rgbs: vec![(30, 30, 46)],
        description: "A test theme".to_string(),
        base_colors: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_formats() {
        let theme = create_test_theme();

        for format in ExportFormat::all() {
            let result = format.export_theme(&theme);
            assert!(
                result.is_ok(),
                "Failed to export {} format",
                format.format_name()
            );

            let content = result.unwrap();
            assert!(
                !content.is_empty(),
                "{} export produced empty content",
                format.format_name()
            );
        }
    }

    #[test]
    fn test_file_extensions() {
        assert_eq!(ExportFormat::Alacritty.file_extension(), "toml");
        assert_eq!(ExportFormat::WezTerm.file_extension(), "toml");
        assert_eq!(ExportFormat::ITerm2.file_extension(), "itermcolors");
        assert_eq!(ExportFormat::Kitty.file_extension(), "conf");
        assert_eq!(ExportFormat::Mintty.file_extension(), "");
        assert_eq!(ExportFormat::WindowsTerminal.file_extension(), "json");
    }

    #[test]
    fn test_installation_instructions() {
        for format in ExportFormat::all() {
            let instructions = generate_installation_instructions(*format, "test_theme.ext");
            assert!(!instructions.is_empty());
            assert!(instructions.contains(format.format_name()));
        }
    }
}
