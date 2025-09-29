//! Integration with the owo-colors terminal styling library
//!
//! This module provides seamless integration between thag's theming system
//! and owo-colors's styling types.

use crate::integrations::ThemedStyle;
use crate::{ColorInfo, ColorValue, Role, Style};
use owo_colors::{Color as OwoColor, Style as OwoStyle};

impl ThemedStyle<Self> for OwoStyle {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        Self::from(style)
    }
}

impl ThemedStyle<Self> for OwoColor {
    fn themed(role: Role) -> Self {
        Self::from(&role)
    }

    fn from_thag_style(style: &Style) -> Self {
        style.foreground.as_ref().map_or(Self::Fixed(7), Self::from) // Default to white
    }
}

// From implementations for owo-colors types
impl From<&ColorInfo> for OwoColor {
    fn from(color_info: &ColorInfo) -> Self {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Self::Rgb(rgb[0], rgb[1], rgb[2]),
            ColorValue::Color256 { color256 } => Self::Fixed(*color256),
            ColorValue::Basic { .. } => Self::Fixed(color_info.index),
        }
    }
}

impl From<&Style> for OwoStyle {
    fn from(style: &Style) -> Self {
        Self {
            foreground: style.foreground.as_ref().map(OwoColor::from),
            background: None, // Background color not supported in current Style struct
            is_bold: style.bold,
            is_dimmed: style.dim,
            is_italic: style.italic,
            is_underline: style.underline,
            is_blink: false,          // Not supported by thag
            is_reverse: false,        // Not supported by thag
            is_hidden: false,         // Not supported by thag
            is_strikethrough: false,  // Strikethrough not supported in current Style struct
            prefix_with_reset: false, // Use owo-colors default
        }
    }
}

impl From<&Role> for OwoStyle {
    fn from(role: &Role) -> Self {
        let style = Style::from(*role);
        Self::from(&style)
    }
}

impl From<&Role> for OwoColor {
    fn from(role: &Role) -> Self {
        let style = Style::from(*role);
        style
            .foreground
            .as_ref()
            .map_or_else(|| Self::Fixed(u8::from(role)), Self::from)
    }
}

/// Convenience methods for owo-colors styling
pub trait OwoColorsStyleExt {
    /// Apply a thag role to this owo-colors style
    #[must_use]
    fn with_role(self, role: Role) -> Self;

    /// Apply a thag style to this owo-colors style
    #[must_use]
    fn with_thag_style(self, style: &Style) -> Self;
}

impl OwoColorsStyleExt for OwoStyle {
    fn with_role(self, role: Role) -> Self {
        let themed = Self::themed(role);
        Self {
            foreground: themed.foreground.or(self.foreground),
            background: themed.background.or(self.background),
            is_bold: themed.is_bold || self.is_bold,
            is_dimmed: themed.is_dimmed || self.is_dimmed,
            is_italic: themed.is_italic || self.is_italic,
            is_underline: themed.is_underline || self.is_underline,
            is_blink: self.is_blink,     // Keep existing
            is_reverse: self.is_reverse, // Keep existing
            is_hidden: self.is_hidden,   // Keep existing
            is_strikethrough: themed.is_strikethrough || self.is_strikethrough,
            prefix_with_reset: self.prefix_with_reset, // Keep existing
        }
    }

    fn with_thag_style(self, style: &Style) -> Self {
        let themed = Self::from(style);
        Self {
            foreground: themed.foreground.or(self.foreground),
            background: themed.background.or(self.background),
            is_bold: themed.is_bold || self.is_bold,
            is_dimmed: themed.is_dimmed || self.is_dimmed,
            is_italic: themed.is_italic || self.is_italic,
            is_underline: themed.is_underline || self.is_underline,
            is_blink: self.is_blink,     // Keep existing
            is_reverse: self.is_reverse, // Keep existing
            is_hidden: self.is_hidden,   // Keep existing
            is_strikethrough: themed.is_strikethrough || self.is_strikethrough,
            prefix_with_reset: self.prefix_with_reset, // Keep existing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themed_style_creation() {
        let style = OwoStyle::themed(Role::Error);
        // Should have some styling applied
        assert_ne!(style, OwoStyle::default());
    }

    #[test]
    fn test_themed_color_creation() {
        let color = OwoColor::themed(Role::Success);
        // Should be a fixed color
        match color {
            OwoColor::Fixed(_) | OwoColor::Rgb(_, _, _) => (),
            _ => panic!("Expected Fixed or Rgb color"),
        }
    }

    #[test]
    fn test_style_extension() {
        let base_style = OwoStyle {
            is_bold: true,
            ..Default::default()
        };
        let themed_style = base_style.with_role(Role::Warning);

        // Should preserve the bold modifier
        assert!(themed_style.is_bold);
    }

    #[test]
    fn test_from_implementations() {
        use crate::{ColorInfo, ColorValue, Style};

        // Test ColorInfo to OwoColor conversion
        let color_info = ColorInfo {
            value: ColorValue::TrueColor { rgb: [255, 0, 0] },
            ansi: "31",
            index: 1,
        };
        let owo_color = OwoColor::from(&color_info);
        match owo_color {
            OwoColor::Rgb(255, 0, 0) => (),
            _ => panic!("Expected RGB color"),
        }

        // Test Role to OwoStyle conversion
        let owo_style = OwoStyle::from(&Role::Error);
        assert!(owo_style.foreground.is_some());

        // Test Style to OwoStyle conversion
        let style = Style::from(Role::Warning);
        let owo_style = OwoStyle::from(&style);
        assert!(owo_style.foreground.is_some());

        // Test using .into() syntax
        let role = Role::Success;
        let _color: OwoColor = (&role).into();
        let _style: OwoStyle = (&role).into();
    }
}
