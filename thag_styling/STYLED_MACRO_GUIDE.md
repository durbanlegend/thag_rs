# Enhanced `styled!` Macro Quick Reference

The `styled!` macro provides low-level ANSI styling with support for multiple color formats and text effects.

## Basic Usage

```rust
use thag_proc_macros::{ansi_styling_support, styled};

// Enable ANSI styling support
ansi_styling_support! {}

// Basic usage
println!("{}", styled!("Hello, world!", fg = Red, bold));
```

## Color Formats

### 1. Basic ANSI Colors (8 colors)
Uses the terminal's color palette - works everywhere but limited selection.

```rust
styled!("Text", fg = Red)        // Red from terminal palette
styled!("Text", fg = Green)      // Green from terminal palette
styled!("Text", fg = Blue)       // Blue from terminal palette
```

**Available colors**: `Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`

### 2. 256-Color Palette
Uses the standard 256-color palette (0-255) - widely supported, good variety.

```rust
styled!("Text", fg = Color256(196))  // Bright red
styled!("Text", fg = Color256(214))  // Orange
styled!("Text", fg = Color256(93))   // Purple
```

**Color ranges**:
- 0-15: Standard colors + bright variants
- 16-231: 216-color cube (6×6×6)
- 232-255: Grayscale ramp

### 3. True RGB Colors
Exact color values (0-255 per component) - most control, requires modern terminals.

```rust
styled!("Text", fg = Rgb(255, 0, 0))     // Bright red
styled!("Text", fg = Rgb(255, 165, 0))   // Orange
styled!("Text", fg = Rgb(138, 43, 226))  // Blue violet
```

### 4. Hex Colors
Familiar web-style hex colors - converted to RGB at compile time.

```rust
styled!("Text", fg = "#ff0000")  // Red
styled!("Text", fg = "#ffa500")  // Orange
styled!("Text", fg = "#8a2be2")  // Blue violet
```

**Format**: Must be exactly 6 hex digits after `#`

## Text Effects

All effects can be combined:

```rust
styled!("Text", bold)                                    // Bold
styled!("Text", italic)                                  // Italic
styled!("Text", underline)                              // Underlined
styled!("Text", reversed)                               // Inverted colors
styled!("Text", bold, italic, underline)                // Multiple effects
styled!("Text", fg = "#ff0000", bold, italic, reversed) // Color + effects
```

## Practical Examples

### Error Messages
```rust
println!("{}", styled!("ERROR:", fg = Rgb(220, 50, 47), bold));
println!("{}", styled!("  File not found", fg = Color256(167)));
```

### Success Messages
```rust
println!("{}", styled!("SUCCESS:", fg = "#00ff00", bold));
println!("{}", styled!("  Build completed", fg = Color256(107)));
```

### Code Syntax Highlighting
```rust
println!("{}{}{}{}",
    styled!("fn ", fg = "#859900", bold),     // Keyword
    styled!("main", fg = "#268bd2"),          // Function name  
    styled!("() {", fg = "#93a1a1"),          // Punctuation
    styled!("}", fg = "#93a1a1")              // Punctuation
);
```

## Color Format Comparison

| Format | Example | Pros | Cons | Terminal Support |
|--------|---------|------|------|------------------|
| Basic ANSI | `fg = Red` | Universal compatibility | Limited to 8 colors | All terminals |
| 256-color | `fg = Color256(196)` | Good variety, wide support | Limited color precision | Most modern terminals |
| RGB | `fg = Rgb(255, 0, 0)` | Exact colors, 16.7M options | Requires true color support | Modern terminals |
| Hex | `fg = "#ff0000"` | Familiar web format | Requires true color support | Modern terminals |

## Terminal Compatibility

- **Basic ANSI**: Works on all terminals
- **256-color**: Supported by most terminals since ~2010
- **RGB/Hex**: Requires "true color" or "24-bit color" support
  - iTerm2, Terminal.app, WezTerm, Alacritty, Windows Terminal: ✅
  - Legacy terminals, basic terminal emulators: ❌

## Best Practices

1. **Use Basic ANSI** for maximum compatibility
2. **Use 256-color** for good balance of variety and compatibility  
3. **Use RGB/Hex** when you need exact colors and know your target terminals
4. **Combine effects** sparingly to avoid visual clutter
5. **Test on your target terminals** to ensure colors render correctly

## Integration with thag_styling

The `styled!` macro is complementary to thag_styling's semantic system:

```rust
// Semantic styling (recommended for applications)
"Error message".error().println();

// Direct styling (useful for specific color needs)  
println!("{}", styled!("Custom color", fg = "#ff6347", bold));
```

Use `styled!` when you need:
- Specific colors not available in themes
- One-off styling without semantic meaning
- Low-level control over ANSI sequences
- Integration with existing ANSI-based code

Use thag_styling's semantic system when you need:
- Consistent theming across an application
- Automatic terminal adaptation
- Theme switching capabilities
- Coordinated color palettes