# Integration Development Guide

This guide establishes consistent patterns for creating integrations between thag's theming system and third-party terminal styling libraries.

## Overview

Integrations provide seamless theme-aware styling for popular terminal UI libraries by implementing standard traits that convert thag's semantic roles and styles into library-specific types.

## Required Implementations

Every integration **MUST** implement the following patterns consistently:

### 1. ThemedStyle Trait Implementation

```rust
impl ThemedStyle<Self> for LibraryStyleType {
    fn themed(role: Role) -> Self {
        // Primary implementation logic goes here
        // Convert role to thag Style, then to library format
        let thag_style = Style::from(role);
        Self::from_thag_style(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        // Convert thag Style to library format
        // This is the core conversion logic
        // Handle colors, bold, italic, underline, dim, etc.
        // IMPORTANT: Never call Self::from() or other methods that might 
        // delegate back to ThemedStyle - implement conversion directly here
    }
}
```

**Key principles:**
- `themed()` takes `Role` **by value** and is the primary API
- `from_thag_style()` takes `&Style` **by reference** and contains core conversion logic
- `themed()` should delegate to `from_thag_style()` for consistency
- **CRITICAL**: `from_thag_style()` must not call `From` implementations that delegate back to `ThemedStyle` to avoid infinite recursion

### 2. From Trait Implementations (Complete Set)

Every integration **MUST** provide ALL of these From implementations:

```rust
// For Role conversions - delegate to ThemedStyle::themed()
impl From<&Role> for LibraryType {
    fn from(role: &Role) -> Self {
        Self::themed(*role)  // Dereference and delegate
    }
}

impl From<Role> for LibraryType {
    fn from(role: Role) -> Self {
        Self::themed(role)   // Direct delegate
    }
}

// For Style conversions - delegate to ThemedStyle::from_thag_style()
impl From<&Style> for LibraryType {
    fn from(style: &Style) -> Self {
        Self::from_thag_style(style)  // Direct delegate
    }
}

// For ColorInfo conversions - handle color format conversion
impl From<&ColorInfo> for LibraryColorType {
    fn from(color_info: &ColorInfo) -> Self {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => /* convert RGB */,
            ColorValue::Color256 { color256 } => /* convert 256-color */,
            ColorValue::Basic { .. } => /* convert basic color using index */,
        }
    }
}
```

**Key principles:**
- **Always provide both `From<Role>` and `From<&Role>`** for ergonomics
- **From implementations are convenience wrappers** that delegate to ThemedStyle methods
- **Never duplicate logic** - From impls should always delegate to the primary implementations

### 3. Extension Trait (Recommended)

Provide library-specific convenience methods:

```rust
pub trait LibraryStyleExt {
    #[must_use]
    fn with_role(self, role: Role) -> Self;
    
    #[must_use]
    fn with_thag_style(self, style: &Style) -> Self;
    
    // Add library-specific helpers as needed
}
```

### 4. Helper Functions Module (Recommended)

```rust
pub mod helpers {
    use super::{Role, ThemedStyle, LibraryStyleType};

    #[must_use]
    pub fn success_style() -> LibraryStyleType {
        LibraryStyleType::themed(Role::Success)
    }

    #[must_use]
    pub fn error_style() -> LibraryStyleType {
        LibraryStyleType::themed(Role::Error)
    }
    
    // Add more role-specific helpers
}
```

## Complete Implementation Template

Here's a complete template for a new integration:

