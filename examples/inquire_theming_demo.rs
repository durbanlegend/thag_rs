/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["inquire_theming"] }

[features]
default = ["inquire_theming"]
inquire_theming = ["thag_profiler/inquire_theming"]
*/

//! Demonstration of theme-aware inquire integration in thag_profiler
//!
//! This example shows how the new inquire theming system works in thag_profiler
//! without requiring the full thag_rs styling system.
//!
//! To run this example:
//! ```bash
//! cd thag_rs
//! cargo run --example inquire_theming_demo --features "tools"
//! ```

use inquire::{Confirm, MultiSelect, Select, Text};
use std::error::Error;

// Import the inquire theming from thag_profiler
use thag_profiler::ui::inquire_theming;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🎨 Inquire Theming Demo");
    println!("{}", "=".repeat(50));

    // Show terminal capabilities detection
    #[cfg(feature = "inquire_theming")]
    {
        let (color_support, background) = inquire_theming::get_terminal_info();
        println!("Detected terminal capabilities:");
        println!("  Color support: {:?}", color_support);
        println!("  Background: {:?}", background);
        println!();
    }

    #[cfg(not(feature = "inquire_theming"))]
    {
        println!("Theming is disabled - using default inquire styling");
        println!();
    }

    // Apply theme-aware styling globally
    inquire_theming::apply_global_theming();

    println!("🚀 Testing different inquire prompt types with theming...");
    println!();

    // Test 1: Select prompt
    let theme_options = vec![
        "Dark theme optimized",
        "Light theme optimized",
        "High contrast mode",
        "Colorblind friendly",
        "Minimal styling",
    ];

    let selected_theme = Select::new("Choose a theme preference:", theme_options)
        .with_help_message("This demonstrates theme-aware select styling")
        .prompt()?;

    println!("✅ You selected: {}", selected_theme);
    println!();

    // Test 2: MultiSelect prompt
    let features = vec![
        "Color detection",
        "Terminal background detection",
        "Theme switching",
        "Color distance optimization",
        "Fallback support",
    ];

    let selected_features = MultiSelect::new("Which features interest you?", features)
        .with_help_message("Multiple selections allowed - notice the themed colors")
        .prompt()?;

    println!("✅ You're interested in: {:?}", selected_features);
    println!();

    // Test 3: Text input
    let user_input = Text::new("Enter a comment about the theming:")
        .with_placeholder("The colors look great!")
        .with_help_message("This text input also uses themed colors")
        .prompt()?;

    println!("✅ Your comment: {}", user_input);
    println!();

    // Test 4: Confirmation
    let continue_demo = Confirm::new("Would you like to see error handling?")
        .with_default(true)
        .with_help_message("This tests themed error messages")
        .prompt()?;

    if continue_demo {
        // Test 5: Error handling with themed colors
        let _invalid_input = Text::new("Enter 'error' to see themed error styling:")
            .with_validator(|input: &str| {
                if input == "error" {
                    Ok(inquire::validator::Validation::Invalid(
                        "This is a demonstration error message with themed colors!".into()
                    ))
                } else {
                    Ok(inquire::validator::Validation::Valid)
                }
            })
            .prompt()?;
    }

    println!();
    println!("🎉 Demo complete!");
    println!();
    println!("Key benefits of the new approach:");
    println!("  ✨ Lightweight - no dependency on full thag_rs styling");
    println!("  🎯 Automatic - detects terminal capabilities");
    println!("  🔄 Fallback - graceful degradation on limited terminals");
    println!("  🚀 Fast - minimal overhead");
    println!("  🔧 Extensible - easy to customize colors");
    println!();

    #[cfg(feature = "inquire_theming")]
    println!("🔧 Theming is ENABLED - you saw theme-aware colors");

    #[cfg(not(feature = "inquire_theming"))]
    println!("🔧 Theming is DISABLED - you saw default inquire colors");

    Ok(())
}
