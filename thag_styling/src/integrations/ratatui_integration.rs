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
        Self::from_thag_style(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        let mut rata_style = Self::default();

        // Apply foreground color
        if let Some(color_info) = &style.foreground {
            rata_style = rata_style.fg(color_info_to_ratatui_color(color_info));
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

impl ThemedStyle<Self> for RataColor {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        thag_style.foreground.as_ref().map_or_else(
            || Self::Indexed(u8::from(&role)),
            color_info_to_ratatui_color,
        )
    }

    fn from_thag_style(style: &Style) -> Self {
        style
            .foreground
            .as_ref()
            .map_or(Self::Reset, color_info_to_ratatui_color)
    }
}

/// Convert `ColorInfo` to `ratatui` Color
const fn color_info_to_ratatui_color(color_info: &ColorInfo) -> RataColor {
    match &color_info.value {
        ColorValue::TrueColor { rgb } => RataColor::Rgb(rgb[0], rgb[1], rgb[2]),
        ColorValue::Color256 { color256 } => RataColor::Indexed(*color256),
        ColorValue::Basic { .. } => RataColor::Indexed(color_info.index),
    }
}

// Note: From implementations are provided in the main styling.rs file to avoid conflicts

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
        let themed = Self::from_thag_style(style);
        self.patch(themed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
