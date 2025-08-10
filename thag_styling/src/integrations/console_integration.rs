//! Integration with the console terminal styling library
//!
//! This module provides seamless integration between thag's theming system
//! and the console library's styling types, enabling console-based applications
//! to use theme-aware colors automatically.

use crate::integrations::ThemedStyle;
use crate::{ColorInfo, ColorValue, Role, Style};
use console::{Color as ConsoleColor, Style as ConsoleStyle, Term};

impl ThemedStyle<ConsoleStyle> for ConsoleStyle {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from_thag_style(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        let mut console_style = Self::new();

        // Apply foreground color
        if let Some(color_info) = &style.foreground {
            console_style = console_style.fg(ConsoleColor::from(color_info));
        }

        // Apply background color
        if let Some(color_info) = &style.background {
            console_style = console_style.bg(ConsoleColor::from(color_info));
        }

        // Apply style attributes
        if style.bold {
            console_style = console_style.bold();
        }
        if style.italic {
            console_style = console_style.italic();
        }
        if style.dim {
            console_style = console_style.dim();
        }
        if style.underline {
            console_style = console_style.underlined();
        }
        // Note: console doesn't support strikethrough

        console_style
    }
}

impl ThemedStyle<ConsoleColor> for ConsoleColor {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        thag_style
            .foreground
            .as_ref()
            .map(Self::from)
            .unwrap_or_else(|| Self::Color256(u8::from(&role)))
    }

    fn from_thag_style(style: &Style) -> Self {
        style
            .foreground
            .as_ref()
            .map(Self::from)
            .unwrap_or(Self::White) // Default to white
    }
}

/// Convert ColorInfo to console Color
impl From<&ColorInfo> for ConsoleColor {
    fn from(color_info: &ColorInfo) -> Self {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Self::TrueColor {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
            },
            ColorValue::Color256 { color256 } => Self::Color256(*color256),
            ColorValue::Basic { .. } => {
                // Map basic colors to console's named colors
                match color_info.index {
                    0 => Self::Black,
                    1 => Self::Red,
                    2 => Self::Green,
                    3 => Self::Yellow,
                    4 => Self::Blue,
                    5 => Self::Magenta,
                    6 => Self::Cyan,
                    7 => Self::White,
                    _ => Self::Color256(color_info.index),
                }
            }
        }
    }
}

/// Legacy From implementations for backward compatibility
impl From<&Style> for ConsoleStyle {
    fn from(style: &Style) -> Self {
        Self::from_thag_style(style)
    }
}

impl From<&Role> for ConsoleColor {
    fn from(role: &Role) -> Self {
        Self::themed(*role)
    }
}

impl From<Role> for ConsoleColor {
    fn from(role: Role) -> Self {
        Self::themed(role)
    }
}

impl From<&Role> for ConsoleStyle {
    fn from(role: &Role) -> Self {
        Self::themed(*role)
    }
}

impl From<Role> for ConsoleStyle {
    fn from(role: Role) -> Self {
        Self::themed(role)
    }
}

/// Convenience methods for console styling
pub trait ConsoleStyleExt {
    /// Apply a thag role to this console style
    fn with_role(self, role: Role) -> Self;

    /// Apply a thag style to this console style
    fn with_thag_style(self, style: &Style) -> Self;
}

impl ConsoleStyleExt for ConsoleStyle {
    fn with_role(self, role: Role) -> Self {
        let themed = Self::themed(role);
        // Console styles are immutable, so we need to combine them
        let mut combined = self;

        // Apply themed foreground if not already set
        if let Some(fg) = themed.get_fg() {
            combined = combined.fg(fg);
        }

        // Apply themed background if not already set
        if let Some(bg) = themed.get_bg() {
            combined = combined.bg(bg);
        }

        // Combine attributes
        if themed.get_bold() {
            combined = combined.bold();
        }
        if themed.get_italic() {
            combined = combined.italic();
        }
        if themed.get_dim() {
            combined = combined.dim();
        }
        if themed.get_underlined() {
            combined = combined.underlined();
        }

        combined
    }

    fn with_thag_style(self, style: &Style) -> Self {
        let themed = Self::from_thag_style(style);
        self.with_role(Role::Normal) // Use the style conversion logic
    }
}

/// Helper functions for common console operations
pub mod console_helpers {
    use super::*;
    use console::Term;

    /// Print themed content to stdout
    pub fn print_themed(role: Role, content: &str) -> console::Result<()> {
        let style = ConsoleStyle::themed(role);
        let term = Term::stdout();
        term.write_line(&style.apply_to(content).to_string())
    }

