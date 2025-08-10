//! Integration with the nu-ansi-term terminal styling library
//!
//! This module provides seamless integration between thag's theming system
//! and nu-ansi-term's styling types, enabling reedline and Nu shell applications
//! to use theme-aware colors automatically.

use crate::integrations::ThemedStyle;
use crate::{ColorInfo, ColorValue, Role, Style};
use nu_ansi_term::{Color as NuColor, Style as NuStyle};

impl ThemedStyle<NuStyle> for NuStyle {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from_thag_style(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        Self {
            foreground: style.foreground.as_ref().map(color_info_to_nu_color),
            background: None, // Background color not supported in current Style struct
            is_bold: style.bold,
            is_dimmed: style.dim,
            is_italic: style.italic,
            is_underline: style.underline,
            is_blink: false,          // Not supported by thag
            is_reverse: false,        // Not supported by thag
            is_hidden: false,         // Not supported by thag
            is_strikethrough: false,  // Strikethrough not supported in current Style struct
            prefix_with_reset: false, // Use nu-ansi-term default
        }
    }
}

impl ThemedStyle<NuColor> for NuColor {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        thag_style
            .foreground
            .as_ref()
            .map(color_info_to_nu_color)
            .unwrap_or_else(|| Self::Fixed(u8::from(&role)))
    }

    fn from_thag_style(style: &Style) -> Self {
        style
            .foreground
            .as_ref()
            .map(color_info_to_nu_color)
            .unwrap_or(Self::Fixed(7)) // Default to white
    }
}

/// Convert ColorInfo to nu-ansi-term Color
fn color_info_to_nu_color(color_info: &ColorInfo) -> NuColor {
    match &color_info.value {
        ColorValue::TrueColor { rgb } => NuColor::Rgb(rgb[0], rgb[1], rgb[2]),
        ColorValue::Color256 { color256 } => NuColor::Fixed(*color256),
        ColorValue::Basic { .. } => NuColor::Fixed(color_info.index),
    }
}

/// Note: From implementations are provided in the main styling.rs file to avoid conflicts

/// Convenience methods for nu-ansi-term styling
pub trait NuAnsiTermStyleExt {
    /// Apply a thag role to this nu-ansi-term style
    fn with_role(self, role: Role) -> Self;

    /// Apply a thag style to this nu-ansi-term style
    fn with_thag_style(self, style: &Style) -> Self;
}

impl NuAnsiTermStyleExt for NuStyle {
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
        let themed = Self::from_thag_style(style);
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

/// Helper functions for reedline integration
pub mod reedline_helpers {
    use super::*;
    use nu_ansi_term::Style as NuStyle;

    /// Create a themed nu-ansi-term style for reedline prompts
    pub fn prompt_style() -> NuStyle {
        NuStyle::themed(Role::Normal)
    }

    /// Create a themed nu-ansi-term style for reedline completions
    pub fn completion_style() -> NuStyle {
        NuStyle::themed(Role::Subtle)
    }

    /// Create a themed nu-ansi-term style for reedline selections
    pub fn selection_style() -> NuStyle {
        NuStyle::themed(Role::Emphasis)
    }

    /// Create a themed nu-ansi-term style for reedline errors
    pub fn error_style() -> NuStyle {
        NuStyle::themed(Role::Error)
    }

    /// Create a themed nu-ansi-term style for reedline hints
    pub fn hint_style() -> NuStyle {
        NuStyle::themed(Role::Info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themed_style_creation() {
        let style = NuStyle::themed(Role::Error);
        // Should have some styling applied
        assert_ne!(style, NuStyle::default());
    }

    #[test]
    fn test_themed_color_creation() {
        let color = NuColor::themed(Role::Success);
        // Should be a fixed color
        match color {
            NuColor::Fixed(_) | NuColor::Rgb(_, _, _) => (),
            _ => panic!("Expected Fixed or Rgb color"),
        }
    }

    #[test]
    fn test_style_extension() {
        let base_style = NuStyle {
            is_bold: true,
            ..Default::default()
        };
        let themed_style = base_style.with_role(Role::Warning);

        // Should preserve the bold modifier
        assert!(themed_style.is_bold);
    }

    #[test]
    fn test_reedline_helpers() {
        let prompt = reedline_helpers::prompt_style();
        let completion = reedline_helpers::completion_style();
        let selection = reedline_helpers::selection_style();
        let error = reedline_helpers::error_style();
        let hint = reedline_helpers::hint_style();

        // All should be different styles
        assert_ne!(prompt, completion);
        assert_ne!(completion, selection);
        assert_ne!(selection, error);
        assert_ne!(error, hint);
    }
}
