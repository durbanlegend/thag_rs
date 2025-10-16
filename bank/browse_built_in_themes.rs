/*[toml]
[dependencies]
# thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog", "tools"] }
thag_styling = { version = "0.2, thag-auto", default-features = false, features = ["inquire_theming"] }
*/

use thag_styling::{StylingResult, TermAttributes, Theme};
use inquire::Select;

fn select_builtin_theme() -> Option<String> {
    // Initialize terminal attributes to ensure styling works
    let _term_attrs = TermAttributes::get_or_init();
    // eprintln!("_term_attrs={_term_attrs:#?}");

    let mut themes = Theme::list_builtin();
    themes.sort();

    // Create theme options with descriptions
    let theme_options: Vec<String> = themes
        .iter()
        .map(|theme_name| {
            let theme = Theme::get_builtin(theme_name).unwrap_or_else(|_| {
                // Fallback in case theme can't be loaded
                Theme::get_builtin("none").expect("Failed to load fallback theme")
            });
            format!("{} - {}", theme_name, theme.description)
        })
        .collect();

    // Clear screen initially
    print!("\x1b[2J\x1b[H");

    let mut cursor = 0_usize;
    use inquire::error::InquireResult;
    use inquire::list_option::ListOption;

    let maybe_theme_name = loop {
        println!("\n🎨 Interactive Theme Browser");
        println!("{}", "═".repeat(80));
        println!("📚 {} themes available", themes.len());
        println!("💡 Start typing to filter themes by name");
        println!("{}", "═".repeat(80));

        let selection: InquireResult<ListOption<String>> = Select::new(
            "🔍 Select a theme to preview:",
            theme_options.clone(),
        )
        .with_page_size(24)
        .with_help_message(
            "↑↓, PageUp, PageDown: navigate • type to filter • Enter to select • Esc to quit",
        )
        .with_reset_cursor(false)
        .with_starting_cursor(cursor)
        .raw_prompt();

        match selection {
            Ok(selected) => {
                cursor = selected.index;

                // Extract theme name from selection (before the " - " separator)
                let theme_name = selected
                    .value
                    .split(" - ")
                    .next()
                    .unwrap_or(&selected.value);
                return Some(theme_name.to_string());
            }
            Err(inquire::InquireError::OperationCanceled) => {
                print!("\x1b[2J\x1b[H");
                println!("👋 Thanks for using the theme browser!");
                break None;
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                break None;
            }
        }
    };
    maybe_theme_name
}

let maybe_theme_name = select_builtin_theme();
println!("Selected maybe_theme_name={maybe_theme_name:?}");
