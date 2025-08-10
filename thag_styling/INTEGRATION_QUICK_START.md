# Thag Styling Integration Quick Start

Get started with thag_styling's theme-aware terminal styling in under 5 minutes!

## 🚀 Zero Configuration Setup

### 1. Add to Your Project

```toml
# Cargo.toml
[dependencies]
thag_styling = { version = "0.2.0", features = ["crossterm_support", "color_detect", "config"] }
crossterm = "0.28.1"
```

### 2. Use It Immediately

```rust
use thag_styling::{Role, ThemedStyle};
use crossterm::style::ContentStyle;

fn main() {
    // That's it! Colors automatically adapt to your terminal theme
    let success = ContentStyle::themed(Role::Success);
    let error = ContentStyle::themed(Role::Error);
    
    println!("{}", success.apply("✓ Success!"));
    println!("{}", error.apply("✗ Error!"));
}
```

## 📋 Feature Selection

Choose the integrations you need:

```toml
[dependencies.thag_styling]
version = "0.2.0"
features = [
    "color_detect",         # Essential: Enables terminal color detection
    "config",               # Essential: Enables sophisticated theme selection
    "crossterm_support",    # Cross-platform terminal (recommended)
    "ratatui_support",      # Terminal UI applications
    "nu_ansi_term_support", # Nu shell / reedline
    "console_support",      # Console styling library
    "inquire_theming",      # Interactive prompts
    "full",                 # Everything (for experimentation)
]
```

## 🎨 The ThemedStyle Trait

Works identically across all supported libraries:

```rust
use thag_styling::{Role, ThemedStyle};

// Same API, different libraries
let ratatui_style = ratatui::style::Style::themed(Role::Warning);
let crossterm_style = crossterm::style::ContentStyle::themed(Role::Warning);
let nu_style = nu_ansi_term::Style::themed(Role::Warning);
```

## 🔄 Drop-in Replacements

### Before (hardcoded colors):
```rust
// ❌ Not theme-aware, may be invisible on some backgrounds
let error_style = ratatui::style::Style::default()
    .fg(ratatui::style::Color::Red);

let success_color = crossterm::style::Color::Green;
```

### After (theme-aware):
```rust
// ✅ Automatically adapts to terminal theme and capabilities
let error_style = ratatui::style::Style::themed(Role::Error);
let success_color = crossterm::style::Color::themed(Role::Success);
```

## 🎯 Common Patterns

### 1. Semantic Roles
```rust
use thag_styling::Role;

// Use semantic meaning, not colors
Role::Success    // ✓ Operations that succeed
Role::Error      // ✗ Critical problems
Role::Warning    // ⚠ Potential issues  
Role::Info       // ℹ Informational messages
Role::Code       // Code snippets
Role::Emphasis   // Important text
Role::Heading1   // Main headers
Role::Normal     // Regular text
Role::Subtle     // Less important details
```

### 2. Extending Existing Styles
```rust
use thag_styling::{ThemedStyle, CrosstermStyleExt};

let base = ContentStyle::default().bold().italic();
let themed = base.with_role(Role::Success);  // Add theme-aware colors
```

### 3. Interactive Prompts (Zero Config)
```rust
#[cfg(feature = "inquire_theming")]
{
    use inquire::Text;
    
    let name = Text::new("Your name:")
        .with_render_config(thag_styling::themed_inquire_config())
        .prompt()?;
}
```

## 📚 Library-Specific Examples

<details>
<summary><strong>🔧 Crossterm</strong></summary>

```rust
use crossterm::{execute, style::Print};
use crossterm::style::{ContentStyle, Color};
use thag_styling::{Role, ThemedStyle, ThemedStylize};

// Method 1: Direct styling
let success = ContentStyle::themed(Role::Success);
let error = Color::themed(Role::Error);

// Method 2: Fluent API
let styled = "Hello".role(Role::Emphasis);

// Method 3: Helper functions
use thag_styling::integrations::crossterm_integration::crossterm_helpers;
execute!(
    std::io::stdout(),
    Print(crossterm_helpers::success_style().apply("Done!"))
)?;
```
</details>

<details>
<summary><strong>📊 Ratatui</strong></summary>

```rust
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use thag_styling::{Role, ThemedStyle, RatatuiStyleExt};

// Themed widget styling
let block = Block::default()
    .title("Status")
    .style(Style::themed(Role::Heading2))
    .border_style(Style::themed(Role::Subtle));

// Combining styles
let text_style = Style::default()
    .bold()
    .with_role(Role::Success);  // Add themed colors
```
</details>

