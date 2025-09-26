# thag_styling

[![Crates.io](https://img.shields.io/crates/v/thag_styling)](https://crates.io/crates/thag_styling)
[![Documentation](https://docs.rs/thag_styling/badge.svg)](https://docs.rs/thag_styling)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**A semantic terminal styling system for Rust applications across platforms.**

**Uses the current terminal theme to automatically select the right color and style for each message type - headings, errors, code, normal etc.**

**Extend and enhance existing terminal themes with automatic color detection and extensive tool integration.**

**Works with the most popular terminal emulators.**

**Generate gorgeous `thag_styling` and terminal themes automatically from your favourite artwork.**

`thag_styling` builds upon the foundation of popular terminal themes like Solarized, Gruvbox, Dracula, Nord, and Base16 variants, providing a comprehensive library of **290+ curated themes** plus over a dozen original creations. Instead of hardcoding colors, you define content by *semantic meaning* (errors, warnings, code, headings) and the library automatically applies coordinated 15-color palettes that work beautifully across all terminal environments. Includes powerful tools for creating stunning new themes and exporting them to popular terminal emulators.

![thag-raphael-school-of-athens-dark](/Users/donf/projects/thag_rs/docs/thag_styling/assets/thag-raphael-school-of-athens-dark.png)
<!--(https://durbanlegend.github.io/thag_rs/thag_styling/assets/thag-raphael-school-of-athens-dark.png)]-->

## Features

### Core Styling System
- **🚦 Semantic Roles** — Style by meaning (`Role::Error`, `Role::Success`) not colors
- **🔍 Automatic Detection** — Terminal capabilities and theme selection
- **🎨 Rich Palettes** — Coordinated 15-color schemes from TrueColor/256-color spectrum
- **👓 Proven Legibility** — Based on tested terminal themes with proper contrast
- **🪶 Zero Overhead** — Compile-time theme resolution

### Styling APIs
- **`StyledString`** — Fluent method chaining: `"text".error().println()`
- **Styled Macros** — Direct printing: `sprtln!(Role::Warning, "Alert: {}", msg)`
- **Paint Functions** — Functional style: `paint_for_role(Role::Code, code)`
- **ANSI Macros** — Enhanced styling: `styled!("text", fg = "#ff0000", bold)`

### Library Integrations
- **Ratatui** — Complete TUI theming with `Style::themed(Role::Error)`
- **Crossterm** — Terminal manipulation with themed helpers
- **Console** — Popular styling library integration
- **Nu-ANSI-Term** — Shell and REPL theming support

### Theme Ecosystem
- **300+ Theme Library** — Solarized, Gruvbox, Dracula, Nord, Monokai, and many more
- **Theme Generation** — From images using advanced color extraction
- **Multi-Format Export** — Alacritty, iTerm2, Kitty, Mintty (Git Bash and Cygwin), WezTerm, Windows Terminal. These and additional OSC-compatible terminals are also supported by the `thag_sync_palette` command below, which can be invoked from the terminal profile file, e.g. Apple terminal, VS Code and Zed.
- **Palette Sync** — Runtime terminal palette updates via OSC sequences
- **Base16/Base24** — Converts standard Base16/Base24 themes to thag format (290+ themes included)

## Quick Start

Add `thag_styling` to your `Cargo.toml`:

```toml
[dependencies]
thag_styling = "0.2"
```

Basic usage:

```rust
use thag_styling::{Styleable, StyledPrint};

fn main() {
    // Fluent method chaining - natural and composable
    "✅ Operation completed successfully".success().println();
    "❌ Connection failed".error().println();
    "⚠️  High memory usage detected".warning().println();
    "cargo run --release".code().println();

    // Works with any Display type
    42.success().println();              // Numbers
    std::path::Path::new("/usr/bin").display().code().println(); // Paths
}
```

## Why thag_styling?

### 🎨 **Effortless Aesthetics**
Instead of manually selecting and coordinating colors, `thag_styling` provides carefully curated 15-color palettes that work harmoniously together. Each palette is designed with proper contrast ratios and visual hierarchy.

**Before:**
```rust
// Manual color coordination - error prone
println!("{}", "Error".red().bold());
println!("{}", "Warning".yellow());
println!("{}", "Success".green());
// Do these colors work well together? 🤷
```

**After:**
```rust
// Semantic coordination - guaranteed harmony
"Error".error().println();
"Warning".warning().println();
"Success".success().println();
// Perfect color coordination automatically ✨
```

### 👀 **Assured Legibility**
All themes use proven color combinations from established terminal themes, ensuring text remains readable across different terminal backgrounds and lighting conditions.

### ⚡ **Reduced Development Time**
Focus on your application logic instead of color theory. Define content semantically once, and `thag_styling` handles the visual presentation across all terminal environments.

## Library Integrations

`thag_styling` integrates seamlessly with popular terminal libraries:

### Ratatui TUI Integration

```rust
use thag_styling::integrations::ThemedStyle;
use ratatui::{style::Style, widgets::Gauge};

// Semantic TUI styling
let error_style = Style::themed(Role::Error);
let success_gauge = Gauge::default()
    .gauge_style(Style::themed(Role::Success))
    .block(Block::default()
        .border_style(Style::themed(Role::Subtle))
        .title_style(Style::themed(Role::Heading2)));

// Cargo.toml: features = ["ratatui_support"]
```

### Multiple API Styles

```rust
// Method chaining (recommended)
format!("Status: {} | Memory: {}", "OK".success(), "85%".warning())
    .info().println();

// Styled print macros
sprtln!(Role::Error, "Connection failed: {}", error_msg);
svprtln!(Role::Debug, Verbosity::Debug, "Processing {}", item);

// Functional style
println!("{}", paint_for_role(Role::Code, "fn main()"));

// Low-level ANSI with enhanced color support
use thag_styling::styled;
println!("{}", styled!("Alert", fg = Red, bold, underline));          // Basic ANSI
println!("{}", styled!("Custom", fg = Rgb(255, 165, 0), italic));     // RGB orange
println!("{}", styled!("Palette", fg = Color256(93), bold));          // 256-color purple
println!("{}", styled!("Hex", fg = "#ff6347", underline));            // Hex tomato
```

## Semantic Roles

`thag_styling` provides a comprehensive set of semantic roles:

| Role | Purpose | Example Usage |
|------|---------|---------------|
| `Heading1`, `Heading2`, `Heading3` | Document structure | Section titles, command help headers |
| `Error`, `Warning`, `Success` | Status messages | Operation results, validation feedback |
| `Info` | General information | Status updates, notifications |
| `Code` | Technical content | Commands, file paths, identifiers |
| `Emphasis` | Important content | Key points, highlighted text |
| `Normal` | Default text | Body content, descriptions |
| `Subtle` | De-emphasized content | Timestamps, metadata, borders |
| `Quote`, `Link` | Special content | Citations, URLs |
| `Debug` | Development info | Diagnostic output, verbose logging |

## Terminal Compatibility

`thag_styling` automatically adapts to your terminal's capabilities:

- **TrueColor terminals** (16M colors) — Full RGB palette with rich gradients
- **256-color terminals** — Carefully mapped colors maintaining theme integrity
- **Basic terminals** (16 colors) — Graceful fallback to standard ANSI colors
- **Monochrome terminals** — Uses styling attributes (bold, italic, underline)

## Advanced Features

### Complex Nested Styling
```rust
use thag_styling::{Styleable, StyledPrint};

// Unlimited nesting with method chaining
format!("Server {} responded with {} in {}ms",
        server_name.emphasis(),
        format!("HTTP {}", status_code.success()),  // Nested styling
        latency.code())
    .info()
    .println();

// Verbosity-controlled output
format!("Debug: {}", diagnostic_data.code())
    .debug()
    .vprintln(Verbosity::Debug);
```

### Theme Generation from Images
```rust
// Generate themes from any image (requires 'image_themes' feature)
use thag_styling::image_themes::generate_theme_from_image;

let theme = generate_theme_from_image("sunset.jpg", "my-sunset-theme")?;
theme.save_to_file("themes/my-sunset-theme.toml")?;
```

### Multi-Format Theme Export
```rust
use thag_styling::{export_theme_to_file, ExportFormat};

let theme = Theme::load_from_file("my-theme.toml")?;

// Export to various terminal formats (alphabetically)
export_theme_to_file(&theme, "alacritty.toml", ExportFormat::Alacritty)?;
export_theme_to_file(&theme, "iterm2.itermcolors", ExportFormat::ITerm2)?;
export_theme_to_file(&theme, "kitty.conf", ExportFormat::Kitty)?;
export_theme_to_file(&theme, "mintty.config", ExportFormat::Mintty)?;
export_theme_to_file(&theme, "wezterm.toml", ExportFormat::WezTerm)?;
export_theme_to_file(&theme, "windows-terminal.json", ExportFormat::WindowsTerminal)?;
```

### Runtime Palette Synchronization
```rust
use thag_styling::PaletteSync;

// Sync terminal palette with theme colors (programmatically)
let sync = PaletteSync::new()?;
sync.apply_theme_palette(&theme)?;
// Terminal colors now match your theme!
```

For terminals that don't support theme files (like Apple Terminal), you can use shell integration to automatically sync palettes on startup:

**Unix shells (~/.bashrc or ~/.zshrc):**
```bash
if [[ "$TERM_PROGRAM" == "Apple_Terminal" ]]; then
    export THAG_COLOR_MODE=256
    export THAG_THEME=thag-botticelli-birth-of-venus-dark
    thag_sync_palette apply $THAG_THEME
elif [[ "$TERM_PROGRAM" == "WezTerm" ]]; then
    export THAG_COLOR_MODE=truecolor
    export THAG_THEME=thag-raphael-school-of-athens-dark
    thag_sync_palette apply $THAG_THEME
fi
```

**Windows PowerShell ($PROFILE):**
```powershell
$env:PATH += ";C:\Users\my_name\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin"
$env:THAG_COLOR_MODE = "truecolor"
$env:THAG_THEME = "thag-morning-coffee-light"
thag_sync_palette apply $THAG_THEME
```

## Examples

The library includes comprehensive examples:

- **Basic Usage** — Simple semantic styling
- **Ratatui Showcase** — Full TUI application with themed components
- **Library Integrations** — Usage patterns with different terminal libraries
- **Custom Themes** — Creating and loading custom color schemes

Run examples:
```bash
# Complete styling system demonstration
cargo run demo/styling_migration_guide.rs

# Interactive TUI showcase (full-featured application)
cargo run --example ratatui_theming_showcase --features "ratatui_support" -p thag_styling

# Quick integration demo
cargo run demo/ratatui_integration_demo.rs

# Theme generation from images
cargo run demo/image_theme_generation.rs

# Enhanced styled! macro demonstration
cargo run demo/styled_macro_enhanced.rs

# View all examples
ls thag_styling/examples/ demo/
```

## Theme Management Tools

`thag_styling` includes a comprehensive suite of command-line tools for theme management. These tools are part of the main `thag_rs` project and provide powerful theme generation, conversion, and management capabilities.

### Installation

The tools are installed as part of `thag_rs` with the `tools` feature:

```bash
# Install from crates.io with tools
cargo install thag_rs --features tools

# Install from GitHub repository
cargo install --git https://github.com/durbanlegend/thag_rs thag_rs --features tools

# Or build from source
git clone https://github.com/durbanlegend/thag_rs
cd thag_rs
cargo install --path . --features tools
```

**Note**: The tools are part of `thag_rs`, not `thag_styling` directly. This allows them to integrate the full `thag_rs` ecosystem while being available to `thag_styling` users.

### Theme Generation Tools
```bash
# Generate theme from image
thag_image_to_theme sunset.jpg my-sunset-theme

# Convert between theme formats
thag_convert_themes input-theme.toml output-format

# Generate terminal emulator themes
thag_gen_terminal_themes my-theme.toml --all-formats
```

### Theme Management Tools
```bash
# Display available themes
thag_show_themes

# Apply theme and sync terminal palette
thag_theme my-theme-name
thag_sync_palette

# Show terminal palette and compare with current theme
thag_palette

# Compare terminal palette with selected theme
thag_palette_vs_theme

# Add themes to specific terminals (alphabetically)
thag_alacritty_add_theme my-theme.toml
thag_mintty_add_theme my-theme.toml      # Cygwin, Git Bash
thag_winterm_add_theme my-theme.toml     # Windows Terminal
```

### Theme Development
```bash
# Detect terminal capabilities
thag_detect_term

# Test theme legibility
thag_legible my-theme.toml
```

## Integration Examples

### Complete TUI Application
See `examples/ratatui_theming_showcase.rs` for a 4-tab TUI demonstrating:
- Dashboard with metrics and progress bars
- Log viewer with semantic severity colors
- Settings and configuration display
- Comprehensive help system

All styled consistently using semantic roles.

### Library Integration Patterns
```rust
// Ratatui - semantic TUI theming
let gauge = Gauge::default()
    .gauge_style(Style::themed(Role::Success))
    .label("Progress");

// Crossterm - terminal control with theming
let styled_text = crossterm_helpers::success_style()
    .apply("Operation completed");

// Console - popular styling library integration
let warning = console::Style::themed(Role::Warning)
    .apply_to("High CPU usage");

// Enhanced styled! macro with multiple color formats
println!("{}", styled!("Error", fg = Rgb(220, 50, 47), bold));        // RGB
println!("{}", styled!("Warning", fg = Color256(214), underline));    // 256-color
println!("{}", styled!("Success", fg = "#00ff00", italic));           // Hex
println!("{}", styled!("Info", fg = Blue, reversed));                 // Basic ANSI
```

## Documentation

- **[API Documentation](https://docs.rs/thag_styling)** — Complete API reference
- **[Integration Guide](examples/)** — Library-specific examples and patterns
- **[Theme Development](themes/)** — Custom theme creation and tools
- **[Tool Reference](src/bin/README.md)** — Command-line tool documentation

## Contributing

Contributions welcome! Areas needing help:
- Additional terminal emulator export formats
- More library integrations (owo-colors, indicatif, etc.)
- Theme collection expansion
- Performance optimizations

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
