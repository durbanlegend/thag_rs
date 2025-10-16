# Base Colors Feature Documentation

## Overview

The `base_colors` feature adds an optional array to thag theme files that preserves the original base16/24 color palette for accurate ANSI terminal mapping. This bridges the gap between semantic color roles (used for text styling) and positional ANSI color slots (used for terminal colors).

## Problem Statement

### The Semantic vs. ANSI Divide

**thag_styling themes** use semantic roles:

- 16 named roles: `Heading1`, `Error`, `Normal`, `Code`, etc.
- Each role has a specific purpose in text styling
- Multiple base16/24 colors may map to the same role
- Some base16/24 colors may not be used at all

**ANSI terminals** use positional slots:

- 16 numbered slots: ANSI colors 0-15
- Each slot has a specific meaning (Black, Red, Green, etc.)
- Applications expect consistent color positions

### The Conversion Loss

When converting base16/24 themes to thag format:

```
base16/24 (16-24 colors) → thag themes (16 roles) → terminal ANSI (16 slots)
           ↓                                              ↑
    Many-to-one mapping                           Lost information!
```

**Example:**

- `base00` (background) → Not used in any role
- `base0E` (magenta) → `Heading1` role
- `base0F` (brown) → `Heading2` role
- `base0C` (cyan) → `Code` role AND `Heading3` role

When `thag_sync_palette` tried to populate ANSI slots 0-15, it had to guess from the 16 semantic roles, losing the original theme designer's intended ANSI mapping.

## Solution: Optional base_colors Array

### Design

Add an **optional array** to theme files that preserves the original base16/24 RGB values:

```toml
base_colors = [
    [30, 30, 46],    # base00 - background
    [24, 24, 37],    # base01 - lighter background
    [49, 50, 68],    # base02 - selection background
    # ... through base0F (16 total)
    # ... or base17 (24 total for base24 themes)
]
```

### Key Characteristics

1. **Optional**: Not required; existing themes work without it
2. **Lazy-loaded**: Skipped during normal theme loading to save memory
3. **On-demand**: Loaded only when needed via `load_base_colors()`
4. **Preserved**: Original palette maintained for perfect ANSI mapping
5. **Backward compatible**: No breaking changes to existing code

## Technical Implementation

### 1. Theme Structure

Added to `thag_styling/src/styling.rs`:

```rust
pub struct Theme {
    // ... existing fields

    /// Base16/24 color array for ANSI terminal mapping (optional)
    /// Skipped during normal theme loading to save memory.
    /// Load on-demand using `load_base_colors()` when needed.
    pub base_colors: Option<Vec<[u8; 3]>>,
}
```

**Memory Impact**: Only ~72 bytes per theme (24 colors × 3 bytes) when loaded.

### 2. Lazy Loading Method

```rust
impl Theme {
    /// Load base16/24 colors on-demand for ANSI terminal mapping
    pub fn load_base_colors(&mut self) -> StylingResult<()> {
        if self.base_colors.is_some() {
            return Ok(()); // Already loaded
        }

        // Re-read theme file and extract just base_colors
        let content = fs::read_to_string(&self.filename)?;
        let value: toml::Value = toml::from_str(&content)?;

        if let Some(base_colors_value) = value.get("base_colors") {
            // Parse and store the array
            // ...
        }

        Ok(())
    }
}
```

### 3. Converter Integration

Both `thag_convert_themes.rs` and `thag_convert_themes_alt.rs` now:

1. Extract base16/24 colors from source

2. Convert to `[u8; 3]` arrays

3. Store in `Theme.base_colors`

4. Serialize to TOML output

```rust
impl BaseTheme {
    fn extract_base_colors(&self) -> Result<Vec<[u8; 3]>, Box<dyn Error>> {
        let to_array = |hex: &str| -> Result<[u8; 3], Box<dyn Error>> {
            let (r, g, b) = hex_to_rgb(hex)?;
            Ok([r, g, b])
        };

        let mut colors = vec![
            to_array(&self.palette.base00)?,
            to_array(&self.palette.base01)?,
            // ... through base0F
        ];

        // Add base24 colors if present (base10-17)
        if self.is_base24() {
            // ...
        }

        Ok(colors)
    }
}
```

### 4. ANSI Mapping in thag_sync_palette

Updated `build_ansi_color_map()` in `palette_sync.rs`:

```rust
fn build_ansi_color_map(theme: &Theme) -> Vec<[u8; 3]> {
    // Prefer base_colors if available
    if let Some(base_colors) = &theme.base_colors {
        return base_colors.iter().take(16).copied().collect();
    }

    // Fallback to role-based mapping for themes without base_colors
    vec![
        theme.bg_rgb,
        extract_rgb(Role::Emphasis),   // Red
        extract_rgb(Role::Success),    // Green
        // ... etc
    ]
}
```

### 5. Usage in thag_sync_palette Binary

