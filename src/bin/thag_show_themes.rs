use std::env;
use thag_rs::{
    auto_help,
    styling::{display_theme_details, display_theme_roles, TermAttributes, Theme},
    ThagResult,
};

/// Display built-in themes and their styling with terminal setup instructions
///
/// This tool helps you explore the available built-in themes and provides
/// terminal setup instructions for optimal display.
//# Purpose: Help get best use out of styling with built-in themes.
//# Categories: reference, technique, tools

fn print_usage() {
    println!("Usage:");
    println!("  thag_show_themes <theme-name>  Show details for a specific theme");
    println!("  thag_show_themes list          List all available themes");
    println!("  thag_show_themes help          Show this help message");
}

fn list_themes() -> ThagResult<()> {
    let mut themes = Theme::list_builtin();
    themes.sort();

    println!("\nAvailable built-in themes:");
    println!("{}", "=".repeat(25));

    for theme_name in &themes {
        let theme = Theme::get_builtin(theme_name)?;
        println!("  {} - {}", theme_name, theme.description);
    }

    println!("\nTotal: {} themes available", themes.len());
    Ok(())
}

fn show_theme(theme_name: &str) -> ThagResult<()> {
    let theme = Theme::get_builtin(theme_name)?;

    // Initialize terminal attributes to ensure styling works
    let _term_attrs = TermAttributes::get_or_init();

    println!("\n{} Theme", theme.name);
    println!("{}", "=".repeat(theme.name.len() + 6));
    println!("{}\n", theme.description);

    // Display the theme's role styles
    display_theme_roles(&theme);

    // Display theme details
    display_theme_details(&theme);

    // Provide terminal setup instructions
    show_terminal_instructions(&theme);

    Ok(())
}

fn show_terminal_instructions(theme: &Theme) {
    println!("\n\t{}", "Terminal Setup Instructions:");
    println!("\t{}", "─".repeat(80));

    if theme.backgrounds.is_empty() {
        println!("\tThis theme does not specify background colors.");
        println!("\tIt will work with any terminal background.");
        return;
    }

    let bg_color = &theme.backgrounds[0]; // Use first background color

    println!(
        "\tFor optimal display, set your terminal background to: {}",
        bg_color
    );
    println!();

    // Detect environment and provide specific instructions
    let instructions = get_terminal_setup_instructions(bg_color, &theme.term_bg_luma.to_string());
    println!("{}", instructions);
}

fn get_terminal_setup_instructions(bg_color: &str, luma: &str) -> String {
    let env_type = detect_environment();

    match env_type {
        TerminalEnv::ITerm => format!(
            "\tiTerm2 Setup:\n\
            \t1. Open iTerm2 Preferences (Cmd + ,)\n\
            \t2. Go to Profiles → Colors\n\
            \t3. Set Background Color to: {}\n\
            \t4. Recommended: Use {} background themes\n\
            \t5. Ensure 'Minimum Contrast' is set to 0\n",
            bg_color,
            luma.to_lowercase()
        ),

        TerminalEnv::AppleTerminal => format!(
            "\tApple Terminal Setup:\n\
            \t1. Terminal → Preferences → Profiles\n\
            \t2. Select your profile or create new\n\
            \t3. Set Background Color to: {}\n\
            \t4. Recommended for {} backgrounds\n",
            bg_color,
            luma.to_lowercase()
        ),

        TerminalEnv::GnomeTerminal => format!(
            "\tGNOME Terminal Setup:\n\
            \t1. Edit → Preferences → Profiles\n\
            \t2. Select your profile\n\
            \t3. Colors tab → Background Color: {}\n\
            \t4. Optimized for {} backgrounds\n",
            bg_color,
            luma.to_lowercase()
        ),

        TerminalEnv::Generic => format!(
            "\tGeneric Terminal Setup:\n\
            \t1. Look for Color/Appearance settings in Preferences\n\
            \t2. Set Background Color to: {}\n\
            \t3. This theme works best with {} backgrounds\n\
            \t4. Ensure true color support is enabled if available\n",
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
}

fn detect_environment() -> TerminalEnv {
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "iTerm.app" => return TerminalEnv::ITerm,
            "Apple_Terminal" => return TerminalEnv::AppleTerminal,
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

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].to_lowercase().as_str() {
        "list" => {
            list_themes()?;
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
