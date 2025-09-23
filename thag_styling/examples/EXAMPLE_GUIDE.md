# thag_styling Examples Guide

This directory contains comprehensive examples demonstrating `thag_styling` integration with various terminal libraries.

## Quick Start

All examples require appropriate feature flags to be enabled:

```bash
# Ratatui integration showcase
cargo run --example ratatui_theming_showcase --features "ratatui_support" -p thag_styling

# Basic styling demo (no features required)
cargo run --example basic_styling -p thag_styling

# Integration comparisons
cargo run --example themed_style_demo --features "ratatui_support,crossterm_support" -p thag_styling
```

## Feature Requirements

| Example | Required Features | Description |
|---------|------------------|-------------|
| `ratatui_theming_showcase` | `ratatui_support` | Full TUI application demo |
| `ratatui_integration_test` | `ratatui_support` | Integration testing/debugging |
| `themed_style_demo` | Library-specific | Multi-library comparison |
| `basic_styling` | None | Core functionality only |

## Ratatui Integration

### `ratatui_theming_showcase.rs`

**Features:** `ratatui_support`

A comprehensive TUI application demonstrating:
- 4-tab interface (Dashboard, Logs, Settings, About)
- All major ratatui widgets with semantic theming
- Interactive navigation and help system
- Both `ThemedStyle` trait and extension method usage

**Run:**
```bash
cargo run --example ratatui_theming_showcase --features "ratatui_support" -p thag_styling
```

**Navigation:**
- `Tab/→` - Next tab
- `Shift+Tab/←` - Previous tab
- `1-4` - Direct tab access
- `h/F1` - Toggle help
- `q` - Quit

### Demo Script

For quick demonstrations, use the simplified demo in the main project:

```bash
# From project root
cargo run demo/ratatui_integration_demo.rs
```

This provides the same theming capabilities in a more compact example.

## Integration Patterns

### Using ThemedStyle Trait

```rust
use thag_styling::integrations::ThemedStyle;
use ratatui::style::{Style, Color};

// Direct semantic theming
let error_style = Style::themed(Role::Error);
let success_color = Color::themed(Role::Success);
```

### Using Extension Methods

```rust
use thag_styling::integrations::RatatuiStyleExt;

// Flexible composition
let themed_style = Style::default()
    .bold()
    .with_role(Role::Warning);
```

### Widget Integration

```rust
use thag_styling::Role;
use ratatui::widgets::{Block, Borders, Gauge};

// Theme-aware widgets
let gauge = Gauge::default()
    .gauge_style(Style::themed(Role::Success))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::themed(Role::Subtle))
        .title_style(Style::themed(Role::Heading2)));
```

## Benefits Demonstrated

1. **Automatic Coordination** - All colors work harmoniously together
2. **Terminal Adaptation** - Automatically uses best available color mode
3. **Semantic Clarity** - Code expresses meaning, not just appearance
4. **Easy Maintenance** - Change themes without touching widget code
5. **Professional Results** - Consistent, polished appearance

## Testing

Run example tests:
```bash
cargo test --example ratatui_theming_showcase --features "ratatui_support" -p thag_styling
```

## Dependencies for Your Projects

Add to your `Cargo.toml`:

```toml
[dependencies]
thag_styling = { version = "0.2", features = ["ratatui_support"] }
ratatui = "0.29"
crossterm = "0.28"  # for terminal handling
```

## Color Detection

All examples use `thag_styling`'s default color detection, which automatically:
- Detects terminal capabilities (TrueColor, 256-color, 16-color)
- Selects appropriate theme for terminal background
- Falls back gracefully on limited terminals
- Maintains consistent appearance across environments

No manual theme configuration needed - it just works!

## Troubleshooting

**"function or associated item not found"**
- Ensure you're using `--features "ratatui_support"`
- Check that imports include `use thag_styling::integrations::ThemedStyle;`

**Colors don't appear themed**
- Verify terminal supports colors with `echo $TERM`
- Try running in a modern terminal (Terminal.app, iTerm2, etc.)
- Color detection is automatic but may need terminal restart

**Example won't compile**
- Ensure you're running from the correct directory
- Use `-p thag_styling` flag when running examples
- Check that ratatui version matches (0.29+)

For more help, see the main [thag_styling documentation](../README.md) or the integration-specific README files.