    /// Print themed content to stderr
    pub fn eprint_themed(role: Role, content: &str) -> console::Result<()> {
        let style = ConsoleStyle::themed(role);
        let term = Term::stderr();
        term.write_line(&style.apply_to(content).to_string())
    }

    /// Create themed styles for common UI elements
    pub fn success_style() -> ConsoleStyle {
        ConsoleStyle::themed(Role::Success)
    }

    pub fn error_style() -> ConsoleStyle {
        ConsoleStyle::themed(Role::Error)
    }

    pub fn warning_style() -> ConsoleStyle {
        ConsoleStyle::themed(Role::Warning)
    }

    pub fn info_style() -> ConsoleStyle {
        ConsoleStyle::themed(Role::Info)
    }

    pub fn code_style() -> ConsoleStyle {
        ConsoleStyle::themed(Role::Code)
    }

    pub fn emphasis_style() -> ConsoleStyle {
        ConsoleStyle::themed(Role::Emphasis)
    }

    pub fn subtle_style() -> ConsoleStyle {
        ConsoleStyle::themed(Role::Subtle)
    }

    /// Get a themed console Term with appropriate settings
    pub fn themed_term() -> Term {
        let term = Term::stdout();
        // Could potentially configure term based on theme here
        term
    }
}

/// Extension trait for console's Term to support themed output
pub trait TermThemedExt {
    /// Write themed content
    fn write_themed(&self, role: Role, content: &str) -> console::Result<()>;

    /// Write themed line
    fn write_line_themed(&self, role: Role, content: &str) -> console::Result<()>;
}

impl TermThemedExt for Term {
    fn write_themed(&self, role: Role, content: &str) -> console::Result<()> {
        let style = ConsoleStyle::themed(role);
        self.write_str(&style.apply_to(content).to_string())
    }

    fn write_line_themed(&self, role: Role, content: &str) -> console::Result<()> {
        let style = ConsoleStyle::themed(role);
        self.write_line(&style.apply_to(content).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themed_style_creation() {
        let style = ConsoleStyle::themed(Role::Error);
        // Should have some styling applied
        assert_ne!(format!("{:?}", style), format!("{:?}", ConsoleStyle::new()));
    }

    #[test]
    fn test_themed_color_creation() {
        let color = ConsoleColor::themed(Role::Success);
        // Should be a valid color
        match color {
            ConsoleColor::Color256(_)
            | ConsoleColor::TrueColor { .. }
            | ConsoleColor::Red
            | ConsoleColor::Green
            | ConsoleColor::Blue
            | ConsoleColor::Yellow
            | ConsoleColor::Magenta
            | ConsoleColor::Cyan
            | ConsoleColor::White
            | ConsoleColor::Black => (),
            _ => panic!("Unexpected color type: {:?}", color),
        }
    }

    #[test]
    fn test_style_extension() {
        let base_style = ConsoleStyle::new().bold();
        let themed_style = base_style.with_role(Role::Warning);

        // Should preserve the bold attribute
        assert!(themed_style.get_bold());
    }

    #[test]
    fn test_color_conversion() {
        let color_info = ColorInfo {
            index: 42,
            value: ColorValue::Color256 { color256: 42 },
            hex: "#ff0000".to_string(),
            name: "test".to_string(),
        };

        let console_color = ConsoleColor::from(&color_info);
        assert_eq!(console_color, ConsoleColor::Color256(42));
    }

    #[test]
    fn test_helper_functions() {
        let success = console_helpers::success_style();
        let error = console_helpers::error_style();
        let warning = console_helpers::warning_style();

        // All should be different styles (at least different roles)
        // Note: We can't directly compare ConsoleStyle, so we test that they exist
        assert!(
            success.get_fg().is_some()
                || success.get_bg().is_some()
                || success.get_bold()
                || success.get_italic()
        );
        assert!(
            error.get_fg().is_some()
                || error.get_bg().is_some()
                || error.get_bold()
                || error.get_italic()
        );
        assert!(
            warning.get_fg().is_some()
                || warning.get_bg().is_some()
                || warning.get_bold()
                || warning.get_italic()
        );
    }

    #[test]
    fn test_basic_color_mapping() {
        let red_info = ColorInfo {
            index: 1,
            value: ColorValue::Basic { basic: 1 },
            hex: "#ff0000".to_string(),
            name: "red".to_string(),
        };

        let console_color = ConsoleColor::from(&red_info);
        assert_eq!(console_color, ConsoleColor::Red);
    }
}
