/// Terminal palette synchronization using OSC sequences
///
/// This binary provides command-line access to `thag_styling`'s palette synchronization
/// functionality, allowing you to apply theme colors directly to your terminal's palette.
//# Purpose: Configure custom terminal themes.
//# Categories: ansi, color, customization, interactive, styling, terminal, theming, tools, windows, xterm
use std::env;
use std::process;
use thag_common::get_verbosity;
use thag_styling::{ColorInitStrategy, PaletteSync, TermAttributes, Theme, V};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Initialize thag_styling system
    TermAttributes::initialize(&ColorInitStrategy::Default);

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
    if get_verbosity() >= V::Normal {
        println!("🎨 Loading theme: {}", theme_name);
    }

    let theme = match Theme::get_builtin(theme_name) {
        Ok(theme) => theme,
        Err(e) => {
            eprintln!("❌ Failed to load theme '{}': {}", theme_name, e);
            if get_verbosity() >= V::Normal {
                println!("💡 Try running 'thag_sync_palette list' to see available themes");
            }
            process::exit(1);
        }
    };

    if get_verbosity() >= V::Normal {
        println!("📝 Description: {}", theme.description);
        println!("🌈 Applying palette...");
    }

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
    if get_verbosity() >= V::Normal {
        println!("🎨 Previewing theme: {}", theme_name);
    }

    let theme = match Theme::get_builtin(theme_name) {
        Ok(theme) => theme,
        Err(e) => {
            eprintln!("❌ Failed to load theme '{}': {}", theme_name, e);
            if get_verbosity() >= V::Normal {
                println!("💡 Try running 'thag_sync_palette list' to see available themes");
            }
            process::exit(1);
        }
    };

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
    if get_verbosity() >= V::Normal {
        println!("🔄 Resetting terminal palette to defaults...");
    }

    if let Err(e) = PaletteSync::reset_palette() {
        eprintln!("❌ Failed to reset palette: {}", e);
        process::exit(1);
    }

    if get_verbosity() >= V::Normal {
        println!("✅ Terminal palette reset successfully!");
    }
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
    println!("  thag_sync_palette apply thag-botticelli-birth-of-venus");
    println!("  thag_sync_palette preview thag-dark");
    println!("  thag_sync_palette reset");
    println!();
    println!("💡 This tool uses OSC sequences to update your terminal's color palette");
    println!("   in real-time. Most modern terminals support this feature.");
    println!();
    println!("🌈 Supported terminals: WezTerm, Alacritty, iTerm2, Kitty, Windows Terminal,");
    println!("   Gnome Terminal, and most other modern terminal emulators.");
}
