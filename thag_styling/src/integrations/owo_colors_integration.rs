//! Integration with the owo-colors terminal styling library
//!
//! This module provides seamless integration between thag's theming system
//! and owo-colors's styling types.

#[cfg(feature = "owo_colors_support")]
use crate::ThemedStyle;
#[cfg(feature = "owo_colors_support")]
use crate::{ColorInfo, ColorValue, Role, Style};
#[cfg(feature = "owo_colors_support")]
use owo_colors::{AnsiColors, DynColors, Style as OwoStyle, XtermColors};

#[cfg(feature = "owo_colors_support")]
impl ThemedStyle<Self> for OwoStyle {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        let mut owo_style = Self::new();

        // Apply foreground color if present
        if let Some(fg_color) = &style.foreground {
            let dyn_color = DynColors::from(fg_color);
            owo_style = owo_style.color(dyn_color);
        }

        // Apply text effects
        if style.bold {
            owo_style = owo_style.bold();
        }
        if style.dim {
            owo_style = owo_style.dimmed();
        }
        if style.italic {
            owo_style = owo_style.italic();
        }
        if style.underline {
            owo_style = owo_style.underline();
        }

        owo_style
    }
}

#[cfg(feature = "owo_colors_support")]
impl ThemedStyle<Self> for DynColors {
    fn themed(role: Role) -> Self {
        let style = Style::from(role);
        Self::from_thag_style(&style)
    }

    fn from_thag_style(style: &Style) -> Self {
        style
            .foreground
            .as_ref()
            .map_or(
                Self::Ansi(AnsiColors::White),
                |color_info| match &color_info.value {
                    ColorValue::TrueColor { rgb } => Self::Rgb(rgb[0], rgb[1], rgb[2]),
                    ColorValue::Color256 { color256 } => Self::Xterm(XtermColors::from(*color256)),
                    ColorValue::Basic { .. } => Self::Xterm(XtermColors::from(color_info.index)),
                },
            ) // Default to white
    }
}

// From implementations for owo-colors types
#[cfg(feature = "owo_colors_support")]
impl From<&ColorInfo> for DynColors {
    fn from(color_info: &ColorInfo) -> Self {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Self::Rgb(rgb[0], rgb[1], rgb[2]),
            ColorValue::Color256 { color256 } => Self::Xterm(XtermColors::from(*color256)),
            ColorValue::Basic { .. } => Self::Xterm(XtermColors::from(color_info.index)),
        }
    }
}

#[cfg(feature = "owo_colors_support")]
impl From<&Style> for OwoStyle {
    fn from(style: &Style) -> Self {
        Self::from_thag_style(style)
    }
}

#[cfg(feature = "owo_colors_support")]
impl From<&Role> for DynColors {
    fn from(role: &Role) -> Self {
        Self::themed(*role)
    }
}

#[cfg(feature = "owo_colors_support")]
impl From<Role> for DynColors {
    fn from(role: Role) -> Self {
        Self::themed(role)
    }
}

#[cfg(feature = "owo_colors_support")]
impl From<&Role> for OwoStyle {
    fn from(role: &Role) -> Self {
        Self::themed(*role)
    }
}

#[cfg(feature = "owo_colors_support")]
impl From<Role> for OwoStyle {
    fn from(role: Role) -> Self {
        Self::themed(role)
    }
}

/// Convenience methods for owo-colors styling
#[cfg(feature = "owo_colors_support")]
pub trait OwoColorsStyleExt {
    /// Apply a thag role to this owo-colors style
    #[must_use]
    fn with_role(self, role: Role) -> Self;

    /// Apply a thag style to this owo-colors style
    #[must_use]
    fn with_thag_style(self, style: &Style) -> Self;

    /// Apply a thag foreground color to this style
    #[must_use]
    fn with_thag_color(self, color_info: &ColorInfo) -> Self;
}

#[cfg(feature = "owo_colors_support")]
impl OwoColorsStyleExt for OwoStyle {
    fn with_role(self, role: Role) -> Self {
        // Chain the themed style effects with existing style
        self.color(DynColors::from(&role))
    }

    fn with_thag_style(self, style: &Style) -> Self {
        let mut result = self;

        // Apply foreground color if present
        if let Some(fg_color) = &style.foreground {
            let dyn_color = DynColors::from(fg_color);
            result = result.color(dyn_color);
        }

        // Apply text effects (these accumulate with existing effects)
        if style.bold {
            result = result.bold();
        }
        if style.dim {
            result = result.dimmed();
        }
        if style.italic {
            result = result.italic();
        }
        if style.underline {
            result = result.underline();
        }

        result
    }

    fn with_thag_color(self, color_info: &ColorInfo) -> Self {
        let dyn_color = DynColors::from(color_info);
        self.color(dyn_color)
    }
}

/// Helper functions for common owo-colors use cases
#[cfg(feature = "owo_colors_support")]
pub mod helpers {
    use super::{Role, ThemedStyle};
    use owo_colors::Style as OwoStyle;

    /// Create a themed owo-colors style for success messages
    #[must_use]
    pub fn success_style() -> OwoStyle {
        OwoStyle::themed(Role::Success)
    }