```rust
fn apply_theme(theme_name: &str) {
    let mut theme = Theme::get_builtin(theme_name)?;

    // Load base_colors for accurate ANSI mapping
    if let Err(e) = theme.load_base_colors() {
        // Silently fall back to role-based mapping
        vprtln!(V::V, "⚠️  Could not load base colors: {}", e);
    }

    PaletteSync::apply_theme(&theme)?;
}
```

## Usage Examples

### For Theme Converters

```bash
# Convert base16/24 theme - base_colors automatically included
thag_convert_themes_alt --input dracula.yaml -o themes/

# Resulting theme file includes base_colors array
cat themes/dracula.toml
# name = "Dracula"
# ...
# base_colors = [
#     [40, 42, 54],   # base00
#     ...
# ]
```

### For Applications Using Themes

```rust
use thag_styling::Theme;
use std::path::Path;

// Normal usage - base_colors not loaded (saves memory)
let theme = Theme::load_from_file(Path::new("my-theme.toml"))?;
let style = theme.style_for(Role::Heading1);
println!("{}", style.paint("Hello!"));

// When you need base_colors (e.g., for ANSI terminal mapping)
let mut theme = Theme::load_from_file(Path::new("my-theme.toml"))?;
theme.load_base_colors()?;

if let Some(colors) = &theme.base_colors {
    println!("Theme has {} base colors", colors.len());
    // Use for ANSI mapping, color pickers, etc.
}
```

### For Terminal Synchronization

```bash
# Apply theme to terminal - uses base_colors for accurate ANSI mapping
thag_sync_palette apply catppuccin-mocha

# Preview theme temporarily
thag_sync_palette preview dracula

# The tool automatically loads base_colors when available
```

## Benefits

### 1. Perfect Roundtrip Conversion

```
base16 theme → thag theme (with base_colors) → terminal ANSI
                              ↓
                    Preserves exact original mapping!
```

### 2. Theme Designer Intent Preserved

Original base16/24 color positions are maintained:

- `base08` (red) → ANSI slot 1 (Red)

- `base0B` (green) → ANSI slot 2 (Green)

- `base0D` (blue) → ANSI slot 4 (Blue)

### 3. Zero Overhead for Normal Use

- Themes load without `base_colors` by default

- Only ~72 bytes when explicitly loaded

- Most applications never need it

### 4. Backward Compatible

- Existing themes work without modification

- Optional field - no breaking changes

- Graceful fallback to role-based mapping

### 5. Future-Proof

Opens doors for:

- Interactive theme editor color pickers

- More sophisticated ANSI mappings

- Color palette analysis tools

- Theme validation utilities

## File Format

### TOML Structure

```toml
name = "Theme Name"
description = "Theme description"
term_bg_luma = "dark"
min_color_support = "true_color"
backgrounds = ["#282a36"]
bg_rgbs = [[40, 42, 54]]

# Base16/24 colors for ANSI terminal mapping
base_colors = [
    [40, 42, 54],    # base00 - Background
    [68, 71, 90],    # base01 - Lighter Background
    [98, 114, 164],  # base02 - Selection Background
    [98, 114, 164],  # base03 - Comments
    [189, 147, 249], # base04 - Dark Foreground
    [248, 248, 242], # base05 - Default Foreground
    [248, 248, 242], # base06 - Light Foreground
    [255, 255, 255], # base07 - Light Background
    [255, 85, 85],   # base08 - Red
    [255, 184, 108], # base09 - Orange
    [241, 250, 140], # base0A - Yellow
    [80, 250, 123],  # base0B - Green
    [139, 233, 253], # base0C - Cyan
    [189, 147, 249], # base0D - Blue
    [255, 121, 198], # base0E - Magenta
    [189, 147, 249], # base0F - Brown
]

[palette.heading1]
rgb = [255, 121, 198]
style = ["bold"]
# ... etc
```

### Array Format

- **Base16**: 16 entries (base00 through base0F)
- **Base24**: 24 entries (base00 through base17)

- Each entry: `[R, G, B]` where R, G, B are 0-255

## Validation

### Converter Guarantees

1. **Correct count**: 16 for base16, 24 for base24
2. **Valid RGB**: All values 0-255
3. **Preserved order**: Matches base16/24 specification
4. **Serialization**: Always included in converter output

### Runtime Behavior

1. **Missing array**: Silently uses role-based fallback
2. **Invalid format**: Returns error from `load_base_colors()`
3. **Wrong count**: Accepted (takes first 16 for ANSI)
4. **Invalid RGB**: Filtered out during parsing

## Future Enhancements

### 1. Theme Editor Integration

**Planned**: Enhance `thag_edit_theme` to show all base_colors:

```rust
struct ColorCandidateWithProvenance {
    hex: String,
    rgb: [u8; 3],
    base_indices: Vec<String>,  // ["base00", "base01"]
    roles: Vec<Role>,            // [Role::Subtle, Role::Commentary]
}
```