<details>
<summary><strong>🐚 Nu-ANSI-Term</strong></summary>

```rust
use nu_ansi_term::{Style, Color};
use thag_styling::{Role, ThemedStyle};
use thag_styling::integrations::nu_ansi_term_integration::reedline_helpers;

// Direct usage
let success = Style::themed(Role::Success);
println!("{}", success.paint("Success!"));

// Reedline integration
let prompt_style = reedline_helpers::prompt_style();
let error_style = reedline_helpers::error_style();
```
</details>

<details>
<summary><strong>🖥️ Console</strong></summary>

```rust
use console::{Style, Term};
use thag_styling::{Role, ThemedStyle, TermThemedExt};
use thag_styling::integrations::console_integration::console_helpers;

// Themed styling
let success = Style::themed(Role::Success);
println!("{}", success.apply_to("Done!"));

// Terminal extensions
let term = Term::stdout();
term.write_line_themed(Role::Error, "Error occurred!")?;

// Helper functions
console_helpers::print_themed(Role::Info, "Information")?;
```
</details>

## 🔧 Advanced Features

### Runtime Theme Detection
```rust
use thag_styling::TermAttributes;

let attrs = TermAttributes::get_or_init();
println!("Terminal: {} colors, {} background, using theme '{}'", 
    attrs.color_support,
    attrs.term_bg_luma,
    attrs.theme.name
);
```

### Manual Theme Selection
```rust
use thag_styling::{TermAttributes, ColorSupport};

// Override automatic detection if needed
let attrs = TermAttributes::initialize_with_theme(
    "gruvbox_dark", 
    ColorSupport::TrueColor
)?;
```

### Performance Tips
```rust
// ✅ Good: Create styles once, reuse
let success_style = Style::themed(Role::Success);
for item in items {
    println!("{}", success_style.apply(&item));
}

// ❌ Avoid: Creating styles in loops
for item in items {
    println!("{}", Style::themed(Role::Success).apply(&item));  // Inefficient
}
```

## 🎨 Theme Gallery

Your application automatically gets professional themes:
- `gruvbox_dark` / `gruvbox_light` - Warm, retro color scheme
- `solarized_dark` / `solarized_light` - Precision colors for machines and people  
- `dracula` - Dark theme with vivid colors
- `nord` - Arctic, north-bluish clean and elegant
- `tomorrow_night` - Pastel colors on dark background
- `base16_*` - Extensive collection of carefully crafted themes

## ❓ Troubleshooting

### Colors Not Appearing?

**Most common issue**: Missing required features. You need both `color_detect` and `config`:

```toml
[dependencies.thag_styling]
features = ["color_detect", "config", "your_integration_support"]
```

Without these features, thag_styling falls back to basic ANSI colors instead of rich themed colors.

Check what was detected:
```bash
# Check what thag detected
cargo run --example zero_config --features "full"
```

### Compilation Errors?
```toml
# Make sure you have the required core features plus your integration
[dependencies.thag_styling]
features = ["color_detect", "config", "crossterm_support"]

[dependencies.crossterm]
version = "0.28.1"  # Make sure you have the actual library too
```

### Performance Issues?
```toml
# Use minimal features for production (but keep color_detect + config for good colors)
[dependencies.thag_styling]
features = ["color_detect", "config", "crossterm_support"]
```

## 🌟 Benefits

- **Zero Configuration**: Works out of the box (with `color_detect` + `config` features)
- **Background Sensitive**: Colors adapt to light/dark terminals
- **Rich Theme Selection**: Automatic selection from 200+ professional themes
- **Performance Optimized**: Cached color calculations
- **Cross-Library Consistent**: Same colors everywhere
- **Capability Aware**: Automatically uses best available colors
- **Professional Themes**: Carefully designed color schemes

## ⚠️ Important: Required Features

For the best experience with rich themed colors, always include:
- `color_detect` - Enables automatic terminal capability detection
- `config` - Enables sophisticated theme selection from the full theme database

Without these features, thag_styling falls back to basic ANSI colors instead of the rich themed colors shown in examples.

## 📖 Next Steps

- [View full examples](examples/)
- [Integration documentation](src/integrations/README.md)
- [Add your own integration](src/integrations/README.md#adding-new-integrations)
- [Browse themes](themes/)
- [Performance guide](docs/performance.md)

Ready to make your terminal applications beautiful? Just add `ThemedStyle::themed(Role::YourRole)` and enjoy automatic theme-aware styling! 🎨