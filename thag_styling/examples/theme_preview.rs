//! Preview a selection of built-in themes side by side.
//!
//! Each theme is rendered without modifying the global `TermAttributes` singleton:
//! [`TermAttributes::build_from_strategy`] creates an owned instance for each theme,
//! and [`TermAttributes::with_context`] scopes it so that role-based styling inside
//! the closure automatically uses the right theme via [`TermAttributes::current`].
//!
//! This is the intended pattern for theme preview UIs, test fixtures that need
//! predictable output, or any code that needs to temporarily operate under a
//! different theme without touching global state.
//!
//! Run with: `cargo run -p thag_styling --example theme_preview`

use thag_styling::{ColorInitStrategy, Role, Style, TermAttributes};

/// A cross-section of built-in themes: basic, dark TrueColor, light, and classics.
const THEMES: &[&str] = &["basic_dark", "dracula", "github", "solarized-dark", "nord"];

fn main() {
    // Detect the real terminal once to get colour support and bg luma.
    // All previews will use the terminal's actual capabilities.
    let base = TermAttributes::build_from_strategy(&ColorInitStrategy::Match);
    let color_support = base.color_support;
    println!("Terminal colour support: {color_support}\n");

    for &name in THEMES {
        // Clone the base attrs and swap only the theme, keeping the detected
        // colour support so colours are remapped appropriately.
        let preview = match base.clone().with_theme(name, color_support) {
            Ok(attrs) => attrs,
            Err(e) => {
                eprintln!("Skipping {name}: {e}");
                continue;
            }
        };

        // Push this TermAttributes onto the thread-local context stack.
        // Code inside the closure sees it via TermAttributes::current(),
        // but the global singleton is unchanged.
        preview.with_context(render_sample);
    }
}

/// Renders a mock CLI build report using role-based styling.
///
/// Deliberately avoids naming a theme — it simply calls the role-based API
/// and lets `TermAttributes::current()` (set by the enclosing `with_context`)
/// supply the right colours. This is exactly how real application code works.
fn render_sample() {
    let attrs = TermAttributes::current();
    let name = &attrs.theme.name;
    // Top bar pads to at least 44 columns total; bottom mirrors the exact same width.
    let name_cols = name.chars().count();
    let top_fill = 44usize.saturating_sub(name_cols).max(4);
    let bar = "─".repeat(top_fill);
    // "┌─ {name} {bar}" = 1+1+1 + name_cols + 1 + top_fill chars; bottom gets the rest.
    let bottom = "─".repeat(name_cols + top_fill + 3);

    println!("┌─ {name} {bar}");
    println!(
        "│ {}",
        Style::for_role(Role::Heading1).paint("my_app  v0.3.0")
    );
    println!(
        "│ {}  compiled in 1.4 s",
        Style::for_role(Role::Success).paint("✓  47 tests passed")
    );
    println!(
        "│ {}  use `bar()` instead of `foo()`",
        Style::for_role(Role::Warning).paint("⚠  deprecated")
    );
    println!(
        "│ {}  missing `}}` at src/main.rs:42",
        Style::for_role(Role::Error).paint("✗  syntax error")
    );
    println!(
        "│ {}  target/debug/my_app",
        Style::for_role(Role::Info).paint("ℹ  linking")
    );
    println!(
        "│    {}",
        Style::for_role(Role::Code).paint("cargo build --release --quiet")
    );
    println!("└{bottom}");
    println!();
}
