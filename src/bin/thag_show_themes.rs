/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["tools"] }
*/

use inquire::{set_global_render_config, Select};
use std::{env, io};
use thag_rs::{
    auto_help, cprtln, display_theme_details, display_theme_roles, themed_inquire_config, Role,
    TermAttributes, ThagResult, Theme,
};
use thag_styling::styling;

/// Display built-in themes and their styling with terminal setup instructions
///
/// This tool helps you explore the available built-in themes and provides
/// terminal setup instructions for optimal display.
//# Purpose: Help get best use out of styling with built-in themes.
//# Categories: reference, technique, tools

fn print_usage() {
    println!("Usage:");
    println!("  thag_show_themes                Interactive theme browser (default)");
    println!("  thag_show_themes <theme-name>   Show details for a specific theme");
    println!("  thag_show_themes list           List all available themes");
    println!("  thag_show_themes interactive    Interactive theme browser");
    println!("  thag_show_themes help           Show this help message");
}

fn list_themes() -> ThagResult<()> {
    let mut themes = Theme::list_builtin();
    themes.sort();

    println!("\nAvailable built-in themes:");
    println!("{}", "═".repeat(25));

    for theme_name in &themes {
        let theme = Theme::get_builtin(theme_name)?;
        println!("  {} - {}", theme_name, theme.description);
    }

    println!("\nTotal: {} themes available", themes.len());
    Ok(())
}

