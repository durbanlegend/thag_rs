/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog", "tools"] }

[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Displays the current theme palette and attributes.
///
/// E.g. `thag_theme` or `thag src/bin/thag_theme.rs`
//# Purpose: Show current theme.
//# Categories: ansi, color, styling, terminal, theming, tools, xterm
use thag_rs::{auto_help, help_system::check_help_and_exit};
use thag_styling::{
    display_terminal_attributes, display_theme_details, display_theme_roles, sprtln,
    ColorInitStrategy, Role, Style, TermAttributes,
};

fn main() {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    let term_attrs = TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);
    let theme = &term_attrs.theme;

    print!("\t");
    sprtln!(
        Style::from(Role::NORM).underline(),
        "Current theme on this terminal\x1b[24m: {}\n",
        Style::from(Role::HD1).underline().paint(&theme.name)
    );
    display_theme_roles(theme);
    display_theme_details(theme);
    display_terminal_attributes(theme);
}
