//! Terminal palette synchronization using OSC sequences
//!
//! This binary provides command-line access to thag_styling's palette synchronization
//! functionality, allowing you to apply theme colors directly to your terminal's palette.

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
        Some("help") | Some("-h") | Some("--help") => {
            print_usage();
        }
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
        if let Err(e) = PaletteSync::show_background_info(&theme) {
            eprintln!("Warning: Failed to show background info: {}", e);
        }
        if let Err(e) = PaletteSync::demonstrate_hybrid_styling(&theme) {
            eprintln!("Warning: Failed to demonstrate hybrid styling: {}", e);
        }
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

        if let Err(e) = PaletteSync::show_background_info(&theme) {
            eprintln!("Warning: Failed to show background info: {}", e);
        }
        if let Err(e) = PaletteSync::demonstrate_hybrid_styling(&theme) {
            eprintln!("Warning: Failed to demonstrate hybrid styling: {}", e);
        }

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
    if let Err(e) = PaletteSync::demonstrate_palette() {
        eprintln!("❌ Failed to demonstrate palette: {}", e);
        process::exit(1);
    }
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

fn demonstrate_colors() {
    println!("🌈 Colors using updated ANSI palette:");

    // Use direct ANSI codes that correspond to our palette mapping
    println!("\x1b[95m# Heading 1 - Major sections\x1b[0m (ANSI 13: Bright Magenta)");
    println!("\x1b[94m## Heading 2 - Subsections\x1b[0m (ANSI 12: Bright Blue)");
    println!("\x1b[93m### Heading 3 - Minor sections\x1b[0m (ANSI 11: Bright Yellow)");
    println!("\x1b[31m❌ Error - Something went wrong\x1b[0m (ANSI 1: Red)");
    println!("\x1b[33m⚠️  Warning - Pay attention\x1b[0m (ANSI 3: Yellow)");
    println!("\x1b[32m✅ Success - Everything worked!\x1b[0m (ANSI 2: Green)");
    println!("\x1b[36mℹ️  Info - Informational message\x1b[0m (ANSI 6: Cyan)");
    println!("\x1b[35m⭐ Emphasis - Important content\x1b[0m (ANSI 5: Magenta)");
    println!("\x1b[34m💻 Code - `filenames and code blocks`\x1b[0m (ANSI 4: Blue)");
    println!("\x1b[37m📄 Normal - Regular text content\x1b[0m (ANSI 7: White)");
    println!("\x1b[90m🔍 Subtle - Secondary information\x1b[0m (ANSI 8: Bright Black)");
    println!("\x1b[96m💡 Hint - Helpful suggestions\x1b[0m (ANSI 14: Bright Cyan)");
    println!("\x1b[92m🐛 Debug - Development info\x1b[0m (ANSI 10: Bright Green)");
    println!("\x1b[91m🔍 Trace - Detailed diagnostic\x1b[0m (ANSI 9: Bright Red)");

    println!();
    println!("🎯 These colors should now match the Botticelli theme!");
    println!("🔧 Try opening a new terminal tab to see the updated colors in action");
    println!("📝 Note: Attributes like bold, italic, dim require thag styling system");
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
