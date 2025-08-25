/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Displays the current theme palette and attributes.
///
/// E.g. `thag demo/thag_theme.rs`
//# Purpose: Show current theme.
//# Categories: tools
use thag_styling::{cprtln, display_theme_details, display_theme_roles, ColorInitStrategy, Role, Style, TermAttributes, Theme, V};

let term_attrs = TermAttributes::initialize(&ColorInitStrategy::Match);
let theme = &term_attrs.theme;

print!("\t");
cprtln!(Style::from(Role::NORM).underline(), "Current theme on this terminal\x1b[24m: {}\n", Style::from(Role::HD1).underline().paint(&theme.name));
display_theme_roles(theme);
display_theme_details(theme);
