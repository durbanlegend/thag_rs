/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect", "image_themes"] }
*/
/// Terminal palette synchronization using OSC sequences,
///
/// This binary provides command-line access to `thag_styling`'s palette synchronization
/// functionality, allowing you to apply theme colors directly to your terminal's palette for convenience.
///
/// Note that it does not support `KDE Konsole` or `Mintty` terminal types because they do not support
/// the required OSC 4 ANSI escape sequences.
//# Purpose: Try out custom terminal themes or apply them dynamically, including for a session if incorporated in a terminal profile file.
//# Categories: ansi, color, customization, interactive, styling, terminal, theming, tools, windows, xterm
use std::env;
use std::process;
use thag_styling::{
    auto_help, get_verbosity, help_system::check_help_and_exit, is_konsole, is_mintty,
    set_verbosity_from_env, vprtln, ColorInitStrategy, PaletteSync, TermAttributes, Theme, V,
};

fn main() {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    set_verbosity_from_env();

    if is_konsole() {
        eprintln!(
            r"KDE Konsole terminal type detected. Konsole does not support the OSC 4 ANSI escape sequence that this tool uses.
            Instead you can use `thag_gen_terminal_themes` to generate a Konsole theme."
        );
    }

    if is_mintty() {
        eprintln!(
            r"Mintty terminal type detected. Mintty does not support the OSC 4 ANSI escape sequence that this tool uses.
            Instead you can use `thag_gen_terminal_themes` to generate a Mintty theme."
        );
    }

    let args: Vec<String> = env::args().collect();

    // Initialize thag_styling system
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Default);

    match args.get(1).map(String::as_str) {
        Some("apply") => {
            let theme_name = args.get(2);
            if let Some(name) = theme_name {
                apply_theme(name);
            } else {
                eprintln!("❌ Theme name required");
                print_usage();
                process::exit(1);
            }
        }
        Some("preview") => {
            let theme_name = args.get(2);
            if let Some(name) = theme_name {
                preview_theme(name);
            } else {
                eprintln!("❌ Theme name required");
                print_usage();
                process::exit(1);
            }
        }
        Some("reset") => {
            reset_palette();
        }
        Some("demo") => {
            demo_palette();
        }
        Some("list") => {
            list_themes();
        }
        // Some("help") | Some("-h") | Some("--help") => {
        //     print_usage();
        // }
        _ => {
            print_usage();
        }
    }
}

fn apply_theme(theme_name: &str) {
    vprtln!(V::N, "🎨 Loading theme: {}", theme_name);

    let mut theme = match Theme::get_builtin(theme_name) {
        Ok(theme) => theme,
        Err(e) => {
            eprintln!("❌ Failed to load theme '{}': {}", theme_name, e);
            vprtln!(
                V::N,
                "💡 Try running 'thag_sync_palette list' to see available themes"
            );
            process::exit(1);
        }
    };

    // Load base_colors for accurate ANSI terminal mapping
    if let Err(e) = theme.load_base_colors() {
        vprtln!(V::V, "⚠️ Could not load base colors: {}", e);
        vprtln!(V::V, "Falling back to role-based ANSI mapping");
    }

    vprtln!(V::N, "📝 Description: {}", theme.description);
    vprtln!(V::N, "🌈 Applying palette...");

    if let Err(e) = PaletteSync::apply_theme(&theme) {
        eprintln!("❌ Failed to apply theme: {}", e);
        process::exit(1);
    }

    if get_verbosity() >= V::Normal {
        println!("✅ Theme applied successfully!");
        println!("🔄 To reset colors, run: thag_sync_palette reset");

        // Show a quick demonstration
        println!("\n🎭 Color demonstration:");
        PaletteSync::show_background_info(&theme);
        PaletteSync::demonstrate_palette();
    }
}

fn preview_theme(theme_name: &str) {
    vprtln!(V::N, "🎨 Previewing theme: {}", theme_name);

    let mut theme = match Theme::get_builtin(theme_name) {
        Ok(theme) => theme,
        Err(e) => {
            eprintln!("❌ Failed to load theme '{}': {}", theme_name, e);
            vprtln!(
                V::N,
                "💡 Try running 'thag_sync_palette list' to see available themes"
            );
            process::exit(1);
        }
    };

    // Load base_colors for accurate ANSI terminal mapping
    if let Err(e) = theme.load_base_colors() {
        vprtln!(V::V, "⚠️ Could not load base colors: {}", e);
        vprtln!(V::V, "Falling back to role-based ANSI mapping");
    }

    if let Err(e) = PaletteSync::preview_theme(&theme) {
        eprintln!("❌ Failed to preview theme: {}", e);
        process::exit(1);
    }

    if get_verbosity() >= V::Normal {
        // Wait for user input before continuing
        println!("\n⏸️  Preview applied! Press Enter to see color demo, or Ctrl+C to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);

        PaletteSync::show_background_info(&theme);

        PaletteSync::demonstrate_palette();

        println!(
            "\n🔄 To make this permanent, run: thag_sync_palette apply {}",
            theme_name
        );
        println!("🔄 To reset colors, run: thag_sync_palette reset");
    }
}

fn reset_palette() {
    vprtln!(V::N, "🔄 Resetting terminal palette to defaults...");

    if let Err(e) = PaletteSync::reset_palette() {
        eprintln!("❌ Failed to reset palette: {}", e);
        process::exit(1);
    }

    vprtln!(V::N, "✅ Terminal palette reset successfully!");
}

fn demo_palette() {
    PaletteSync::demonstrate_palette();
}

fn list_themes() {
    println!("🎨 Available built-in themes:");
    println!();

    let themes = Theme::list_builtin();
    for theme_name in themes {
        if let Ok(theme) = Theme::get_builtin(&theme_name) {
            println!("  📦 {} - {}", theme_name, theme.description);
        } else {
            println!("  📦 {}", theme_name);
        }
    }

    println!();
    println!("💡 Use any theme name with the 'apply' or 'preview' command");
}

fn print_usage() {
    println!("🎨 Thag Terminal Palette Synchronization");
    println!();
    println!("Sync your terminal's color palette with thag_styling themes using OSC sequences");
    println!();
    println!("Usage:");
    println!("  thag_sync_palette apply <theme>    Apply theme to terminal palette");
    println!("  thag_sync_palette preview <theme>  Preview theme temporarily");
    println!("  thag_sync_palette reset            Reset palette to defaults");
    println!("  thag_sync_palette demo             Show current palette");
    println!("  thag_sync_palette list             List available themes");
    println!("  thag_sync_palette help             Show this help message");
    println!();
    println!("Examples:");
    println!("  thag_sync_palette apply thag-botticelli-birth-of-venus-dark");
    println!("  thag_sync_palette preview thag-dark");
    println!("  thag_sync_palette reset");
    println!();
    println!("💡 This tool uses OSC sequences to update your terminal's color palette");
    println!("   in real-time. Most modern terminals support this feature.");
    println!();
    println!("🌈 Supported terminals: WezTerm, Alacritty, iTerm2, Kitty, Windows Terminal,");
    println!("   Gnome Terminal, and most other modern terminal emulators.");
}