```rust
//! Integration with the library-name terminal styling library
//!
//! This module provides seamless integration between thag's theming system
//! and library-name's styling types.

#[cfg(feature = "library_support")]
use crate::integrations::ThemedStyle;
#[cfg(feature = "library_support")]
use crate::{ColorInfo, ColorValue, Role, Style};
#[cfg(feature = "library_support")]
use library_name::{StyleType, ColorType};

// 1. ThemedStyle implementations (PRIMARY API)
#[cfg(feature = "library_support")]
impl ThemedStyle<Self> for StyleType {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from_thag_style(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        // Core conversion logic here
        // Convert colors, apply bold/italic/underline/dim
    }
}

#[cfg(feature = "library_support")]
impl ThemedStyle<Self> for ColorType {
    fn themed(role: Role) -> Self {
        let thag_style = Style::from(role);
        Self::from_thag_style(&thag_style)
    }

    fn from_thag_style(style: &Style) -> Self {
        // IMPORTANT: Implement color conversion directly to avoid recursion
        style.foreground.as_ref().map_or(
            /* default color */,
            |color_info| match &color_info.value {
                ColorValue::TrueColor { rgb } => /* convert RGB */,
                ColorValue::Color256 { color256 } => /* convert 256-color */,
                ColorValue::Basic { .. } => /* convert basic using index */,
            }
        )
    }
}

// 2. From implementations (CONVENIENCE WRAPPERS)
#[cfg(feature = "library_support")]
impl From<&ColorInfo> for ColorType {
    fn from(color_info: &ColorInfo) -> Self {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => /* convert RGB */,
            ColorValue::Color256 { color256 } => /* convert 256-color */,
            ColorValue::Basic { .. } => /* convert basic using index */,
        }
    }
}

#[cfg(feature = "library_support")]
impl From<&Style> for StyleType {
    fn from(style: &Style) -> Self {
        Self::from_thag_style(style)  // Delegate to primary
    }
}

#[cfg(feature = "library_support")]
impl From<&Role> for StyleType {
    fn from(role: &Role) -> Self {
        Self::themed(*role)  // Delegate to primary
    }
}

#[cfg(feature = "library_support")]
impl From<Role> for StyleType {
    fn from(role: Role) -> Self {
        Self::themed(role)  // Delegate to primary
    }
}

#[cfg(feature = "library_support")]
impl From<&Role> for ColorType {
    fn from(role: &Role) -> Self {
        Self::themed(*role)  // Delegate to primary
    }
}

#[cfg(feature = "library_support")]
impl From<Role> for ColorType {
    fn from(role: Role) -> Self {
        Self::themed(role)  // Delegate to primary
    }
}

// 3. Extension trait
#[cfg(feature = "library_support")]
pub trait LibraryStyleExt {
    #[must_use]
    fn with_role(self, role: Role) -> Self;
    
    #[must_use]
    fn with_thag_style(self, style: &Style) -> Self;
}

#[cfg(feature = "library_support")]
impl LibraryStyleExt for StyleType {
    fn with_role(self, role: Role) -> Self {
        // Combine existing style with themed role
        // Implementation depends on library's API
    }

    fn with_thag_style(self, style: &Style) -> Self {
        // Combine existing style with thag style
        // Implementation depends on library's API
    }
}

// 4. Helper functions
#[cfg(feature = "library_support")]
pub mod helpers {
    use super::{Role, ThemedStyle, StyleType};

    #[must_use]
    pub fn success_style() -> StyleType {
        StyleType::themed(Role::Success)
    }

    #[must_use]
    pub fn error_style() -> StyleType {
        StyleType::themed(Role::Error)
    }

    #[must_use]
    pub fn warning_style() -> StyleType {
        StyleType::themed(Role::Warning)
    }

    #[must_use]
    pub fn info_style() -> StyleType {
        StyleType::themed(Role::Info)
    }
}

// 5. Tests (REQUIRED)
#[cfg(all(test, feature = "library_support"))]
mod tests {
    use super::*;

    #[test]
    fn test_themed_style_creation() {
        let style = StyleType::themed(Role::Error);
        // Verify styling was applied
    }

    #[test]
    fn test_from_implementations() {
        // Test all From implementations
        let role = Role::Success;
        let _style1: StyleType = (&role).into();
        let _style2: StyleType = role.into();
    }

    #[test]
    fn test_color_conversions() {
        // Test ColorInfo conversions for all color types
    }

    #[test]
    fn test_helper_functions() {
        // Test helper functions work
    }
}
```

## Integration Checklist

Before submitting an integration, verify:

- [ ] **ThemedStyle implemented** for all relevant library types
- [ ] **All From implementations provided** (`&Role`, `Role`, `&Style`, `&ColorInfo`)
- [ ] **From implementations delegate** to ThemedStyle methods (no duplicate logic)
- [ ] **Extension trait provided** with `with_role()` and `with_thag_style()`
- [ ] **Helper functions module** with common role shortcuts
- [ ] **Feature-gated properly** with `#[cfg(feature = "library_support")]`
- [ ] **Added to mod.rs** with feature gate
- [ ] **Added to Cargo.toml** as optional dependency and feature
- [ ] **Comprehensive tests** covering all implementations
- [ ] **Demo script created** showing practical usage

## Common Pitfalls

### ⚠️ Infinite Recursion

**NEVER** create circular dependencies between `ThemedStyle` and `From` implementations:

```rust
// ❌ WRONG - Creates infinite recursion
impl ThemedStyle<Self> for ColorType {
    fn themed(role: Role) -> Self {
        Self::from(&role)  // Calls From<&Role>
    }
}

impl From<&Role> for ColorType {
    fn from(role: &Role) -> Self {
        Self::themed(*role)  // Calls ThemedStyle::themed - INFINITE LOOP!
    }
}
```

```rust
// ✅ CORRECT - From delegates to ThemedStyle, ThemedStyle has direct implementation
impl ThemedStyle<Self> for ColorType {
    fn themed(role: Role) -> Self {
        let style = Style::from(role);
        Self::from_thag_style(&style)  // Direct conversion
    }

    fn from_thag_style(style: &Style) -> Self {
        // Direct implementation - no delegation to From traits
        match style.foreground.as_ref() { /* ... */ }
    }
}

impl From<&Role> for ColorType {
    fn from(role: &Role) -> Self {
        Self::themed(*role)  // Safe - delegates to ThemedStyle
    }
}
```

## Rationale

### Why delegate From to ThemedStyle?

1. **Single source of truth**: Core logic lives in one place
2. **Consistency**: All code paths go through the same conversion
3. **Maintainability**: Changes only need to be made in one location
4. **Testing**: Easier to test one implementation thoroughly

### Why provide both `From<Role>` and `From<&Role>`?

1. **Ergonomics**: Users can use either owned or borrowed values naturally
2. **API consistency**: Matches Rust ecosystem patterns
3. **Flexibility**: Works with different usage patterns without forcing clones

### Why separate ThemedStyle from From?

1. **Semantic clarity**: ThemedStyle is the primary API, From is convenience
2. **Trait organization**: Each trait has a clear, focused purpose
3. **Extensibility**: Easy to add new conversion sources later