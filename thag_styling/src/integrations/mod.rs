//! Integration modules for third-party terminal styling crates
//!
//! This module provides feature-gated integrations with popular terminal UI and styling
//! libraries, allowing them to use thag's theme-aware color system.

use crate::{Role, Style};

/// Trait for making any styling crate theme-aware
///
/// This trait provides a consistent interface for creating styled content
/// using thag's role-based theming system across different styling libraries.
///
/// # Examples
/// ```ignore
/// use thag_styling::{Role, ThemedStyle};
///
/// // Works with any integrated styling crate
/// let success_style = console::Style::themed(Role::Success);
/// let error_style = ratatui::style::Style::themed(Role::Error);
/// ```
pub trait ThemedStyle<T> {
    /// Create a themed style for the specified role
    ///
    /// # Arguments
    /// * `role` - The semantic role that determines styling
    ///
    /// # Returns
    /// A style object configured with theme-appropriate colors and attributes
    fn themed(role: Role) -> T;

    /// Create a themed style from a thag Style
    ///
    /// # Arguments
    /// * `style` - The thag Style to convert
    ///
    /// # Returns
    /// A style object in the target crate's format
    fn from_thag_style(style: &Style) -> T;
}

/// Trait for libraries that support theme switching
pub trait ThemeSwitchable {
    /// Switch to a new theme by name
    ///
    /// # Arguments
    /// * `theme_name` - Name of the theme to switch to
    ///
    /// # Errors
    /// Returns an error if the theme is not found or incompatible
    fn switch_theme(theme_name: &str) -> crate::StylingResult<()>;

    /// Get the current theme name
    fn current_theme() -> String;
}

// Feature-gated module declarations
#[cfg(feature = "ratatui_support")]
pub mod ratatui_integration;

#[cfg(feature = "nu_ansi_term_support")]
pub mod nu_ansi_term_integration;

#[cfg(feature = "crossterm_support")]
pub mod crossterm_integration;

#[cfg(feature = "console_support")]
pub mod console_integration;

// Future integrations (commented out to avoid warnings)
// #[cfg(feature = "owo_colors_support")]
// pub mod owo_colors_integration;

// #[cfg(feature = "indicatif_support")]
// pub mod indicatif_integration;

// Re-export integration types for convenience
#[cfg(feature = "ratatui_support")]
pub use ratatui_integration::*;

#[cfg(feature = "nu_ansi_term_support")]
pub use nu_ansi_term_integration::*;

#[cfg(feature = "crossterm_support")]
pub use crossterm_integration::*;

#[cfg(feature = "console_support")]
pub use console_integration::*;

// Future integrations (commented out to avoid warnings)
// #[cfg(feature = "owo_colors_support")]
// pub use owo_colors_integration::*;

// #[cfg(feature = "indicatif_support")]
// pub use indicatif_integration::*;
