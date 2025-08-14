//! Integration with the ratatui terminal UI library
//!
//! This module provides seamless integration between thag's theming system
//! and ratatui's styling types, allowing ratatui applications to use
//! theme-aware colors automatically.

use crate::integrations::ThemedStyle;
use crate::{ColorInfo, ColorValue, Role, Style};
use ratatui::style::{Color as RataColor, Modifier, Style as RataStyle};

impl ThemedStyle<Self> for RataStyle {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        Self::from(style)
    }
}

impl ThemedStyle<Self> for RataColor {
    fn themed(role: Role) -> Self {
        Self::from(&role)
    }

    fn from_thag_style(style: &Style) -> Self {
        style
            .foreground
            .as_ref()
            .map_or(Self::Reset, RataColor::from)
    }
}

// From implementations for ratatui types
impl From<&ColorInfo> for RataColor {
    fn from(color_info: &ColorInfo) -> Self {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Self::Rgb(rgb[0], rgb[1], rgb[2]),
            ColorValue::Color256 { color256 } => Self::Indexed(*color256),
            ColorValue::Basic { .. } => Self::Indexed(color_info.index),
        }
    }
}

impl From<&Style> for RataStyle {
    fn from(style: &Style) -> Self {
        let mut rata_style = Self::default();

        // Apply foreground color
        if let Some(color_info) = &style.foreground {
            rata_style = rata_style.fg(RataColor::from(color_info));
        }

        // Note: Background color not supported in current Style struct

        // Apply style attributes
        if style.bold {
            rata_style = rata_style.add_modifier(Modifier::BOLD);
        }
        if style.italic {
            rata_style = rata_style.add_modifier(Modifier::ITALIC);
        }
        if style.dim {
            rata_style = rata_style.add_modifier(Modifier::DIM);
        }
        if style.underline {
            rata_style = rata_style.add_modifier(Modifier::UNDERLINED);
        }
        // Note: Strikethrough not supported in current Style struct

        rata_style
    }
}

impl From<&Role> for RataStyle {
    fn from(role: &Role) -> Self {
        let style = Style::from(*role);
        Self::from(&style)
    }
}

impl From<&Role> for RataColor {
    fn from(role: &Role) -> Self {
        let style = Style::from(*role);
        style
            .foreground
            .as_ref()
            .map_or_else(|| Self::Indexed(u8::from(role)), RataColor::from)
    }
}

/// Convenience methods for ratatui styling
pub trait RatatuiStyleExt {
    /// Apply a thag role to this ratatui style
    #[must_use]
    fn with_role(self, role: Role) -> Self;

    /// Apply a thag style to this ratatui style
    #[must_use]
    fn with_thag_style(self, style: &Style) -> Self;
}

impl RatatuiStyleExt for RataStyle {
    fn with_role(self, role: Role) -> Self {
        let themed = Self::themed(role);
        self.patch(themed)
    }

    fn with_thag_style(self, style: &Style) -> Self {
        let themed = Self::from(style);
        self.patch(themed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Stylize;

    #[test]
    fn test_themed_style_creation() {
        let style = RataStyle::themed(Role::Error);
        // Should have some styling applied
        assert_ne!(style, RataStyle::default());
    }

    #[test]
    fn test_themed_color_creation() {
        let color = RataColor::themed(Role::Success);
        // Should not be the reset color
        assert_ne!(color, RataColor::Reset);
    }

    #[test]
    fn test_style_extension() {
        let base_style = RataStyle::default().bold();
        let themed_style = base_style.with_role(Role::Warning);

        // Should preserve the bold modifier
        assert!(themed_style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_from_implementations() {
        use crate::{ColorInfo, ColorValue, Style};

        // Test ColorInfo to RataColor conversion
        let color_info = ColorInfo {
            value: ColorValue::TrueColor { rgb: [255, 0, 0] },
            ansi: "31",
            index: 1,
        };
        let rata_color = RataColor::from(&color_info);
        match rata_color {
            RataColor::Rgb(255, 0, 0) => (),
            _ => panic!("Expected RGB color"),
        }

        // Test Role to RataStyle conversion
        let rata_style = RataStyle::from(&Role::Error);
        assert!(rata_style.fg.is_some());

        // Test Style to RataStyle conversion
        let style = Style::from(Role::Warning);
        let rata_style = RataStyle::from(&style);
        assert!(rata_style.fg.is_some());

        // Test using .into() syntax
        let role = Role::Success;
        let _color: RataColor = (&role).into();
        let _style: RataStyle = (&role).into();
    }
}