fn interactive_theme_browser() -> ThagResult<()> {
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

    loop {
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

                // Clear screen for better presentation
                print!("\x1b[2J\x1b[H");

                match show_theme(theme_name) {
                    Ok(()) => {
                        println!("\n{}", "═".repeat(80));
                        println!("🔙 Press Enter to return to theme browser, or Ctrl+C to exit...");
                        let _ = io::stdin().read_line(&mut String::new());
                        // Clear screen before returning to menu
                        print!("\x1b[2J\x1b[H");
                    }
                    Err(e) => {
                        println!("❌ Error displaying theme '{}': {}", theme_name, e);
                        println!("Press Enter to continue...");
                        let _ = io::stdin().read_line(&mut String::new());
                        print!("\x1b[2J\x1b[H");
                    }
                }
            }
            Err(inquire::InquireError::OperationCanceled) => {
                print!("\x1b[2J\x1b[H");
                println!("👋 Thanks for using the theme browser!");
                break;
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn show_theme(theme_name: &str) -> ThagResult<()> {
    let term_attrs = TermAttributes::get_or_init();

    let theme = Theme::get_theme_with_color_support(theme_name, term_attrs.color_support)
        .unwrap_or_else(|_| {
            // Fallback in case theme can't be loaded
            Theme::get_builtin("none").expect("Failed to load fallback theme")
        });

    // Initialize terminal attributes to ensure styling works
    let _term_attrs = TermAttributes::get_or_init();

    println!("\n{} Theme", theme.name);
    println!("{}", "═".repeat(theme.name.len() + 6));
    println!("{}", theme.description);

    // Display the theme's role styles
    display_theme_roles(&theme);

    // Display theme details
    display_theme_details(&theme);

    // Provide terminal setup instructions
    show_terminal_instructions(&theme);

    Ok(())
}

fn show_terminal_instructions(theme: &Theme) {
    cprtln!(
        theme.style_for(Role::HD2).bold(),
        "\t{}",
        "Terminal Setup Instructions:"
    );
    println!("\t{}", "─".repeat(80));

    if theme.backgrounds.is_empty() {
        println!("\tThis theme does not specify background colors.");
        println!("\tIt will work with any terminal background.");
        return;
    }

    let bg_color = &theme.backgrounds[0]; // Use first background color

    let palette = &theme.palette;
    let color_info = palette.normal.foreground.clone().unwrap();
    let value = color_info.value;

    #[allow(unused_variables)]
    let fg_rgb = match value {
        thag_styling::ColorValue::Basic { basic } => styling::index_to_rgb(color_info.index),
        thag_styling::ColorValue::Color256 { color256 } => styling::index_to_rgb(color256),
        thag_styling::ColorValue::TrueColor { rgb } => rgb.into(),
    };

    let fg = styling::rgb_to_hex(&fg_rgb);
    let bg = &bg_color;

    cprtln!(
        theme.style_for(Role::Normal),
        r#"        To view the colors clearly, set your terminal background to {bg} and foreground and cursor to {fg}."#
    );

    cprtln!(
        theme.style_for(Role::Normal),
        "\tCommand-line shortcut for *nix and other OSC-compliant terminals do this:"
    );
    cprtln!(
        theme.style_for(Role::Code),
        r#"
        printf "\x1b]10;{fg}\x07\x1b]12;{fg}\x07\x1b]11;{bg}\x07""#
    );
    println!();
    cprtln!(
        theme.style_for(Role::Normal),
        r#"        For best results, rather configure the desired theme in your terminal settings.
        To be sure that `thag` will identify the desired theme from the background color, edit your configuration with `thag -C`.
        Add this theme to the `preferred_dark` or `preferred_light` list above any other themes that use the same
        background, and save the configuration."#
    );
    println!();

    // Detect environment and provide specific instructions
    let instructions = get_terminal_setup_instructions(bg_color, &theme.term_bg_luma.to_string());
    cprtln!(theme.style_for(Role::Normal), "{}", instructions);
}

fn get_terminal_setup_instructions(bg_color: &str, luma: &str) -> String {
    let env_type = detect_environment();

    match env_type {
        TerminalEnv::ITerm => format!(
            "\tiTerm2 Setup:\n\
            \t1. Open iTerm2 Settings (Cmd + ,)\n\
            \t2. Go to Profiles → Colors\n\
            \t3. Recommended: Install desired theme from Color Presets per `https://iterm2colorschemes.com/`\n\
            \t4. Otherwise Set Background Color to {}.",
            bg_color,
            // luma.to_lowercase()
        ),

        TerminalEnv::AppleTerminal => format!(
            r#"	Apple Terminal Setup:
	1. Terminal → Settings → Profiles
	2. Recommended: Import .terminal file from bottom left drop-down menu.
	       See e.g. `https://github.com/lysyi3m/macos-terminal-themes/tree/master`.
    3. Select your profile or create new.
	4. Ensure Background Color is set to {}
	5. Recommended for {} backgrounds"#,
            bg_color,
            luma.to_lowercase()
        ),

        TerminalEnv::WezTerm => format!(
            r#"	WezTerm Setup:
	1. Edit ~/.wezterm.lua
	2. Refer to https://wezterm.org/config/appearance.html and https://wezterm.org/colorschemes/index.html
	3. Ensure background is set to {}
	4. Optimized for {} backgrounds"#,
            bg_color,
            luma.to_lowercase()
        ),

        TerminalEnv::GnomeTerminal => format!(
            r#"	GNOME Terminal Setup:
	1. Edit → Preferences → Profiles
	2. Select your profile
	3. Colors tab → Background Color: {}
	4. Optimized for {} backgrounds"#,
            bg_color,
            luma.to_lowercase()
        ),

        TerminalEnv::Generic => format!(
            "\tGeneric Terminal Setup:\n\
            \t1. Look for Color/Appearance settings in Preferences\n\
            \t2. Set Background Color to: {}\n\
            \t3. This theme works best with {} backgrounds\n\
            \t4. Ensure true color support is enabled if available",
            bg_color,
            luma.to_lowercase()
        ),
    }
}

#[derive(Debug)]
enum TerminalEnv {
    ITerm,
    AppleTerminal,
    GnomeTerminal,
    Generic,
    WezTerm,
}

fn detect_environment() -> TerminalEnv {
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "iTerm.app" => return TerminalEnv::ITerm,
            "Apple_Terminal" => return TerminalEnv::AppleTerminal,
            "WezTerm" => return TerminalEnv::WezTerm,
            _ => {}
        }
    }

    if env::var("GNOME_TERMINAL_SCREEN").is_ok() || env::var("GNOME_TERMINAL_SERVICE").is_ok() {
        return TerminalEnv::GnomeTerminal;
    }

    TerminalEnv::Generic
}

fn main() -> ThagResult<()> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_show_themes");
    let _ = &help.check_help();

    set_global_render_config(themed_inquire_config());

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Default to interactive mode when no arguments provided
        println!("🎨 Welcome to thag theme browser!");
        println!("Starting interactive mode...\n");
        interactive_theme_browser()?;
        return Ok(());
    }

    match args[1].to_lowercase().as_str() {
        "list" => {
            list_themes()?;
        }
        "interactive" | "i" => {
            interactive_theme_browser()?;
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        theme_name => match show_theme(theme_name) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Error: Could not load theme '{}': {}", theme_name, e);
                eprintln!("\nUse 'thag_show_themes list' to see available themes.");
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