**UI Preview**:
```
? Select new color:
  ❯ #282a36 (base00, base01 | Subtle, Commentary)
    #f8f8f2 (base05 | Normal)
    #ff5555 (base08 | Error)
    #50fa7b (base0B | Link, Success)
    #8be9fd (base0C, base15 | Code, Heading3)
```

### 2. Color Palette Analysis

```rust
impl Theme {
    pub fn analyze_base_colors(&self) -> ColorAnalysis {
        // Contrast ratios
        // Color distribution
        // Duplicate detection
        // Accessibility scores
    }
}
```

### 3. Extended ANSI Support

Support for 256-color and true-color terminals:

- Map base24 colors 16-23 to extended slots

- Provide color downsampling for 256-color mode

- Smart fallbacks for limited color support

### 4. Theme Validation Tool

```bash
thag_validate_theme my-theme.toml
# ✓ base_colors present (24 entries)
# ✓ All RGB values valid
# ✓ Contrast ratios meet WCAG AA
# ⚠ base10 and base11 are identical
```

## Performance Characteristics

### Memory Usage
- **Without loading**: 0 bytes (None)
- **Base16 loaded**: 48 bytes (16 × 3)
- **Base24 loaded**: 72 bytes (24 × 3)
- **Preloaded themes**: 0 bytes (skipped during compilation)

### Load Time

- **Normal theme load**: No impact (field skipped)
- **On-demand load**: ~1-2ms (file I/O + TOML parsing)
- **Cached after first load**: 0ms (already in memory)

### ANSI Mapping Performance

- **With base_colors**: O(1) - direct array access
- **Without base_colors**: O(16) - role extraction
- **Difference**: Negligible (~1μs)

## Testing

### Converter Tests

```bash
# Test base16 conversion
thag_convert_themes_alt --input base16-theme.yaml -o /tmp/test.toml
grep -c "base_colors = \[" /tmp/test.toml  # Should be 1

# Count entries
sed -n '/base_colors = \[/,/\]/p' /tmp/test.toml | grep -c "^\],"  # Should be 16

# Test base24 conversion
thag_convert_themes_alt --input base24-theme.yaml -o /tmp/test.toml
sed -n '/base_colors = \[/,/\]/p' /tmp/test.toml | grep -c "^\],"  # Should be 24
```

### Runtime Tests

```rust
#[test]
fn test_base_colors_loading() {
    let mut theme = Theme::load_from_file(Path::new("test-theme.toml")).unwrap();

    // Initially not loaded
    assert!(theme.base_colors.is_none());

    // Load on demand
    theme.load_base_colors().unwrap();
    assert!(theme.base_colors.is_some());

    let colors = theme.base_colors.as_ref().unwrap();
    assert_eq!(colors.len(), 16); // or 24 for base24

    // All RGB values valid
    for rgb in colors {
        assert!(rgb[0] <= 255);
        assert!(rgb[1] <= 255);
        assert!(rgb[2] <= 255);
    }
}
```

### ANSI Mapping Tests

```rust
#[test]
fn test_ansi_mapping_with_base_colors() {
    let mut theme = Theme::get_builtin("catppuccin-mocha").unwrap();
    theme.load_base_colors().unwrap();

    let color_map = PaletteSync::build_ansi_color_map(&theme);

    // Should use base_colors
    assert_eq!(color_map.len(), 16);

    // First color should be base00 (background)
    let base00 = theme.base_colors.as_ref().unwrap()[0];
    assert_eq!(color_map[0], base00);
}
```

## Migration Guide

### For Existing Themes

Existing themes **do not need modification**. They will:

1. Load normally without `base_colors`

2. Use role-based ANSI mapping as fallback

3. Work exactly as before

To add `base_colors`:

1. Re-convert from original base16/24 source, or

2. Manually add the array to your theme file, or

3. Use a theme editor tool (future)

### For Applications

Applications using `thag_styling` **require no changes**. To use `base_colors`:

```rust
// Before (still works)
let theme = Theme::load_from_file(path)?;
use_theme(&theme);

// After (opt-in to base_colors)
let mut theme = Theme::load_from_file(path)?;
if theme.load_base_colors().is_ok() {
    // Can now use theme.base_colors if present
}
use_theme(&theme);
```

## Conclusion

The `base_colors` feature elegantly solves the semantic vs. ANSI divide by:

- **Preserving** original base16/24 palettes
- **Optimizing** memory usage with lazy loading
- **Maintaining** backward compatibility
- **Enabling** perfect terminal theme synchronization

It's a minimal, zero-overhead addition that provides maximum value for ANSI terminal mapping while opening doors for future enhancements like interactive theme editing and palette analysis tools.

## References

- [Base16 Specification](https://github.com/chriskempson/base16)
- [Base24 Specification](https://github.com/Base24/base24)
- [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors)
- Implementation: `thag_styling/src/styling.rs` (Theme struct and load_base_colors method)
- Converters: `src/bin/thag_convert_themes{,_alt}.rs`
- Usage: `src/bin/thag_sync_palette.rs`
