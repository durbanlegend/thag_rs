use std::env;
use std::error::Error;

#[derive(Debug)]
struct Theme {
    name: &'static str,
    background: &'static str,
    foreground: &'static str,
    description: &'static str,
    xterm_background: u8,
    xterm_foreground: u8,
    styles: &'static [(&'static str, &'static str)], // (name, example)
}

const DRACULA: Theme = Theme {
    name: "Dracula",
    background: "#282a36",
    foreground: "#f8f8f2",
    description: "Dark theme with vibrant colors",
    xterm_background: 234,
    xterm_foreground: 253,
    styles: &[
        ("heading1", "Main Heading"),
        ("heading2", "Secondary Heading"),
        ("subheading", "Subheading"),
        ("error", "Error: Critical failure"),
        ("warning", "Warning: Proceed with caution"),
        ("emphasis", "Important information"),
        ("bright", "Highlighted text"),
        ("normal", "Regular text"),
        ("ghost", "De-emphasized text"),
        ("debug", "Debug information"),
    ],
};

const GRUVBOX_LIGHT: Theme = Theme {
    name: "Gruvbox Light",
    background: "#fbf1c7",
    foreground: "#3c3836",
    description: "Light theme with warm, retro colors",
    xterm_background: 230,
    xterm_foreground: 237,
    styles: DRACULA.styles, // Same style names, different colors
};

const GRUVBOX_LIGHT_HARD: Theme = Theme {
    name: "Gruvbox Light Hard",
    background: "#f9f5d7", // Harder contrast background
    foreground: "#3c3836",
    description: "Light theme with high contrast and warm colors",
    xterm_background: 230,
    xterm_foreground: 237,
    styles: &[
        // Headers and Structure
        ("heading1", "\x1b[1;38;5;124m"),   // Bold Red
        ("heading2", "\x1b[1;38;5;100m"),   // Bold Green
        ("subheading", "\x1b[1;38;5;172m"), // Bold Orange
        // Alerts and Status
        ("error", "\x1b[38;5;160m"),   // Bright Red
        ("warning", "\x1b[38;5;214m"), // Bright Yellow
        ("success", "\x1b[38;5;142m"), // Bright Green
        ("info", "\x1b[38;5;66m"),     // Bright Blue
        // Emphasis Levels
        ("emphasis", "\x1b[1;38;5;126m"), // Bold Purple
        ("bright", "\x1b[38;5;72m"),      // Bright Aqua
        ("normal", "\x1b[38;5;239m"),     // Dark Gray
        ("ghost", "\x1b[38;5;245m"),      // Medium Gray
        // Debug and Development
        ("debug", "\x1b[3;38;5;166m"), // Italic Orange
        ("trace", "\x1b[38;5;246m"),   // Gray
    ],
};

fn get_terminal_type() -> Option<String> {
    env::var("TERM").ok()
}

enum TerminalEnv {
    Xorg,
    Wayland,
    AppleTerminal,
    ITerm,
    Tmux,
    Pure,
    Unknown,
}

fn detect_environment() -> TerminalEnv {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        TerminalEnv::Wayland
    } else if env::var("DISPLAY").is_ok() {
        TerminalEnv::Xorg
    } else if env::var("TERM_PROGRAM").map_or(false, |t| t == "Apple_Terminal") {
        TerminalEnv::AppleTerminal
    } else if env::var("TERM_PROGRAM").map_or(false, |t| t == "iTerm.app") {
        TerminalEnv::ITerm
    } else if env::var("TMUX").is_ok() {
        TerminalEnv::Tmux
    } else if env::var("TERM").map_or(false, |t| t == "linux") {
        TerminalEnv::Pure
    } else {
        TerminalEnv::Unknown
    }
}

