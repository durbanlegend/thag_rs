# Runtime Theme Loading

This document describes the runtime theme loading feature in `thag_styling`, which allows users to load custom themes from user-specified directories at runtime, in addition to the built-in themes compiled into the binary.

## Overview

The runtime theme loading system provides a flexible way to extend thag's theming capabilities:

1. **Built-in themes**: Themes compiled into the binary (existing functionality)
2. **Runtime themes**: Themes loaded from user directories at runtime (new functionality)
3. **Seamless fallback**: Automatic fallback from runtime to built-in themes

## Configuration

### Method 1: Environment Variables

Set the `THAG_THEME_DIR` environment variable to specify where your custom themes are stored:

```bash
export THAG_THEME_DIR="/Users/username/.config/thag/themes"
```

Then use `THAG_THEME` to select a theme by name:

```bash
export THAG_THEME="my-custom-theme"
```

### Method 2: Configuration File

Add a `theme_dir` setting to your thag configuration file (`~/.config/thag/config.toml`):

```toml
[styling]
theme_dir = "/Users/username/.config/thag/themes"

# Theme preferences (works with both builtin and custom themes)
preferred_dark = ["my-custom-dark", "dracula", "nord"]
preferred_light = ["my-custom-light", "github", "one-light"]
```

### Priority Order

Theme resolution follows this priority:

1. **THAG_THEME_DIR** environment variable (highest priority)
2. **theme_dir** configuration setting
3. **Built-in themes** (fallback)

## Theme File Format

Custom themes use the same TOML format as built-in themes. Here's a minimal example:

```toml
name = "my-custom-theme"
description = "A custom theme description"
term_bg_luma = "dark"  # or "light"
min_color_support = "true_color"  # or "color256", "basic"
backgrounds = ["#1e1e1e", "#2d2d2d"]

[palette.normal]
rgb = [248, 248, 242]

[palette.error]
rgb = [255, 85, 85]

[palette.success]
rgb = [80, 250, 123]

[palette.warning]
rgb = [255, 184, 108]

[palette.info]
rgb = [139, 233, 253]

[palette.emphasis]
rgb = [255, 121, 198]

[palette.code]
rgb = [241, 250, 140]

[palette.subtle]
rgb = [98, 114, 164]

[palette.hint]
rgb = [98, 114, 164]

[palette.debug]
rgb = [68, 71, 90]

[palette.link]
rgb = [139, 233, 253]

[palette.quote]
rgb = [241, 250, 140]

[palette.commentary]
rgb = [98, 114, 164]

[palette.heading1]
rgb = [255, 107, 107]

[palette.heading2]
rgb = [78, 205, 196]

[palette.heading3]
rgb = [69, 183, 209]
```

### Required Fields

All theme files must include:
- `name`: Theme name
- `description`: Human-readable description
- `term_bg_luma`: "light" or "dark"
- `min_color_support`: "basic", "color256", or "true_color"
- `backgrounds`: Array of hex color strings
- `palette`: Complete color palette with all roles

### Color Formats

Colors can be specified in multiple formats:

1. **RGB arrays** (recommended for true color themes):
   ```toml
   [palette.normal]
   rgb = [248, 248, 242]
   ```

2. **ANSI colors** (for basic themes):
   ```toml
   [palette.normal]
   ansi = "white"
   index = 7
   ```

3. **256-color indexes** (for color256 themes):
   ```toml
   [palette.normal]
   color256 = 15
   ```

## File Naming Patterns

The system searches for theme files using these patterns (in order):

