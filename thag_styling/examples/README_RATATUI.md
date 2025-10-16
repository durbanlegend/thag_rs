# Ratatui Integration Example

This directory contains a comprehensive example demonstrating how to integrate `thag_styling` with the `ratatui` terminal UI library.

## Example: `ratatui_theming_showcase.rs`

This example showcases a full-featured TUI application that demonstrates:

### Key Features Demonstrated

- **Automatic Theme Detection**: The app automatically detects your terminal's capabilities and applies an appropriate theme
- **Semantic Role-Based Styling**: Uses semantic roles (`Role::Error`, `Role::Success`, etc.) instead of hardcoded colors
- **ThemedStyle Trait**: Shows how to use `Style::themed(Role::Error)` for consistent theming
- **Extension Methods**: Demonstrates `RatatuiStyleExt` with methods like `base_style.with_role(Role::Warning)`
- **Comprehensive Widget Coverage**: Examples for tabs, lists, gauges, paragraphs, and more

### Running the Example

```bash
# Basic run - themes automatically detected
cargo run -p thag_styling --example ratatui_theming_showcase -p thag_styling

# Or from the demo directory
cargo run demo/ratatui_integration_demo.rs
```

### Navigation

- **Tab/→**: Next tab
- **Shift+Tab/←**: Previous tab
- **1-4**: Direct tab access
- **↑/↓**: Scroll content
- **h/F1**: Toggle help
- **q**: Quit

### Application Tabs

1. **📊 Dashboard**: System metrics with progress bars, CPU/Memory/Network status
2. **📝 Logs**: Application log entries with different severity levels
3. **🎛️ Settings**: Theme and display configuration
4. **📋 About**: Information about the integration and code examples

### Code Patterns Demonstrated

#### Using ThemedStyle Trait

```rust
use thag_styling::integrations::ThemedStyle;

// Create themed styles directly
let error_style = Style::themed(Role::Error);
let success_color = Color::themed(Role::Success);
```

#### Using Extension Methods

```rust
use thag_styling::integrations::RatatuiStyleExt;

// Extend existing styles with theme-aware roles
let base_style = Style::default().bold();
let themed_style = base_style.with_role(Role::Warning);
```

#### Widget Styling Examples

```rust
// Themed progress bar
let progress_bar = Gauge::default()
    .gauge_style(Style::themed(Role::Success))
    .block(Block::default()
        .border_style(Style::themed(Role::Subtle))
        .title_style(Style::themed(Role::Heading3)));

// Themed list with role-based log entries
let log_items: Vec<ListItem> = entries.iter().map(|entry| {
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(timestamp, Style::themed(Role::Subtle)),
            Span::styled(icon, Style::themed(entry.level)),
            Span::styled(message, Style::themed(entry.level)),
        ])
    ])
}).collect();
```

### Theme Roles Used

The example demonstrates styling with these semantic roles:

- `Role::Heading1`, `Role::Heading2`, `Role::Heading3` - Hierarchical headings
- `Role::Error` - Error messages and critical status
- `Role::Warning` - Warnings and cautions
- `Role::Success` - Success messages and positive status
- `Role::Info` - Informational content
- `Role::Code` - Code snippets and technical data
- `Role::Emphasis` - Highlighted or important text
- `Role::Normal` - Standard text content
- `Role::Subtle` - De-emphasized text like borders and timestamps

### Benefits of This Approach

1. **Consistent Theming**: All components automatically use colors that work well together
2. **Terminal Compatibility**: Automatically adapts to terminal capabilities (true color, 256 color, basic)
3. **Theme Switching**: Easy to change themes without modifying widget code
4. **Semantic Meaning**: Roles convey meaning, not just appearance
5. **Maintainability**: Single place to define color schemes for all UI elements

### Testing

The example includes comprehensive tests covering:

- App state management
- Navigation functionality
- Theme integration APIs
- Extension trait methods

Run tests with:
```bash
cargo test --example ratatui_theming_showcase -p thag_styling
```

### Integration Requirements

To use `thag_styling` with `ratatui` in your own project:

1. **Dependencies** in `Cargo.toml`:
```toml
[dependencies]
thag_styling = { version = "0.2", features = ["ratatui_support"] }
ratatui = "0.29"
```

2. **Imports**:
```rust
use thag_styling::{Role, integrations::{ThemedStyle, RatatuiStyleExt}};
use ratatui::style::{Style, Color};
```

3. **Usage**:
```rust
// Direct theming
let style = Style::themed(Role::Error);

// Extension methods
let themed = Style::default().bold().with_role(Role::Success);
```

This integration provides a seamless way to build beautiful, theme-aware TUI applications that automatically adapt to different terminal environments while maintaining consistent, semantic styling throughout your interface.
