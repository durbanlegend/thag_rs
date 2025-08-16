//! Multi-format terminal theme exporters for converting thag themes to various terminal emulator formats
//!
//! This module provides functionality to export thag themes to popular terminal emulator formats,
//! enabling users to apply generated themes across different terminal applications.

use crate::{StylingError, StylingResult, Theme};
use std::path::Path;

pub mod alacritty;
pub mod iterm2;
pub mod kitty;
pub mod wezterm;
pub mod windows_terminal;

/// Trait for exporting themes to different terminal emulator formats
pub trait ThemeExporter {
    /// Export a theme to the format-specific string representation
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
    /// WezTerm terminal emulator (TOML)
    WezTerm,
    /// iTerm2 terminal emulator (JSON/Plist)
    ITerm2,
    /// Kitty terminal emulator (Config format)
    Kitty,
    /// Windows Terminal (JSON)
    WindowsTerminal,
}

impl ExportFormat {
    /// Get all available export formats
    pub fn all() -> &'static [ExportFormat] {
        &[
            ExportFormat::Alacritty,
            ExportFormat::WezTerm,
            ExportFormat::ITerm2,
            ExportFormat::Kitty,
            ExportFormat::WindowsTerminal,
        ]
    }

    /// Get the file extension for this format
    pub fn file_extension(self) -> &'static str {
        match self {
            ExportFormat::Alacritty => alacritty::AlacrittyExporter::file_extension(),
            ExportFormat::WezTerm => wezterm::WezTermExporter::file_extension(),
            ExportFormat::ITerm2 => iterm2::ITerm2Exporter::file_extension(),
            ExportFormat::Kitty => kitty::KittyExporter::file_extension(),
            ExportFormat::WindowsTerminal => {
                windows_terminal::WindowsTerminalExporter::file_extension()
            }
        }
    }

    /// Get the format name
    pub fn format_name(self) -> &'static str {
        match self {
            ExportFormat::Alacritty => alacritty::AlacrittyExporter::format_name(),
            ExportFormat::WezTerm => wezterm::WezTermExporter::format_name(),
            ExportFormat::ITerm2 => iterm2::ITerm2Exporter::format_name(),
            ExportFormat::Kitty => kitty::KittyExporter::format_name(),
            ExportFormat::WindowsTerminal => {
                windows_terminal::WindowsTerminalExporter::format_name()
            }
        }
    }

    /// Export a theme using this format
    pub fn export_theme(self, theme: &Theme) -> StylingResult<String> {
        match self {
            ExportFormat::Alacritty => alacritty::AlacrittyExporter::export_theme(theme),
            ExportFormat::WezTerm => wezterm::WezTermExporter::export_theme(theme),
            ExportFormat::ITerm2 => iterm2::ITerm2Exporter::export_theme(theme),
            ExportFormat::Kitty => kitty::KittyExporter::export_theme(theme),
            ExportFormat::WindowsTerminal => {
                windows_terminal::WindowsTerminalExporter::export_theme(theme)
            }
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
pub fn export_theme_to_file<P: AsRef<Path>>(
    theme: &Theme,
    format: ExportFormat,
    output_path: P,
) -> StylingResult<()> {
    let content = format.export_theme(theme)?;
    std::fs::write(output_path, content).map_err(StylingError::Io)?;
    Ok(())
}

/// Generate installation instructions for a specific format
pub fn generate_installation_instructions(format: ExportFormat, theme_filename: &str) -> String {
    match format {
        ExportFormat::Alacritty => {
            let alacritty_filename = theme_filename.replace(".toml", "_alacritty.toml");
            format!(
                r#"# Alacritty Theme Installation

To use this theme with Alacritty:

1. Copy the theme file to your Alacritty config directory:
   - Linux/macOS: `~/.config/alacritty/themes/{}`
   - Windows: `%APPDATA%\alacritty\themes\{}`

2. Add this to your alacritty.yml or alacritty.toml:
   ```toml
   general.import = ["themes/{}"]
   ```

3. Restart Alacritty to apply the theme.
"#,
                alacritty_filename, alacritty_filename, alacritty_filename
            )
        }
        ExportFormat::WezTerm => {
            let wezterm_filename = theme_filename.replace(".toml", "_wezterm.toml");
            format!(
                r#"# WezTerm Theme Installation

To use this theme with WezTerm:

1. Copy the theme file to your WezTerm config directory:
   - Linux/macOS: `~/.config/wezterm/colors/{}`
   - Windows: `%USERPROFILE%\.config\wezterm\colors\{}`

2. Add this to your wezterm.lua config file:
   ```lua
   local config = wezterm.config_builder()
   config.color_scheme = '{}'
   return config
   ```

3. Restart WezTerm to apply the theme.

Note: The theme name should match the filename without extension.
WezTerm will use this TOML file format, which is different from Alacritty's TOML structure.
"#,
                wezterm_filename,
                wezterm_filename,
                wezterm_filename.trim_end_matches(".toml")
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
5. Choose the {} file
6. Select the imported theme from the dropdown

The theme will be applied to your current profile.
"#,
                theme_filename
            )
        }
        ExportFormat::Kitty => {
            format!(
                r#"# Kitty Theme Installation

To use this theme with Kitty:

1. Copy the theme file to your Kitty config directory:
   - Linux/macOS: `~/.config/kitty/themes/{}`
   - Windows: `%APPDATA%\kitty\themes\{}`

2. Add this to your kitty.conf:
   ```
   include themes/{}
   ```

3. Reload Kitty config with Ctrl+Shift+F5 or restart Kitty.
"#,
                theme_filename, theme_filename, theme_filename
            )
        }
        ExportFormat::WindowsTerminal => {
            let wt_filename = theme_filename.replace(".json", "_windows_terminal.json");
            format!(
                r#"# Windows Terminal Theme Installation

To use this theme with Windows Terminal:

1. Open Windows Terminal
2. Open Settings (Ctrl+,)
3. Go to "Open JSON file" in the bottom left
4. Copy the color scheme from {} into the "schemes" array
5. Add "colorScheme": "{}" to your profile settings

Alternatively, you can merge the JSON content directly into your settings.json file.
"#,
                wt_filename,
                wt_filename.trim_end_matches("_windows_terminal.json")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSupport, Palette, Role, Style, TermBgLuma};
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
            description: "A test theme for unit testing".to_string(),
        }
    }

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
        assert_eq!(ExportFormat::ITerm2.file_extension(), "json");
        assert_eq!(ExportFormat::Kitty.file_extension(), "conf");
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