    /// Create a themed owo-colors style for error messages
    #[must_use]
    pub fn error_style() -> OwoStyle {
        OwoStyle::themed(Role::Error)
    }

    /// Create a themed owo-colors style for warning messages
    #[must_use]
    pub fn warning_style() -> OwoStyle {
        OwoStyle::themed(Role::Warning)
    }

    /// Create a themed owo-colors style for info messages
    #[must_use]
    pub fn info_style() -> OwoStyle {
        OwoStyle::themed(Role::Info)
    }

    /// Create a themed owo-colors style for emphasis
    #[must_use]
    pub fn emphasis_style() -> OwoStyle {
        OwoStyle::themed(Role::Emphasis)
    }

    /// Create a themed owo-colors style for subtle text
    #[must_use]
    pub fn subtle_style() -> OwoStyle {
        OwoStyle::themed(Role::Subtle)
    }

    /// Create a themed owo-colors style for code blocks
    #[must_use]
    pub fn code_style() -> OwoStyle {
        OwoStyle::themed(Role::Code)
    }

    /// Create a themed owo-colors style for links
    #[must_use]
    pub fn link_style() -> OwoStyle {
        OwoStyle::themed(Role::Link)
    }
}

#[cfg(all(test, feature = "owo_colors_support"))]
mod tests {
    use super::*;
    use crate::{ColorInfo, ColorValue, Style};

    #[test]
    fn test_themed_style_creation() {
        let style = OwoStyle::themed(Role::Error);
        // Should not be the default/plain style
        assert!(!style.is_plain());
    }

    #[test]
    fn test_themed_color_creation() {
        let color = DynColors::themed(Role::Success);
        // Should be some form of color
        match color {
            DynColors::Xterm(_) | DynColors::Rgb(_, _, _) | DynColors::Ansi(_) => (),
            DynColors::Css(_) => panic!("Expected a concrete color variant"),
        }
    }

    #[test]
    fn test_style_extension() {
        let base_style = OwoStyle::new().bold();
        let themed_style = base_style.with_role(Role::Warning);

        // Should not be plain since we applied styling
        assert!(!themed_style.is_plain());
    }

    #[test]
    fn test_from_implementations() {
        // Test ColorInfo to DynColors conversion
        let color_info = ColorInfo {
            value: ColorValue::TrueColor { rgb: [255, 0, 0] },
            index: 1,
        };
        let owo_color = DynColors::from(&color_info);
        match owo_color {
            DynColors::Rgb(255, 0, 0) => (),
            _ => panic!("Expected RGB color"),
        }

        // Test Role to OwoStyle conversion
        let owo_style = OwoStyle::from(&Role::Error);
        assert!(!owo_style.is_plain());

        // Test Style to OwoStyle conversion
        let style = Style::from(Role::Warning);
        let owo_style = OwoStyle::from(&style);
        assert!(!owo_style.is_plain());

        // Test using .into() syntax
        let role = Role::Success;
        let _color: DynColors = (&role).into();
        let _style: OwoStyle = (&role).into();
    }

    #[test]
    fn test_helper_functions() {
        let success = helpers::success_style();
        let error = helpers::error_style();
        let warning = helpers::warning_style();
        let info = helpers::info_style();
        let emphasis = helpers::emphasis_style();
        let subtle = helpers::subtle_style();
        let code = helpers::code_style();
        let link = helpers::link_style();

        // All should have some styling applied
        assert!(!success.is_plain());
        assert!(!error.is_plain());
        assert!(!warning.is_plain());
        assert!(!info.is_plain());
        assert!(!emphasis.is_plain());
        assert!(!subtle.is_plain());
        assert!(!code.is_plain());
        assert!(!link.is_plain());
    }

    #[test]
    fn test_color_conversion_types() {
        // Test different color value types
        let true_color = ColorInfo {
            value: ColorValue::TrueColor {
                rgb: [128, 64, 192],
            },
            index: 5,
        };
        match DynColors::from(&true_color) {
            DynColors::Rgb(128, 64, 192) => (),
            _ => panic!("Expected RGB color"),
        }

        let color256 = ColorInfo {
            value: ColorValue::Color256 { color256: 42 },
            index: 42,
        };
        match DynColors::from(&color256) {
            DynColors::Xterm(XtermColors::UserBrightGreen) => (), // Index 42 maps to this
            _ => panic!("Expected UserBrightGreen"),
        }

        let basic_color = ColorInfo {
            value: ColorValue::Basic { index: 1 },
            index: 1,
        };
        match DynColors::from(&basic_color) {
            DynColors::Xterm(_) => (),
            _ => panic!("Expected basic color"),
        }
    }

    #[test]
    fn test_style_chaining() {
        let base_style = OwoStyle::new().bold().underline();
        let role_style = base_style.with_role(Role::Error);

        // Should preserve the chaining
        assert!(!role_style.is_plain());

        // Test with custom style
        let custom_style = Style::from(Role::Success);
        let final_style = base_style.with_thag_style(&custom_style);
        assert!(!final_style.is_plain());
    }
}
