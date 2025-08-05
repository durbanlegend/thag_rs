# ANSI Styling Support Macro

The `ansi_styling_support!` macro provides a complete ANSI terminal styling solution in a single function-like macro invocation. This eliminates the need for mystery imports while giving you full control over your styling implementation.

## Overview

Instead of requiring users to import traits like `AnsiStyleExt`, the `ansi_styling_support!` macro generates all necessary code inline, including:

- `Color` enum with basic ANSI colors
- `Effect` enum for text styling (bold, italic, underline, reversed)
- `Styled` struct for chaining style operations
- `AnsiStyleExt` trait for `.style()` method on strings
- Complete implementations with ANSI escape code generation

## Usage

### Basic Setup

```rust
use thag_proc_macros::{ansi_styling_support, styled};

// Generate all styling support - call once per module/crate
ansi_styling_support! {}

fn main() {
    // Now you can use both the trait methods and styled! macro
    println!("{}", "Hello".style().bold().fg(Color::Red));
    println!("{}", styled!("World", fg = Blue, underline));
}
```

### Available Colors

- `Black`, `Red`, `Green`, `Yellow`
- `Blue`, `Magenta`, `Cyan`, `White`

### Available Effects

- `bold` - Bold text
- `italic` - Italic text  
- `underline` - Underlined text
- `reversed` - Reversed foreground/background

### Styling Methods

#### Using the `styled!` macro:
```rust
styled!("text", bold)                    // Single effect
styled!("text", fg = Red)                // Color only
styled!("text", bold, fg = Blue)         // Combined
styled!("text", italic, underline, fg = Green, reversed)  // Multiple effects
```

#### Using trait methods:
```rust
"text".style().bold()                    // Single effect
"text".style().fg(Color::Red)            // Color only
"text".style().bold().fg(Color::Blue)    // Chained
```

### Advanced Features

#### Embedding Styled Content
```rust
let outer = "outer ".style().fg(Color::Red);
let inner = "inner".style().fg(Color::Green);
println!("{}{} world", outer, outer.embed(inner));
```

## When to Use This Pattern

### ✅ Use `ansi_styling_support!` when:
- You want standalone ANSI styling without external dependencies
- You prefer explicit, self-contained code generation
- You don't need the full `thag_styling` theme system
- You want to avoid "mystery imports"

### ❌ Consider `thag_styling` instead when:
- You need advanced theming capabilities
- You want role-based styling (Error, Warning, etc.)
- You need color palette management
- You want terminal capability detection

## Comparison with Other Approaches

| Approach | Import Required | Self-Contained | Themes | Complexity |
|----------|----------------|----------------|---------|------------|
| `ansi_styling_support!` | None | ✅ | ❌ | Low |
| `thag_styling` prelude | `use thag_styling::prelude::*` | ❌ | ✅ | Medium |
| Custom implementation | Custom trait | ✅ | Custom | High |

## Examples

See the demo files:
- `demo/ansi_styling_support.rs` - Pure styling showcase
- `demo/proc_macro_styled.rs` - Mixed with `thag_styling` for advanced features

## Implementation Notes

The macro generates approximately 150 lines of code per invocation, but this is compile-time overhead with zero runtime cost. The generated code includes proper ANSI reset sequences to avoid terminal state pollution.

This pattern follows the same philosophy as `file_navigator!` - providing complete, self-contained functionality through a single macro invocation rather than requiring complex dependency management.