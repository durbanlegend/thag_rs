//! Integration with the crossterm cross-platform terminal library
//!
//! This module provides seamless integration between thag's theming system
//! and crossterm's styling types, enabling crossterm-based applications
//! to use theme-aware colors automatically.

use crate::ThemedStyle;
use crate::{ColorInfo, ColorValue, Role, Style};
use crossterm::style::{Attribute, Color as CrossColor, ContentStyle, SetAttribute};

impl ThemedStyle<Self> for ContentStyle {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from_thag_style(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        let mut content_style = Self::default();

        // Apply foreground color
        if let Some(color_info) = &style.foreground {
            content_style.foreground_color = Some(CrossColor::from(color_info));
        }

        // Apply style attributes
        let mut attributes = Vec::new();
        if style.bold {
            attributes.push(Attribute::Bold);
        }
        if style.italic {
            attributes.push(Attribute::Italic);
        }
        if style.dim {
            attributes.push(Attribute::Dim);
        }
        if style.underline {
            attributes.push(Attribute::Underlined);
        }

        for attr in attributes {
            content_style.attributes.set(attr);
        }
        content_style
    }
}

impl ThemedStyle<Self> for CrossColor {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        thag_style
            .foreground
            .as_ref()
            .map_or_else(|| Self::AnsiValue(u8::from(&role)), Self::from)
    }

    fn from_thag_style(style: &Style) -> Self {
        style.foreground.as_ref().map_or(Self::Reset, Self::from)
    }
}

/// Convert `ColorInfo` to `crossterm` Color
impl From<&ColorInfo> for CrossColor {
    fn from(color_info: &ColorInfo) -> Self {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Self::Rgb {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
            },
            ColorValue::Color256 { color256 } => Self::AnsiValue(*color256),
            ColorValue::Basic { .. } => Self::AnsiValue(color_info.index),
        }
    }
}

/// Legacy From implementations for backward compatibility
impl From<&Style> for ContentStyle {
    fn from(style: &Style) -> Self {
        Self::from_thag_style(style)
    }
}

impl From<&Role> for CrossColor {
    fn from(role: &Role) -> Self {
        Self::themed(*role)
    }
}

impl From<Role> for CrossColor {
    fn from(role: Role) -> Self {
        Self::themed(role)
    }
}

impl From<&Role> for ContentStyle {
    fn from(role: &Role) -> Self {
        Self::themed(*role)
    }
}

impl From<Role> for ContentStyle {
    fn from(role: Role) -> Self {
        Self::themed(role)
    }
}

/// Convenience methods for crossterm styling
pub trait CrosstermStyleExt {
    /// Apply a thag role to this crossterm style
    #[must_use]
    fn with_role(self, role: Role) -> Self;

    /// Apply a thag style to this crossterm style
    #[must_use]
    fn with_thag_style(self, style: &Style) -> Self;
}

impl CrosstermStyleExt for ContentStyle {
    fn with_role(self, role: Role) -> Self {
        let themed = Self::themed(role);
        let mut combined = self;
        if themed.foreground_color.is_some() {
            combined.foreground_color = themed.foreground_color;
        }
        if themed.background_color.is_some() {
            combined.background_color = themed.background_color;
        }
        if themed.underline_color.is_some() {
            combined.underline_color = themed.underline_color;
        }
        combined.attributes = themed.attributes;
        combined
    }

    fn with_thag_style(self, style: &Style) -> Self {
        let themed = Self::from_thag_style(style);
        let mut combined = self;
        if themed.foreground_color.is_some() {
            combined.foreground_color = themed.foreground_color;
        }
        if themed.background_color.is_some() {
            combined.background_color = themed.background_color;
        }
        if themed.underline_color.is_some() {
            combined.underline_color = themed.underline_color;
        }
        combined.attributes = themed.attributes;
        combined
    }
}

/// Helper functions for common crossterm operations
pub mod crossterm_helpers {
    use super::{Attribute, ContentStyle, CrossColor, Role, SetAttribute, ThemedStyle};

    use crossterm::QueueableCommand;
    use std::io::{self, Write};

