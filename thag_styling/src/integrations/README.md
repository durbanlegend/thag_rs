# Thag Styling Integrations

This module provides feature-gated integrations with popular terminal styling and UI libraries, enabling them to use thag's theme-aware color system automatically.

## Overview

The integration system is built around two main concepts:

1. **ThemedStyle Trait** - Provides a consistent interface for creating themed styles across different libraries
2. **Feature-Gated Modules** - Each integration is optional and only compiled when the corresponding feature is enabled

## ThemedStyle Trait

The `ThemedStyle<T>` trait allows any styling library to become theme-aware with minimal effort:

```rust
use thag_styling::{Role, ThemedStyle};

// Works with any integrated styling crate
let success_style = ratatui::style::Style::themed(Role::Success);
let error_color = crossterm::style::Color::themed(Role::Error);
let warning_style = nu_ansi_term::Style::themed(Role::Warning);
```

### Trait Methods

- `themed(role: Role) -> T` - Create a themed style for the specified semantic role
- `from_thag_style(style: &Style) -> T` - Convert a thag Style to the target library's format

## Available Integrations

### Ratatui (`ratatui_support` feature)

Integration with the popular terminal UI library for building rich text-based interfaces.

```rust
use ratatui::style::{Color, Style};
use thag_styling::{Role, ThemedStyle};

// Create themed styles
let title_style = Style::themed(Role::Heading1);
let error_color = Color::themed(Role::Error);

// Extend existing styles
let base_style = Style::default().bold();
let themed_style = base_style.with_role(Role::Success);
```

**Features:**
- Full `Style` and `Color` support
- Extension trait (`RatatuiStyleExt`) for combining styles
- Automatic color format conversion (RGB, 256-color, basic)

### Nu-ANSI-Term (`nu_ansi_term_support` feature)

Integration for reedline and Nu shell applications.

```rust
use nu_ansi_term::{Color, Style};
use thag_styling::{Role, ThemedStyle};

// Create themed styles
let prompt_style = Style::themed(Role::Normal);
let error_color = Color::themed(Role::Error);

// Reedline helpers
use thag_styling::integrations::nu_ansi_term_integration::reedline_helpers;
let selection_style = reedline_helpers::selection_style();
```

**Features:**
- Full `Style` and `Color` support
- Reedline helper functions for common UI elements
- Extension trait (`NuAnsiTermStyleExt`) for style composition

### Crossterm (`crossterm_support` feature)

Integration with the cross-platform terminal manipulation library.

```rust
use crossterm::style::{ContentStyle, Color};
use thag_styling::{Role, ThemedStyle};

// Create themed styles
let content_style = ContentStyle::themed(Role::Info);
let color = Color::themed(Role::Success);

// Use the ThemedStylize extension
use thag_styling::integrations::crossterm_integration::ThemedStylize;
let styled_content = "Hello".role(Role::Emphasis);
```

**Features:**
- `ContentStyle` and `Color` support
- Helper functions for common operations
- `ThemedStylize` trait for fluent styling
- Queue-based terminal operations support

## Usage Examples

### Drop-in Replacement

```rust
// Before: Hardcoded colors
let error_style = ratatui::style::Style::default()
    .fg(ratatui::style::Color::Red)
    .add_modifier(ratatui::style::Modifier::BOLD);

// After: Theme-aware
let error_style = ratatui::style::Style::themed(Role::Error);
```

### Combining with Existing Styles

```rust
// Extend existing styles with theme-aware colors
let my_style = ratatui::style::Style::default()
    .bold()
    .italic()
    .with_role(Role::Warning);  // Add theme-aware warning colors
```

### Cross-Library Consistency

```rust
// Same role, different libraries - consistent appearance
let ratatui_style = ratatui::style::Style::themed(Role::Success);
let crossterm_style = crossterm::style::ContentStyle::themed(Role::Success);
let nu_style = nu_ansi_term::Style::themed(Role::Success);
```

## Adding New Integrations

To add support for a new styling library:

1. **Create Integration Module**:
   ```rust
   // src/integrations/my_crate_integration.rs
   use crate::integrations::ThemedStyle;
   use crate::{Role, Style};
   
   impl ThemedStyle<my_crate::Style> for my_crate::Style {
       fn themed(role: Role) -> Self {
           let thag_style = Style::from(role);
           Self::from_thag_style(&thag_style)
       }
   
       fn from_thag_style(style: &Style) -> Self {
           // Convert thag Style to my_crate::Style
           todo!()
       }
   }
   ```

2. **Add Feature to Cargo.toml**:
   ```toml
   [features]
   my_crate_support = ["my_crate"]
   
   [dependencies]
   my_crate = { version = "1.0", optional = true }
   ```

3. **Update Module Declaration**:
   ```rust
   // src/integrations/mod.rs
   #[cfg(feature = "my_crate_support")]
   pub mod my_crate_integration;
   ```

## Performance Notes

- Color calculations are cached at theme initialization
- Style conversions are lightweight (mostly struct field mapping)
- No runtime overhead when features are disabled
- Theme detection happens once at startup

## Feature Flags

All integrations are feature-gated to minimize dependencies:

- `ratatui_support` - Ratatui terminal UI library
- `nu_ansi_term_support` - Nu-ANSI-Term for reedline/Nu shell
- `crossterm_support` - Crossterm cross-platform terminal
- `console_support` - Console styling library (planned)
- `owo_colors_support` - OWO Colors zero-allocation styling (planned)

## Migration Guide

### From Direct Color Usage

```rust
// Old approach
use ratatui::style::Color;
let color = Color::Cyan;  // Hardcoded, may not be visible on all backgrounds

// New approach  
use thag_styling::{Role, ThemedStyle};
let color = Color::themed(Role::Info);  // Theme-aware, always visible
```

### From Custom Color Schemes

```rust
// Old approach
struct MyColors {
    success: Color,
    error: Color,
    warning: Color,
}

// New approach - just use roles
use thag_styling::Role;
// Colors automatically adapt to terminal theme
let success_style = Style::themed(Role::Success);
let error_style = Style::themed(Role::Error);
let warning_style = Style::themed(Role::Warning);
```

## Best Practices

1. **Use Semantic Roles**: Prefer `Role::Error` over hardcoded red colors
2. **Combine Carefully**: When combining themed styles, ensure they remain readable
3. **Test Across Terminals**: Verify appearance in both light and dark terminal themes  
4. **Leverage Extensions**: Use library-specific extension traits for advanced features
5. **Feature Gate**: Only enable the integrations you actually use to minimize compile time

## Future Integrations

Planned integrations include:

- `console` - Popular styling library by Mitsuhiko
- `owo-colors` - Zero-allocation terminal colors
- `indicatif` - Progress bars and spinners
- `dialoguer` - Interactive components
- `clap` - Command line parser styling
- `comfy-table` - Pretty table printing

## Troubleshooting

### Colors Not Appearing

- Ensure the integration feature is enabled in `Cargo.toml`
- Check that your terminal supports the required color depth
- Verify theme detection is working with `thag_styling::display_terminal_attributes()`

### Compilation Errors

- Make sure you have the optional dependency installed
- Check feature flag spelling in `Cargo.toml`
- Ensure you're using compatible versions of the styling library

### Performance Issues

- Theme detection only happens once - subsequent style creation is fast
- Consider using `basic` color support for maximum performance
- Profile with `cargo flamegraph` if you suspect styling overhead