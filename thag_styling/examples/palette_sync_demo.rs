//! Palette synchronization demo
//!
//! This example demonstrates how to use OSC sequences to update your terminal's
//! color palette in real-time to match a `thag_styling` theme.
//!
//! Usage:
//! ```bash
//! C
//! cargo run -p thag_styling --example palette_sync_demo --features="color_detect" -- preview thag-botticelli-birth-of-venus
//! cargo run -p thag_styling --example palette_sync_demo --features="color_detect" -- reset
//! cargo run -p thag_styling --example palette_sync_demo --features="color_detect" -- demo
//! ```

use std::env;
use std::process;
use thag_styling::{ColorInitStrategy, PaletteSync, TermAttributes, Theme};

fn main() {
    // Initialize thag_styling system
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Default);

    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("apply") => {
            let default_theme = "thag-botticelli-birth-of-venus".to_string();
            let theme_name = args.get(2).unwrap_or(&default_theme);
            apply_theme(theme_name);
        }
        Some("preview") => {
            let default_theme = "thag-botticelli-birth-of-venus".to_string();
            let theme_name = args.get(2).unwrap_or(&default_theme);
            preview_theme(theme_name);
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
        _ => {
            print_usage();
        }
    }
}

fn apply_theme(theme_name: &str) {
    println!("🎨 Loading theme: {}", theme_name);

    let theme = match Theme::get_builtin(theme_name) {
        Ok(theme) => theme,
        Err(e) => {
            eprintln!("❌ Failed to load theme '{}': {}", theme_name, e);
            println!("💡 Try running with 'list' to see available themes");
            process::exit(1);
        }
    };

    println!("📝 Description: {}", theme.description);
    println!("🌈 Applying palette...");

    if let Err(e) = PaletteSync::apply_theme(&theme) {
        eprintln!("❌ Failed to apply theme: {}", e);
        process::exit(1);
    }

    println!("✅ Theme applied successfully!");
    println!(
        "🔄 To reset colors, run: {} reset",
        args().next().unwrap_or_default()
    );

    // Show a quick demonstration
    println!("\n🎭 Color demonstration:");
    PaletteSync::show_background_info(&theme);

    PaletteSync::demonstrate_palette();
}

fn preview_theme(theme_name: &str) {
    println!("🎨 Previewing theme: {}", theme_name);

    let theme = match Theme::get_builtin(theme_name) {
        Ok(theme) => theme,
        Err(e) => {
            eprintln!("❌ Failed to load theme '{}': {}", theme_name, e);
            println!("💡 Try running with 'list' to see available themes");
            process::exit(1);
        }
    };

    if let Err(e) = PaletteSync::preview_theme(&theme) {
        eprintln!("❌ Failed to preview theme: {}", e);
        process::exit(1);
    }

    // Wait for user input before continuing
    println!("\n⏸️  Preview applied! Press Enter to see color demo, or Ctrl+C to exit...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);

    PaletteSync::show_background_info(&theme);

    PaletteSync::demonstrate_palette();

    println!(
        "\n🔄 To make this permanent, run: {} apply {}",
        args().next().unwrap_or_default(),
        theme_name
    );
    println!(
        "🔄 To reset colors, run: {} reset",
        args().next().unwrap_or_default()
    );
}

fn reset_palette() {
    println!("🔄 Resetting terminal palette to defaults...");

    if let Err(e) = PaletteSync::reset_palette() {
        eprintln!("❌ Failed to reset palette: {}", e);
        process::exit(1);
    }

    println!("✅ Terminal palette reset successfully!");
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
    let program = args()
        .next()
        .unwrap_or_else(|| "palette_sync_demo".to_string());

    println!("🎨 Thag Styling Palette Sync Demo");
    println!();
    println!("Usage:");
    println!(
        "  {} apply <theme>    Apply theme to terminal palette",
        program
    );
    println!("  {} preview <theme>  Preview theme temporarily", program);
    println!("  {} reset           Reset palette to defaults", program);
    println!("  {} demo            Show current palette", program);
    println!("  {} list            List available themes", program);
    println!();
    println!("Examples:");
    println!("  {} apply thag-botticelli-birth-of-venus", program);
    println!("  {} preview thag-dark", program);
    println!("  {} reset", program);
    println!();
    println!("💡 This demo uses OSC sequences to update your terminal's color palette");
    println!("   in real-time. Most modern terminals support this feature.");
}

fn args() -> env::Args {
    env::args()
}
