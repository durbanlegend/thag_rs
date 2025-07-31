//! Example tool demonstrating hybrid inquire theming integration
//!
//! This shows how a real tool would integrate the hybrid theming system,
//! with options for different theming strategies based on use case.
//!
//! To run this example:
//! ```bash
//! cd thag_rs
//! cargo run -p thag_profiler --example tool_with_theming --features inquire_theming
//! ```

use inquire::{Confirm, MultiSelect, Select, Text};
use std::error::Error;
use thag_profiler::ui::inquire_theming::{self, ThemingStrategy};

#[derive(Debug)]
struct ToolConfig {
    theme_strategy: ThemingStrategy,
    auto_detect: bool,
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔧 Example Tool with Hybrid Theming");
    println!("{}", "=".repeat(50));
    println!();

    // Initialize with smart defaults
    let mut config = ToolConfig {
        theme_strategy: ThemingStrategy::Auto,
        auto_detect: true,
        verbose: false,
    };

    // Show available theming options
    show_theming_info();

    // Let user configure theming preferences
    configure_theming(&mut config)?;

    // Apply the chosen theming strategy
    inquire_theming::apply_global_theming_with_strategy(config.theme_strategy);

    if config.verbose {
        println!("✅ Applied theming strategy: {:?}", config.theme_strategy);
        println!("📄 Description: {}", inquire_theming::describe_strategy(config.theme_strategy));
        println!();
    }

    // Simulate a real tool workflow
    run_tool_workflow(&config)?;

    Ok(())
}

fn show_theming_info() {
    println!("🎨 Available Theming Options:");
    println!();

    let strategies = inquire_theming::get_available_strategies();
    for strategy in strategies {
        println!("  • {:?}", strategy);
        println!("    {}", inquire_theming::describe_strategy(strategy));
        println!();
    }

    // Show detected capabilities
    #[cfg(feature = "inquire_theming")]
    {
        let (capability, background) = inquire_theming::get_terminal_info();
        println!("🖥️  Your Terminal:");
        println!("  Color capability: {:?}", capability);
        println!("  Background type: {:?}", background);
        println!();
    }
}

fn configure_theming(config: &mut ToolConfig) -> Result<(), Box<dyn Error>> {
    println!("⚙️  Theming Configuration");
    println!("{}", "-".repeat(30));

    // Use default theming for this configuration step
    inquire_theming::apply_global_theming_with_strategy(ThemingStrategy::Default);

    let strategy_choice = Select::new(
        "Choose your preferred theming strategy:",
        vec![
            "Auto (recommended) - Smart fallback based on environment",
            "Lightweight - Basic terminal-aware colors",
            "Default - Standard inquire colors",
            "Let me see examples first",
        ]
    )
    .with_help_message("This affects how prompts will be colored throughout the tool")
    .prompt()?;

    config.theme_strategy = match strategy_choice {
        "Auto (recommended) - Smart fallback based on environment" => ThemingStrategy::Auto,
        "Lightweight - Basic terminal-aware colors" => ThemingStrategy::Lightweight,
        "Default - Standard inquire colors" => ThemingStrategy::Default,
        "Let me see examples first" => {
            show_strategy_examples()?;
            // After examples, default to Auto
            ThemingStrategy::Auto
        }
        _ => ThemingStrategy::Auto,
    };

    config.verbose = Confirm::new("Enable verbose output?")
        .with_default(false)
        .with_help_message("Shows theming info and debug output")
        .prompt()?;

    println!();
    Ok(())
}

fn show_strategy_examples() -> Result<(), Box<dyn Error>> {
    println!();
    println!("🔍 Strategy Examples");
    println!("{}", "-".repeat(25));

    let strategies = vec![
        (ThemingStrategy::Auto, "Auto"),
        (ThemingStrategy::Lightweight, "Lightweight"),
        (ThemingStrategy::Default, "Default"),
    ];

    for (strategy, name) in strategies {
        println!("\n📝 Testing {} strategy:", name);
        inquire_theming::apply_global_theming_with_strategy(strategy);

        let _example = Select::new(
            &format!("How does {} theming look?", name),
            vec!["Excellent", "Good", "Poor", "Can't see difference"]
        )
        .with_help_message(&format!("Strategy: {} - {}", name, inquire_theming::describe_strategy(strategy)))
        .prompt()?;
    }

    println!();
    Ok(())
}

fn run_tool_workflow(config: &ToolConfig) -> Result<(), Box<dyn Error>> {
    println!("🚀 Running Tool Workflow");
    println!("{}", "-".repeat(30));

    // Step 1: Input gathering
    let project_types = vec![
        "Web Application",
        "CLI Tool",
        "Library/Crate",
        "Desktop Application",
        "Embedded System",
    ];

    let project_type = Select::new("What type of project are you working on?", project_types)
        .with_help_message("This helps determine appropriate tool settings")
        .prompt()?;

    if config.verbose {
        println!("  Selected project type: {}", project_type);
    }

    // Step 2: Feature selection
    let features = vec![
        "Error handling improvements",
        "Performance optimization",
        "Documentation generation",
        "Testing framework setup",
        "CI/CD pipeline configuration",
        "Code formatting and linting",
    ];

    let selected_features = MultiSelect::new("Which features would you like to enable?", features)
        .with_help_message("You can select multiple features - they'll be configured automatically")
        .prompt()?;

    if config.verbose {
        println!("  Selected features: {:?}", selected_features);
    }

    // Step 3: Configuration input
    let project_name = Text::new("Enter your project name:")
        .with_placeholder("my-awesome-project")
        .with_help_message("This will be used for generated files and configurations")
        .prompt()?;

    if config.verbose {
        println!("  Project name: {}", project_name);
    }

    // Step 4: Validation with error demo
    let _version = Text::new("Enter project version (or 'error' to see error styling):")
        .with_default("0.1.0")
        .with_validator(|input: &str| {
            if input == "error" {
                Ok(inquire::validator::Validation::Invalid(
                    "This demonstrates error message theming - notice the color!".into()
                ))
            } else if input.is_empty() {
                Ok(inquire::validator::Validation::Invalid(
                    "Version cannot be empty".into()
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .with_help_message("Use semantic versioning (e.g., 1.0.0)")
        .prompt()?;

    // Step 5: Final confirmation
    let proceed = Confirm::new("Generate configuration files with these settings?")
        .with_default(true)
        .with_help_message("This will create the project structure and configuration")
        .prompt()?;

    if proceed {
        println!();
        println!("✅ Configuration completed successfully!");
        println!("📁 Project: {}", project_name);
        println!("🏷️  Type: {}", project_type);
        println!("🎛️  Features: {} selected", selected_features.len());
        println!("🎨 Theming: {:?}", config.theme_strategy);

        if config.verbose {
            println!();
            println!("🔧 Theming Details:");
            println!("  Strategy: {}", inquire_theming::describe_strategy(config.theme_strategy));
            #[cfg(feature = "inquire_theming")]
            {
                let (capability, background) = inquire_theming::get_terminal_info();
                println!("  Terminal capability: {:?}", capability);
                println!("  Background type: {:?}", background);
            }
        }
    } else {
        println!("❌ Configuration cancelled.");
    }

    println!();
    println!("💡 Integration Tips:");
    println!("  • Call apply_global_theming() once at startup");
    println!("  • Use ThemingStrategy::Auto for best user experience");
    println!("  • Provide strategy selection for power users");
    println!("  • Test with different terminal types and backgrounds");
    println!("  • The improved contrast (magenta/blue) works better than gray");

    Ok(())
}
