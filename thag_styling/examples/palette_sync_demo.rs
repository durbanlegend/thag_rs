//! Palette synchronization demo
//!
//! This example demonstrates how to use OSC sequences to update your terminal's
//! color palette in real-time to match a thag_styling theme.
//!
//! Usage:
//! ```bash
//! cargo run --example palette_sync_demo --features="color_detect" -- apply thag-botticelli-birth-of-venus
//! cargo run --example palette_sync_demo --features="color_detect" -- preview thag-botticelli-birth-of-venus
//! cargo run --example palette_sync_demo --features="color_detect" -- reset
//! cargo run --example palette_sync_demo --features="color_detect" -- demo
//! ```

use std::env;
use std::process;
use thag_styling::{ColorInitStrategy, PaletteSync, TermAttributes, Theme};

fn main() {
    // Initialize thag_styling system
    TermAttributes::initialize(&ColorInitStrategy::Default);

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
        args().nth(0).unwrap_or_default()
    );

    // Show a quick demonstration
    println!("\n🎭 Color demonstration:");
    if let Err(e) = PaletteSync::show_background_info(&theme) {
        eprintln!("Warning: Failed to show background info: {}", e);
    }
    if let Err(e) = PaletteSync::demonstrate_palette() {
        eprintln!("Warning: Failed to demonstrate palette: {}", e);
    }
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

    if let Err(e) = PaletteSync::show_background_info(&theme) {
        eprintln!("Warning: Failed to show background info: {}", e);
    }
    if let Err(e) = PaletteSync::demonstrate_palette() {
        eprintln!("Warning: Failed to demonstrate palette: {}", e);
    }

    println!(
        "\n🔄 To make this permanent, run: {} apply {}",
        args().nth(0).unwrap_or_default(),
        theme_name
    );
    println!(
        "🔄 To reset colors, run: {} reset",
        args().nth(0).unwrap_or_default()
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
    use thag_styling::{Role, Style, StyleLike};

    println!("🌈 Thag styling roles with current theme:");

    let roles_and_messages = [
        (Role::Heading1, "# Heading 1 - Major sections"),
        (Role::Heading2, "## Heading 2 - Subsections"),
        (Role::Heading3, "### Heading 3 - Minor sections"),
        (Role::Error, "❌ Error - Something went wrong"),
        (Role::Warning, "⚠️  Warning - Pay attention"),
        (Role::Success, "✅ Success - Everything worked!"),
        (Role::Info, "ℹ️  Info - Informational message"),
        (Role::Emphasis, "⭐ Emphasis - Important content"),
        (Role::Code, "💻 Code - `filenames and code blocks`"),
        (Role::Normal, "📄 Normal - Regular text content"),
        (Role::Subtle, "🔍 Subtle - Secondary information"),
        (Role::Hint, "💡 Hint - Helpful suggestions"),
        (Role::Debug, "🐛 Debug - Development info"),
        (Role::Trace, "🔍 Trace - Detailed diagnostic"),
    ];

    for (role, message) in roles_and_messages {
        let style = Style::from(role);
        style.prtln(format_args!("{}", message));
    }

    println!();
    println!("🎯 The colors above use the updated terminal palette + thag attributes!");
    println!("📝 Note: Colors only apply to this terminal session (not new tabs/windows)");
}

fn print_usage() {
    let program = args()
        .nth(0)
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
