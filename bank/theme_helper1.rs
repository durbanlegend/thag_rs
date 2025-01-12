use std::env;

#[derive(Debug)]
struct Theme {
    name: &'static str,
    background: &'static str,
    foreground: &'static str,
    description: &'static str,
    xterm_background: u8,
    xterm_foreground: u8,
}

fn print_styled(theme: &str, style: &str, text: &str) {
    // Normalize theme name for matching
    let theme_key = theme.to_lowercase().replace(' ', "-");

    let style_code = match (theme_key.as_str(), style) {
        // Dracula theme styles
        ("dracula", "heading1") => "\x1b[1;38;5;212m", // Bold Pink
        ("dracula", "heading2") => "\x1b[1;38;5;141m", // Bold Purple
        ("dracula", "subheading") => "\x1b[1;38;5;117m", // Bold Cyan
        ("dracula", "error") => "\x1b[38;5;203m",      // Red
        ("dracula", "warning") => "\x1b[38;5;228m",    // Yellow
        ("dracula", "success") => "\x1b[38;5;84m",     // Green
        ("dracula", "info") => "\x1b[38;5;117m",       // Cyan
        ("dracula", "emphasis") => "\x1b[1;38;5;141m", // Bold Purple
        ("dracula", "bright") => "\x1b[38;5;117m",     // Cyan
        ("dracula", "normal") => "\x1b[38;5;253m",     // Light Gray
        ("dracula", "ghost") => "\x1b[2;38;5;244m",    // Dim Light Gray
        ("dracula", "debug") => "\x1b[3;38;5;245m",    // Italic Medium Gray
        ("dracula", "trace") => "\x1b[2;38;5;244m",    // Dim Light Gray

        // Gruvbox Light Hard theme styles
        ("gruvbox-light-hard", "heading1") => "\x1b[1;38;5;124m", // Bold Red
        ("gruvbox-light-hard", "heading2") => "\x1b[1;38;5;100m", // Bold Green
        ("gruvbox-light-hard", "subheading") => "\x1b[1;38;5;172m", // Bold Orange
        ("gruvbox-light-hard", "error") => "\x1b[38;5;160m",      // Bright Red
        ("gruvbox-light-hard", "warning") => "\x1b[38;5;214m",    // Bright Yellow
        ("gruvbox-light-hard", "success") => "\x1b[38;5;142m",    // Bright Green
        ("gruvbox-light-hard", "info") => "\x1b[38;5;66m",        // Bright Blue
        ("gruvbox-light-hard", "emphasis") => "\x1b[1;38;5;126m", // Bold Purple
        ("gruvbox-light-hard", "bright") => "\x1b[38;5;72m",      // Bright Aqua
        ("gruvbox-light-hard", "normal") => "\x1b[38;5;239m",     // Dark Gray
        ("gruvbox-light-hard", "ghost") => "\x1b[38;5;245m",      // Medium Gray
        ("gruvbox-light-hard", "debug") => "\x1b[3;38;5;166m",    // Italic Orange
        ("gruvbox-light-hard", "trace") => "\x1b[38;5;246m",      // Gray

        _ => panic!("Not found"),
    };
    println!("{}{}\x1b[0m", style_code, text);
}

const DRACULA: Theme = Theme {
    name: "Dracula",
    background: "#282a36",
    foreground: "#f8f8f2",
    description: "Dark theme with vibrant colors",
    xterm_background: 234,
    xterm_foreground: 253,
};

const GRUVBOX_LIGHT_HARD: Theme = Theme {
    name: "Gruvbox Light Hard",
    background: "#f9f5d7",
    foreground: "#3c3836",
    description: "Light theme with high contrast and warm colors",
    xterm_background: 230,
    xterm_foreground: 237,
};

fn demonstrate_theme_styles(theme_name: &str) {
    println!("\nStyle Preview:");
    println!("-------------");

    print_styled(theme_name, "heading1", "Main Heading");
    print_styled(theme_name, "heading2", "Secondary Heading");
    print_styled(theme_name, "subheading", "Subheading");
    println!();

    print_styled(theme_name, "error", "Error: Critical failure detected");
    print_styled(theme_name, "warning", "Warning: Proceed with caution");
    print_styled(theme_name, "success", "Success: Operation completed");
    print_styled(theme_name, "info", "Info: Standard information");
    println!();

    print_styled(theme_name, "emphasis", "Emphasized important text");
    print_styled(theme_name, "bright", "Bright highlighted text");
    print_styled(theme_name, "normal", "Normal regular text");
    print_styled(theme_name, "ghost", "Ghost text (de-emphasized)");
    println!();

    print_styled(theme_name, "debug", "Debug: Diagnostic information");
    print_styled(theme_name, "trace", "Trace: Detailed execution path");
}

fn print_theme_info(theme: &Theme) {
    println!("\n{} Theme", theme.name);
    println!("{}", "=".repeat(theme.name.len() + 6));
    println!("{}\n", theme.description);

    println!("Terminal Colors:");
    println!(
        "Background: {} (256-color: {})",
        theme.background, theme.xterm_background
    );
    println!(
        "Foreground: {} (256-color: {})",
        theme.foreground, theme.xterm_foreground
    );
    println!();

    demonstrate_theme_styles(theme.name.to_lowercase().as_str());
}

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Terminal Theme Helper");
    println!("====================");

    if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "dracula" => print_theme_info(&DRACULA),
            "gruvbox-light-hard" => print_theme_info(&GRUVBOX_LIGHT_HARD),
            "list" => {
                println!("\nAvailable themes:");
                println!("- dracula");
                println!("- gruvbox-light-hard");
            }
            _ => println!("Unknown theme. Use 'list' to see available themes."),
        }
    } else {
        println!("\nUsage:");
        println!("  theme-helper <theme-name>");
        println!("  theme-helper list");
    }
}