1. `{theme-name}.toml`
2. `thag-{theme-name}.toml`
3. `thag-{theme-name}-light.toml` (if theme name doesn't end in `-light` or `-dark`)
4. `thag-{theme-name}-dark.toml` (if theme name doesn't end in `-light` or `-dark`)

For example, if you request theme "custom", the system will look for:
- `custom.toml`
- `thag-custom.toml`
- `thag-custom-light.toml`
- `thag-custom-dark.toml`

## Usage Examples

### Creating a Custom Theme Directory

```bash
# Create theme directory
mkdir -p ~/.config/thag/themes

# Create a custom theme
cat > ~/.config/thag/themes/my-theme.toml << 'EOF'
name = "my-theme"
description = "My custom theme"
term_bg_luma = "dark"
min_color_support = "true_color"
backgrounds = ["#282828"]

[palette.normal]
rgb = [235, 219, 178]
# ... (add all required palette entries)
EOF
```

### Using Environment Variables

```bash
# Set theme directory
export THAG_THEME_DIR="$HOME/.config/thag/themes"

# Use your custom theme
export THAG_THEME="my-theme"

# Run thag with custom theme
thag your-script.rs
```

### Using Configuration File

```toml
# ~/.config/thag/config.toml
[styling]
theme_dir = "/Users/username/.config/thag/themes"
```

Then use the theme with:
```bash
THAG_THEME="my-theme" thag your-script.rs
```

## Integration with Existing Features

### Auto-detection

Custom themes work seamlessly with thag's auto-detection features:

```toml
[styling]
theme_dir = "/Users/username/.config/thag/themes"
preferred_dark = ["my-custom-dark", "dracula"]  # Tries custom first, then builtin
preferred_light = ["my-custom-light", "github"]
```

### Color Support Conversion

Custom themes are automatically converted to match your terminal's color support:

- True color themes → downgraded to 256-color or basic as needed
- Colors are intelligently mapped to maintain visual consistency

### Theme Validation

All custom themes are validated on load:
- Required fields must be present
- Color values must be valid
- Color support must match theme requirements

## Error Handling

The system provides graceful error handling:

1. **Missing directory**: Falls back to built-in themes
2. **Invalid theme file**: Shows clear error message and falls back
3. **Missing theme**: Falls back to built-in theme with same name
4. **Validation errors**: Detailed error messages for debugging

## Best Practices

### Theme Organization

```
~/.config/thag/themes/
├── my-dark-themes/
│   ├── thag-my-theme-dark.toml
│   └── thag-another-dark.toml
├── my-light-themes/
│   ├── thag-my-theme-light.toml
│   └── thag-another-light.toml
└── experimental/
    └── work-in-progress.toml
```

### Theme Development

1. Start with an existing theme as a template
2. Test with different color support levels
3. Validate theme files before deployment
4. Use descriptive names and descriptions

### Performance

- Theme files are loaded on-demand
- Validation happens at load time
- Built-in themes remain fast (compiled-in)
- Directory scanning is optimized

## API Reference

### New Functions

```rust
// Load theme with runtime support
Theme::get_theme_runtime_or_builtin(theme_name: &str) -> StylingResult<Theme>

// Load theme with color support conversion
Theme::get_theme_runtime_or_builtin_with_color_support(
    theme_name: &str, 
    color_support: ColorSupport
) -> StylingResult<Theme>
```

### Configuration

```rust
// New field in Styling config
pub struct Styling {
    pub theme_dir: Option<String>,  // Path to custom themes directory
    // ... existing fields
}
```

## Troubleshooting

### Theme Not Found

1. Check `THAG_THEME_DIR` is set correctly
2. Verify theme file exists and has correct name
3. Check file permissions
4. Try with verbose output: `RUST_LOG=debug thag your-script.rs`

### Theme Loading Errors

1. Validate TOML syntax
2. Ensure all required palette entries are present
3. Check color format (use RGB arrays for true color)
4. Verify `min_color_support` matches color format

### Fallback Not Working

1. Ensure built-in theme exists
2. Check for typos in theme names
3. Verify configuration file syntax

## Migration Guide

### From Built-in Only

No changes required - existing functionality continues to work.

### Adding Custom Themes

1. Create theme directory
2. Convert existing themes to TOML format
3. Update configuration or environment variables
4. Test with `THAG_THEME` environment variable

### Converting External Themes

Use the existing theme conversion tools in `thag_styling` to convert from other formats to the thag TOML format.