fn get_terminal_setup_instructions(theme: &Theme) -> String {
    match detect_environment() {
        TerminalEnv::Xorg => format!(
            "X11 Terminal Setup:\n\
             Option 1 - Using .Xresources (if your terminal supports it):\n\
             1. Add to ~/.Xresources:\n\
                *.background: {}\n\
                *.foreground: {}\n\
             2. Run: xrdb -merge ~/.Xresources\n\n\
             Option 2 - Direct terminal configuration:\n\
             - For GNOME Terminal: Edit > Preferences > Profiles\n\
             - For XFCE Terminal: Edit > Preferences\n\
             - For Konsole: Settings > Edit Current Profile\n\
             Set colors:\n\
             - Background: {}\n\
             - Foreground: {}\n",
            theme.background, theme.foreground, theme.background, theme.foreground
        ),

        TerminalEnv::Wayland => format!(
            "Wayland Terminal Setup:\n\
             For foot terminal:\n\
             Edit ~/.config/foot/foot.ini:\n\
             [colors]\n\
             background={}\n\
             foreground={}\n\n\
             For GNOME Terminal:\n\
             Open Terminal > Preferences > Profiles\n\
             Set colors manually:\n\
             - Background: {}\n\
             - Foreground: {}\n",
            theme.background, theme.foreground, theme.background, theme.foreground
        ),

        TerminalEnv::Pure => format!(
            "Terminal Setup (Console):\n\
             For Linux console, add to ~/.bashrc or similar:\n\
             echo -en \"\\e]P0{bg}\" # background\n\
             echo -en \"\\e]P7{fg}\" # foreground\n\
             \n\
             Or consider using a terminal emulator for full theme support.\n",
            bg = &theme.background[1..], // Remove the # from hex color
            fg = &theme.foreground[1..]
        ),

        TerminalEnv::Tmux => format!(
            "Tmux Terminal Setup:\n\
             1. Configure your terminal emulator using the appropriate instructions\n\
             2. Add to ~/.tmux.conf:\n\
             set -g default-terminal \"screen-256color\"\n\
             set -ga terminal-overrides \",*256col*:Tc\"\n\
             \n\
             Recommended terminal colors:\n\
             - Background: {}\n\
             - Foreground: {}\n",
            theme.background, theme.foreground
        ),

        TerminalEnv::ITerm => get_iterm2_instructions(theme),
        _ => format!(
            "Generic Terminal Setup:\n\
             Look for Color settings in your terminal's Preferences or Settings menu.\n\
             Recommended colors:\n\
             - Background: {} (256-color: {})\n\
             - Foreground: {} (256-color: {})\n",
            theme.background, theme.xterm_background, theme.foreground, theme.xterm_foreground
        ),
    }
}

fn get_iterm2_instructions(theme: &Theme) -> String {
    format!(
        "iTerm2 Setup Instructions:\n\
         \n\
         1. Open iTerm2 Preferences (Cmd + ,)\n\
         2. Go to Profiles tab\n\
         3. Select your profile (create new if needed)\n\
         4. Go to Colors tab\n\
         5. Important: Set 'Minimum Contrast' to 0\n\
         6. Disable 'Smart box cursor color'\n\
         7. If using Dark theme:\n\
            - Uncheck 'Use Dark Background'\n\
            - Ensure 'Use built-in PowerLine glyphs' is unchecked\n\
         8. Click 'Color Presets...' dropdown:\n\
            - Select 'Custom'\n\
            - Set Background: {}\n\
            - Set Foreground: {}\n\
         9. Verify in Session menu:\n\
            - Session > Reset Colors to ensure no session override\n\
            - Session > Reset Profile to ensure profile is applied\n\
         \n\
         If colors still aren't correct:\n\
         - Check Terminal > Show Colors to verify actual colors\n\
         - Ensure no dynamic profiles are overriding (Profiles > Dynamic)\n\
         - Try creating a new window to get fresh settings\n",
        theme.background, theme.foreground
    )
}
fn demonstrate_theme_styles(theme: &Theme) {
    println!("\nStyle Preview:");
    println!("-------------");
    for (style, example) in theme.styles {
        // This would use your actual styling logic
        println!("{}: {}", style, example);
    }
}

fn print_theme_info(theme: &Theme) {
    println!("\n{} Theme", theme.name);
    println!("{}", "=".repeat(theme.name.len() + 6));
    println!("{}\n", theme.description);

    println!("Setup Instructions:");
    println!("{}", get_terminal_setup_instructions(theme));

    demonstrate_theme_styles(theme);
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    println!("Terminal Theme Helper");
    println!("====================");

    if args.len() > 1 {
        // Handle specific theme request
        match args[1].to_lowercase().as_str() {
            "dracula" => print_theme_info(&DRACULA),
            "gruvbox-light" => print_theme_info(&GRUVBOX_LIGHT),
            "list" => {
                println!("\nAvailable themes:");
                println!("- dracula");
                println!("- gruvbox-light");
            }
            _ => println!("Unknown theme. Use 'list' to see available themes."),
        }
    } else {
        println!("\nUsage:");
        println!("  theme-helper <theme-name>");
        println!("  theme-helper list");
    }

    Ok(())
}
