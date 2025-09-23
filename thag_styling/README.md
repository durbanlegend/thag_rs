# thag_styling

[![Crates.io](https://img.shields.io/crates/v/thag_styling)](https://crates.io/crates/thag_styling)
[![Documentation](https://docs.rs/thag_styling/badge.svg)](https://docs.rs/thag_styling)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**A semantic terminal styling system with automatic theme detection and rich color palette support.**

`thag_styling` provides a comprehensive solution for terminal applications that need consistent, professional theming. Instead of hardcoding colors, you define content by *semantic meaning* — errors, warnings, code, headings — and the library automatically applies coordinated colors that work beautifully together.

## Features

- **🎨 Semantic Styling** — Style content by meaning (`Role::Error`, `Role::Success`) rather than specific colors
- **🔍 Automatic Theme Detection** — Detects terminal capabilities and applies appropriate themes automatically
- **🎯 Rich Color Palettes** — Uses coordinated 15-color palettes from the full TrueColor or 256-color spectrum
- **📚 Library Integration** — Seamless integration with popular terminal libraries (ratatui, crossterm, console, nu-ansi-term)
- **♿ Accessibility First** — Ensures legibility with tested contrast ratios using proven theme colors
- **⚡ Zero Runtime Cost** — All theme resolution happens at compile time or initialization

## Quick Start

Add `thag_styling` to your `Cargo.toml`:

```toml
[dependencies]
thag_styling = "0.2"
```

Basic usage:

```rust
use thag_styling::{paint_for_role, Role};

fn main() {
    // Semantic styling - colors automatically coordinated
    println!("{}", paint_for_role(Role::Success, "✅ Operation completed successfully"));
    println!("{}", paint_for_role(Role::Error, "❌ Connection failed"));
    println!("{}", paint_for_role(Role::Warning, "⚠️  High memory usage detected"));
    println!("{}", paint_for_role(Role::Code, "cargo run --release"));
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
println!("{}", paint_for_role(Role::Error, "Error"));
println!("{}", paint_for_role(Role::Warning, "Warning"));
println!("{}", paint_for_role(Role::Success, "Success"));
// Colors automatically coordinated ✨
```

### 👀 **Assured Legibility**
All themes use proven color combinations from established terminal themes, ensuring text remains readable across different terminal backgrounds and lighting conditions.

### ⚡ **Reduced Development Time**
Focus on your application logic instead of color theory. Define content semantically once, and `thag_styling` handles the visual presentation across all terminal environments.

## Library Integrations

`thag_styling` integrates seamlessly with popular terminal libraries:

### Ratatui Integration

```rust
use thag_styling::integrations::ThemedStyle;
use ratatui::style::Style;

// Theme-aware TUI components
let error_style = Style::themed(Role::Error);
let success_gauge = Gauge::default().gauge_style(Style::themed(Role::Success));

// Add to Cargo.toml:
// thag_styling = { version = "0.2", features = ["ratatui_support"] }
```

### Crossterm Integration

```rust
use thag_styling::integrations::crossterm_integration::crossterm_helpers;

// Pre-configured styles for common use cases
let prompt_style = crossterm_helpers::prompt_style();
let error_style = crossterm_helpers::error_style();

// Add to Cargo.toml:
// thag_styling = { version = "0.2", features = ["crossterm_support"] }
```

### Console Integration

```rust
use thag_styling::integrations::ThemedStyle;
use console::Style;

let themed_style = Style::themed(Role::Warning);

// Add to Cargo.toml:
// thag_styling = { version = "0.2", features = ["console_support"] }
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

### StyledString for Complex Styling
```rust
use thag_styling::{StyledStringExt, Styleable};

// Chain styling methods naturally
format!("Status: {} and {}",
        "success".success(),
        "warning".warning())
    .info()
    .println();

// Verbosity-controlled output
format!("Debug: {}", "value".code())
    .debug()
    .vprintln(thag_styling::Verbosity::Debug);
```

### Styled Print Macros
```rust
use thag_styling::{sprtln, svprtln, Role, Verbosity};

// Direct styled printing
sprtln!(Role::Error, "Connection failed: {}", error_msg);

// Verbosity-controlled styled printing  
svprtln!(Role::Debug, Verbosity::Debug, "Processing item {}", item_id);

// Legacy names (cprtln, cvprtln) still supported for backward compatibility
```

### Theme Management
```rust
use thag_styling::{TermAttributes, ColorInitStrategy};

// Initialize with specific strategy
TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

// Display theme information
thag_styling::display_theme_details();
thag_styling::display_theme_roles();
```

## Examples

The library includes comprehensive examples:

- **Basic Usage** — Simple semantic styling
- **Ratatui Showcase** — Full TUI application with themed components
- **Library Integrations** — Usage patterns with different terminal libraries
- **Custom Themes** — Creating and loading custom color schemes

Run examples:
```bash
# Styling migration guide (includes StyledString examples)
cargo run demo/styling_migration_guide.rs

# Interactive TUI showcase
cargo run --example ratatui_theming_showcase --features "ratatui_support" -p thag_styling

# Basic theming demo
cargo run demo/ratatui_integration_demo.rs

# View all available examples
ls thag_styling/examples/
```

## Documentation

- **[API Documentation](https://docs.rs/thag_styling)** — Complete API reference
- **[Integration Guide](examples/)** — Library-specific integration examples
- **[Theme Creation](themes/)** — Custom theme development guide

## Contributing

Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md) for details.

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
