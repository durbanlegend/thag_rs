//! Demonstration of hybrid inquire theming with multiple strategies
//!
//! This example shows how the new hybrid inquire theming system works in thag_profiler,
//! offering multiple theming approaches from full thag_rs integration to lightweight options.
//!
//! To run this example:
//! ```bash
//! cd thag_rs
//! cargo run -p thag_profiler --example inquire_theming_demo --features inquire_theming
//! ```

use inquire::{Confirm, MultiSelect, Select, Text};
use std::error::Error;

// Import the hybrid inquire theming from thag_profiler
use thag_profiler::ui::inquire_theming::{self, ThemingStrategy};

fn main() -> Result<(), Box<dyn Error>> {
    println!("🎨 Hybrid Inquire Theming Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Show what strategies are available
    let available_strategies = inquire_theming::get_available_strategies();
    println!("📋 Available theming strategies:");
    for strategy in &available_strategies {
        println!("  • {:?}: {}", strategy, inquire_theming::describe_strategy(*strategy));
    }
    println!();

    // Show terminal capabilities detection
    #[cfg(feature = "inquire_theming")]
    {
        let (color_capability, background_type) = inquire_theming::get_terminal_info();
        println!("🔍 Detected terminal capabilities:");
        println!("  Color capability: {:?}", color_capability);
        println!("  Background type: {:?}", background_type);
        println!();
    }

    #[cfg(not(feature = "inquire_theming"))]
    {
        println!("⚠️  Theming is disabled - using default inquire styling");
        println!();
    }

    // Let user choose which theming strategy to demonstrate
    let strategy_options = vec![
        "Auto (recommended)",
        "Lightweight theming",
        "Default inquire colors",
        "Compare all strategies",
    ];

    let selected_demo = Select::new("Which theming approach would you like to demonstrate?", strategy_options)
        .with_help_message("This selection uses Auto strategy")
        .prompt()?;

    println!();

    match selected_demo {
        "Auto (recommended)" => demonstrate_strategy(ThemingStrategy::Auto, "Auto Strategy")?,
        "Lightweight theming" => demonstrate_strategy(ThemingStrategy::Lightweight, "Lightweight Strategy")?,
        "Default inquire colors" => demonstrate_strategy(ThemingStrategy::Default, "Default Strategy")?,
        "Compare all strategies" => compare_all_strategies()?,
        _ => unreachable!(),
    }

    println!();
    println!("🎉 Demo complete!");
    println!();
    println!("💡 Key improvements in this hybrid approach:");
    println!("  ✨ Multiple strategies - choose what works best for your use case");
    println!("  🎯 Better contrast - magenta/blue for subtle text instead of gray");
    println!("  🌓 Light/dark support - appropriate colors for both backgrounds");
    println!("  🔄 Graceful fallback - always works, even with limited terminal support");
    println!("  🚀 Flexible integration - from full thag_rs themes to lightweight options");
    println!();

    Ok(())
}

fn demonstrate_strategy(strategy: ThemingStrategy, name: &str) -> Result<(), Box<dyn Error>> {
    println!("🎨 Demonstrating: {}", name);
    println!("{}", "-".repeat(40));

    // Apply the chosen strategy
    inquire_theming::apply_global_theming_with_strategy(strategy);

    // Test 1: Select prompt
    let color_options = vec![
        "Bright green (selected items)",
        "Light gray/white (normal text)",
        "Blue/cyan (help messages)",
        "Red (error messages)",
        "Green (success/answers)",
        "Magenta (subtle/placeholder - improved contrast!)",
    ];

    let _selected_color = Select::new("Notice the color scheme - which color looks best?", color_options)
        .with_help_message("Help text uses blue/cyan for better visibility")
        .prompt()?;

    // Test 2: MultiSelect with various elements
    let features = vec![
        "Improved contrast for subtle text",
        "Light/dark background detection",
        "TrueColor/256-color/Basic fallbacks",
        "Magenta instead of gray for better visibility",
        "Multiple theming strategies",
    ];

    let _selected_features = MultiSelect::new("What improvements do you notice?", features)
        .with_help_message("Multiple selections allowed - notice the themed colors")
        .prompt()?;

    // Test 3: Text input with placeholder
    let _user_input = Text::new("How does the color contrast look?")
        .with_placeholder("The magenta placeholder text should be more visible now!")
        .with_help_message("Placeholder text now uses magenta for better contrast")
        .prompt()?;

    // Test 4: Error demonstration
    let continue_demo = Confirm::new("Would you like to see the error styling?")
        .with_default(true)
        .with_help_message("This tests themed error message colors")
        .prompt()?;

    if continue_demo {
        let _error_demo = Text::new("Type 'error' to see themed error styling:")
            .with_validator(|input: &str| {
                if input == "error" {
                    Ok(inquire::validator::Validation::Invalid(
                        "This error message should appear in themed red color!".into()
                    ))
                } else {
                    Ok(inquire::validator::Validation::Valid)
                }
            })
            .prompt()?;
    }

    println!("✅ {} demonstration complete!\n", name);
    Ok(())
}

fn compare_all_strategies() -> Result<(), Box<dyn Error>> {
    println!("🔄 Comparing all theming strategies");
    println!("{}", "-".repeat(40));

    let strategies = vec![
        (ThemingStrategy::Auto, "Auto"),
        (ThemingStrategy::Lightweight, "Lightweight"),
        (ThemingStrategy::Default, "Default"),
    ];

    for (strategy, name) in strategies {
        println!("\n🎨 Testing {} strategy:", name);
        inquire_theming::apply_global_theming_with_strategy(strategy);

        let test_options = vec!["Good contrast", "Poor contrast", "Can't tell the difference"];
        let _rating = Select::new(&format!("How's the contrast with {} theming?", name), test_options)
            .with_help_message(&format!("{}: {}", name, inquire_theming::describe_strategy(strategy)))
            .prompt()?;
    }

    // Reset to auto for final question
    inquire_theming::apply_global_theming_with_strategy(ThemingStrategy::Auto);

    let _final_choice = Select::new("Which strategy did you prefer?", vec![
        "Auto (smart fallback)",
        "Lightweight (consistent basic theming)",
        "Default (no theming)",
        "They all looked the same to me",
    ])
    .with_help_message("This final question uses Auto strategy")
    .prompt()?;

    Ok(())
}