    /// Queue a themed print command
    ///
    /// # Errors
    ///
    /// This function will bubble up any `crossterm` error writing to the terminal.
    pub fn queue_themed_print<'a, W: Write>(
        writer: &'a mut W,
        role: Role,
        content: &str,
    ) -> io::Result<&'a mut W> {
        let color = CrossColor::themed(role);
        writer
            .queue(crossterm::style::SetForegroundColor(color))?
            .queue(crossterm::style::Print(content))
    }

    /// Queue a styled print command using a `ContentStyle`
    ///
    /// # Errors
    ///
    /// This function will bubble up any `crossterm` error writing to the terminal.
    pub fn queue_styled_print<'a, W: Write>(
        writer: &'a mut W,
        style: &ContentStyle,
        content: &str,
    ) -> io::Result<&'a mut W> {
        if let Some(fg) = style.foreground_color {
            writer.queue(crossterm::style::SetForegroundColor(fg))?;
        }

        // Apply attributes one by one since Attributes doesn't implement IntoIterator
        if style.attributes.has(Attribute::Bold) {
            writer.queue(SetAttribute(Attribute::Bold))?;
        }
        if style.attributes.has(Attribute::Italic) {
            writer.queue(SetAttribute(Attribute::Italic))?;
        }
        if style.attributes.has(Attribute::Dim) {
            writer.queue(SetAttribute(Attribute::Dim))?;
        }
        if style.attributes.has(Attribute::Underlined) {
            writer.queue(SetAttribute(Attribute::Underlined))?;
        }

        writer.queue(crossterm::style::Print(content))
    }

    /// Create themed styles for common UI elements
    #[must_use]
    pub fn success_style() -> ContentStyle {
        ContentStyle::themed(Role::Success)
    }

    /// Create themed style for error messages
    #[must_use]
    pub fn error_style() -> ContentStyle {
        ContentStyle::themed(Role::Error)
    }

    /// Create themed style for warning messages
    #[must_use]
    pub fn warning_style() -> ContentStyle {
        ContentStyle::themed(Role::Warning)
    }

    /// Create themed style for informational messages
    #[must_use]
    pub fn info_style() -> ContentStyle {
        ContentStyle::themed(Role::Info)
    }

    /// Create themed style for code content
    #[must_use]
    pub fn code_style() -> ContentStyle {
        ContentStyle::themed(Role::Code)
    }

    /// Create themed style for emphasized text
    #[must_use]
    pub fn emphasis_style() -> ContentStyle {
        ContentStyle::themed(Role::Emphasis)
    }

    /// Create themed style for subtle/less important text
    #[must_use]
    pub fn subtle_style() -> ContentStyle {
        ContentStyle::themed(Role::Subtle)
    }
}

/// Extension trait for crossterm's Stylize trait compatibility
pub trait ThemedStylize {
    /// Apply a thag role as styling
    fn role(self, role: Role) -> crossterm::style::StyledContent<Self>
    where
        Self: Sized + Clone + std::fmt::Display;
}

impl<T> ThemedStylize for T
where
    T: Clone + std::fmt::Display,
{
    fn role(self, role: Role) -> crossterm::style::StyledContent<Self> {
        let style = ContentStyle::themed(role);
        crossterm::style::StyledContent::new(style, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Stylize;

    #[test]
    fn test_themed_content_style_creation() {
        let style = ContentStyle::themed(Role::Error);
        // Should have some styling applied
        assert_ne!(style, ContentStyle::default());
    }

    #[test]
    fn test_themed_color_creation() {
        let color = CrossColor::themed(Role::Success);
        // Should not be the reset color
        assert_ne!(color, CrossColor::Reset);
    }

    #[test]
    fn test_style_extension() {
        let base_style = ContentStyle::default().bold();
        let themed_style = base_style.with_role(Role::Warning);

        // Should preserve the bold attribute
        assert!(themed_style.attributes.has(Attribute::Bold));
    }

    #[test]
    fn test_color_conversion() {
        let color_info = ColorInfo {
            index: 42,
            value: ColorValue::Color256 { color256: 42 },
        };

        let cross_color = CrossColor::from(&color_info);
        assert_eq!(cross_color, CrossColor::AnsiValue(42));
    }

    #[test]
    fn test_themed_stylize() {
        let styled = "Hello".role(Role::Success);
        // Should have the success role styling applied
        assert_ne!(styled.style(), &ContentStyle::default());
    }

    #[test]
    fn test_helper_functions() {
        let success = crossterm_helpers::success_style();
        let error = crossterm_helpers::error_style();
        let warning = crossterm_helpers::warning_style();

        // All should be different styles
        assert_ne!(success, error);
        assert_ne!(error, warning);
        assert_ne!(warning, success);
    }
}
