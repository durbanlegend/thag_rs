/*[toml]
[dependencies]
thag_proc_macros = { version = "0.1, thag-auto" }
*/

use clap::CommandFactory;
use colored::Colorize;
use inquire::MultiSelect;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use thag_proc_macros::file_navigator;

// Import the Cli struct from the main crate
use thag_rs::cmd_args::Cli;

file_navigator! {}

#[derive(Debug, Clone)]
struct OptionInfo {
    name: String,
    short: Option<char>,
    long: String,
    help: String,
    takes_value: bool,
    group: Option<String>,
}

#[derive(Debug, Clone)]
struct OptionGroup {
    name: String,
    options: Vec<OptionInfo>,
    multiple: bool,
}

fn get_clap_groups() -> HashMap<String, Vec<String>> {
    let cmd = Cli::command();
    let mut group_members: HashMap<String, Vec<String>> = HashMap::new();

    // Extract clap argument groups
    for group in cmd.get_groups() {
        let group_name = group.get_id().to_string();
        let mut members = Vec::new();

        for arg_id in group.get_args() {
            members.push(arg_id.to_string());
        }

        group_members.insert(group_name, members);
    }

    group_members
}

fn extract_clap_metadata() -> Vec<OptionGroup> {
    let cmd = Cli::command();
    let clap_groups = get_clap_groups();

    // Pre-define logical groups based on the help headings and argument groups
    let mut output_options = Vec::new();
    let mut processing_options = Vec::new();
    let mut dynamic_options = Vec::new();
    let mut filter_options = Vec::new();
    let mut norun_options = Vec::new();
    let mut verbosity_options = Vec::new();

    // Extract all arguments
    for arg in cmd.get_arguments() {
        let option_info = OptionInfo {
            name: arg.get_id().to_string(),
            short: arg.get_short(),
            long: arg.get_long().unwrap_or("").to_string(),
            help: arg
                .get_help()
                .map_or_else(|| "".to_string(), |h| h.to_string()),
            takes_value: arg.get_action().takes_values(),
            group: arg.get_help_heading().map(|h| h.to_string()),
        };

        // First categorize by help heading, then override with clap groups for mutual exclusivity
        let mut categorized = false;

        // Categorize by help heading first
        match option_info.group.as_deref() {
            Some("Output Options") => {
                output_options.push(option_info.clone());
                categorized = true;
            }
            Some("Processing Options") => {
                processing_options.push(option_info.clone());
                categorized = true;
            }
            Some("Dynamic Options (no script)") => {
                dynamic_options.push(option_info.clone());
                categorized = true;
            }
            Some("Filter Options") => {
                filter_options.push(option_info.clone());
                categorized = true;
            }
            Some("No-run Options") => {
                norun_options.push(option_info.clone());
                categorized = true;
            }
            _ => {}
        }

        // Check if this option belongs to a clap argument group (for mutual exclusivity)
        let mut in_clap_group = false;
        for (group_name, members) in &clap_groups {
            if members.contains(&option_info.name) {
                in_clap_group = true;
                match group_name.as_str() {
                    "commands" => {
                        // Move to dynamic options if not already categorized properly
                        if !categorized
                            || !matches!(
                                option_info.group.as_deref(),
                                Some("Dynamic Options (no script)")
                            )
                        {
                            dynamic_options.push(option_info.clone());
                        }
                    }
                    "verbosity" => {
                        verbosity_options.push(option_info.clone());
                    }
                    "norun_options" => {
                        // Keep in no-run options
                        if !categorized {
                            norun_options.push(option_info.clone());
                        }
                    }
                    _ => {
                        if !categorized {
                            processing_options.push(option_info.clone());
                        }
                    }
                }
                break;
            }
        }

        // If not categorized yet, use fallback logic
        if !categorized && !in_clap_group {
            // Handle verbosity options specially
            if matches!(
                option_info.name.as_str(),
                "verbose" | "quiet" | "normal_verbosity"
            ) {
                verbosity_options.push(option_info);
            } else if !matches!(
                option_info.name.as_str(),
                "script" | "args" | "help" | "version"
            ) {
                // Add other options to processing by default
                processing_options.push(option_info);
            }
        }
    }

    vec![
        OptionGroup {
            name: "Command Type".to_string(),
            options: dynamic_options,
            multiple: false,
        },
        OptionGroup {
            name: "Processing Options".to_string(),
            options: processing_options,
            multiple: true,
        },
        OptionGroup {
            name: "Filter Options".to_string(),
            options: filter_options,
            multiple: true,
        },
        OptionGroup {
            name: "Output Options".to_string(),
            options: output_options,
            multiple: true,
        },
        OptionGroup {
            name: "Verbosity".to_string(),
            options: verbosity_options,
            multiple: false,
        },
        OptionGroup {
            name: "No-run Options".to_string(),
            options: norun_options,
            multiple: true,
        },
    ]
}

fn format_option_display(option: &OptionInfo) -> String {
    let mut display = String::new();

    if let Some(short) = option.short {
        display.push_str(&format!("-{}", short));
        if !option.long.is_empty() {
            display.push_str(&format!(", --{}", option.long));
        }
    } else if !option.long.is_empty() {
        display.push_str(&format!("--{}", option.long));
    }

    if !option.help.is_empty() {
        // Truncate help text to fit better in the display
        let help_text = if option.help.len() > 60 {
            format!("{}...", &option.help[..57])
        } else {
            option.help.clone()
        };
        display.push_str(&format!(" - {}", help_text));
    }

    display
}

fn is_interactive() -> bool {
    use std::io::{self, IsTerminal};
    io::stdin().is_terminal()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "🚀 {} - Interactive Thag Builder",
        "Thag Prompt".bright_cyan()
    );
    println!("=========================================\n");

    // Check for test mode environment variable
    if let Ok(test_mode) = std::env::var("THAG_PROMPT_TEST") {
        return run_test_mode(&test_mode);
    }

    if !is_interactive() {
        eprintln!("Error: This tool requires an interactive terminal.");
        eprintln!("Please run it directly from a terminal, not through pipes or redirects.");
        eprintln!("Tip: Set THAG_PROMPT_TEST=repl to test REPL mode");
        std::process::exit(1);
    }

    let option_groups = extract_clap_metadata();
    let mut selected_options = Vec::new();
    let mut selected_values = HashMap::new();

    // Step 1: Ask user to choose between dynamic mode or script mode
    let mode_choice = Select::new(
        "Choose mode:",
        vec![
            "Dynamic mode (no script needed)",
            "Script mode (run a script file)",
        ],
    )
    .with_help_message("Dynamic mode: expressions, REPL, filters, etc. Script mode: run .rs files")
    .prompt()?;

    let (script_path, use_dynamic_mode) = if mode_choice == "Dynamic mode (no script needed)" {
        (None, true)
    } else {
        // Script mode - select a file
        let mut navigator = FileNavigator::new();
        println!("\n{}", "Step: Select a script file".bright_green());

        match select_file(&mut navigator, Some("rs"), false) {
            Ok(path) => (Some(path), false),
            Err(_) => {
                println!("No file selected. Exiting.");
                return Ok(());
            }
        }
    };

    // Step 2: If dynamic mode, select the dynamic option
    if use_dynamic_mode {
        let dynamic_group = option_groups
            .iter()
            .find(|g| g.name == "Command Type")
            .unwrap();

        if !dynamic_group.options.is_empty() {
            let dynamic_choices: Vec<String> = dynamic_group
                .options
                .iter()
                .map(|opt| format_option_display(opt))
                .collect();

            if let Ok(choice) = Select::new("Select dynamic option:", dynamic_choices.clone())
                .with_help_message("Choose what type of dynamic execution you want")
                .prompt()
            {
                let idx = dynamic_choices.iter().position(|c| c == &choice).unwrap();
                let selected_option = &dynamic_group.options[idx];
                selected_options.push(selected_option.name.clone());

                // Handle options that take values
                match selected_option.name.as_str() {
                    "expression" => {
                        let expr = Text::new("Enter Rust expression:")
                            .with_help_message("e.g. 2 + 2, println!(\"Hello\"), std::env::args().collect::<Vec<_>>()")
                            .prompt()?;
                        selected_values.insert(selected_option.name.clone(), expr);
                    }
                    "filter" => {
                        let filter = Text::new("Enter filter expression:")
                            .with_help_message("e.g. line.contains(\"error\"), line.len() > 10")
                            .prompt()?;
                        selected_values.insert(selected_option.name.clone(), filter);
                    }
                    _ => {}
                }
            }
        }
    }

    // Step 3: Select other options
    for group in &option_groups {
        if group.name == "Command Type" {
            continue; // Already handled
        }

        // Skip Filter Options if filter is not selected
        if group.name == "Filter Options" && !selected_options.contains(&"filter".to_string()) {
            continue;
        }

        if group.options.is_empty() {
            continue;
        }

        let choices: Vec<String> = group
            .options
            .iter()
            .map(|opt| format_option_display(opt))
            .collect();

        if group.multiple {
            if let Ok(selections) =
                MultiSelect::new(&format!("Select {}:", group.name), choices.clone())
                    .with_help_message("Use space to select, enter to confirm, ESC to skip")
                    .prompt_skippable()
            {
                if let Some(selections) = selections {
                    for selection in selections {
                        let idx = choices.iter().position(|c| c == &selection).unwrap();
                        let selected_option = &group.options[idx];
                        selected_options.push(selected_option.name.clone());

                        // Handle options that take values
                        if selected_option.takes_value {
                            match selected_option.name.as_str() {
                                "features" => {
                                    let features =
                                        Text::new("Enter features (comma-separated):").prompt()?;
                                    selected_values.insert(selected_option.name.clone(), features);
                                }
                                "infer" => {
                                    let infer_options = ["none", "min", "config", "max"];
                                    let infer_choice = Select::new(
                                        "Dependency inference level:",
                                        infer_options.to_vec(),
                                    )
                                    .prompt()?;
                                    selected_values.insert(
                                        selected_option.name.clone(),
                                        infer_choice.to_string(),
                                    );
                                }
                                "toml" => {
                                    let toml_input =
                                        Text::new("Enter manifest info (Cargo.toml format):")
                                            .with_help_message(
                                                "e.g. [dependencies]\nserde = \"1.0\"",
                                            )
                                            .prompt()?;
                                    selected_values
                                        .insert(selected_option.name.clone(), toml_input);
                                }
                                "begin" => {
                                    let begin_input = Text::new("Enter pre-loop Rust statements:")
                                        .with_help_message("e.g. let mut count = 0;")
                                        .prompt()?;
                                    selected_values
                                        .insert(selected_option.name.clone(), begin_input);
                                }
                                "end" => {
                                    let end_input = Text::new("Enter post-loop Rust statements:")
                                        .with_help_message("e.g. println!(\"Total: {}\", count);")
                                        .prompt()?;
                                    selected_values.insert(selected_option.name.clone(), end_input);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else {
            if let Ok(selection) = Select::new(&format!("Select {}:", group.name), choices.clone())
                .with_help_message("Press ESC to skip")
                .prompt_skippable()
            {
                if let Some(selection) = selection {
                    let idx = choices.iter().position(|c| c == &selection).unwrap();
                    let selected_option = &group.options[idx];
                    selected_options.push(selected_option.name.clone());

                    // Handle options that take values
                    if selected_option.takes_value {
                        match selected_option.name.as_str() {
                            "filter" => {
                                let filter = Text::new("Enter filter expression:")
                                    .with_help_message(
                                        "e.g. line.contains(\"error\"), line.len() > 10",
                                    )
                                    .prompt()?;
                                selected_values.insert(selected_option.name.clone(), filter);
                            }
                            "toml" => {
                                let toml_input =
                                    Text::new("Enter manifest info (Cargo.toml format):")
                                        .with_help_message("e.g. [dependencies]\nserde = \"1.0\"")
                                        .prompt()?;
                                selected_values.insert(selected_option.name.clone(), toml_input);
                            }
                            "begin" => {
                                let begin_input = Text::new("Enter pre-loop Rust statements:")
                                    .with_help_message("e.g. let mut count = 0;")
                                    .prompt()?;
                                selected_values.insert(selected_option.name.clone(), begin_input);
                            }
                            "end" => {
                                let end_input = Text::new("Enter post-loop Rust statements:")
                                    .with_help_message("e.g. println!(\"Total: {}\", count);")
                                    .prompt()?;
                                selected_values.insert(selected_option.name.clone(), end_input);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Step 4: Handle script arguments if script is selected
    let script_args = if script_path.is_some() {
        if let Ok(Some(args_input)) = Text::new("Enter script arguments (optional):")
            .with_help_message("Arguments to pass to the script (-- will be added automatically)")
            .prompt_skippable()
        {
            let mut args = vec!["--".to_string()];
            args.extend(args_input.split_whitespace().map(|s| s.to_string()));
            args
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Step 5: Build and execute the command
    let mut cmd = Command::new("thag");

    // Add selected options as arguments
    for option in &selected_options {
        match option.as_str() {
            "expression" => {
                cmd.arg("--expr");
                if let Some(expr) = selected_values.get(option) {
                    cmd.arg(expr);
                }
            }
            "filter" => {
                cmd.arg("--loop");
                if let Some(filter) = selected_values.get(option) {
                    cmd.arg(filter);
                }
            }
            "toml" => {
                cmd.arg("--toml");
                if let Some(toml_val) = selected_values.get(option) {
                    cmd.arg(toml_val);
                }
            }
            "begin" => {
                cmd.arg("--begin");
                if let Some(begin_val) = selected_values.get(option) {
                    cmd.arg(begin_val);
                }
            }
            "end" => {
                cmd.arg("--end");
                if let Some(end_val) = selected_values.get(option) {
                    cmd.arg(end_val);
                }
            }
            "repl" => {
                cmd.arg("--repl");
            }
            "stdin" => {
                cmd.arg("--stdin");
            }
            "edit" => {
                cmd.arg("--edit");
            }
            "config" => {
                cmd.arg("--config");
            }
            "verbose" => {
                cmd.arg("--verbose");
            }
            "quiet" => {
                cmd.arg("--quiet");
            }
            "normal_verbosity" => {
                cmd.arg("--normal");
            }
            "force" => {
                cmd.arg("--force");
            }
            "generate" => {
                cmd.arg("--gen");
            }
            "build" => {
                cmd.arg("--build");
            }
            "check" => {
                cmd.arg("--check");
            }
            "executable" => {
                cmd.arg("--executable");
            }
            "expand" => {
                cmd.arg("--expand");
            }
            "cargo" => {
                cmd.arg("--cargo");
            }
            "test_only" => {
                cmd.arg("--test-only");
            }
            "multimain" => {
                cmd.arg("--multimain");
            }
            "timings" => {
                cmd.arg("--timings");
            }
            "features" => {
                cmd.arg("--features");
                if let Some(features) = selected_values.get(option) {
                    cmd.arg(features);
                }
            }
            "infer" => {
                cmd.arg("--infer");
                if let Some(infer) = selected_values.get(option) {
                    cmd.arg(infer);
                }
            }
            "unquote" => {
                cmd.arg("--unquote");
            }
            _ => {} // Handle other options as needed
        }
    }

    // Add script path if selected
    if let Some(script) = script_path {
        cmd.arg(script);
    }

    // Add script arguments
    if !script_args.is_empty() {
        cmd.args(&script_args);
    }

    // Display and execute the command
    let cmd_display = format_command_display(&cmd);
    println!(
        "\n{} {}",
        "Running:".bright_green(),
        cmd_display.bright_cyan()
    );

    let status = cmd.status()?;

    if !status.success() {
        println!(
            "\n{} Command failed with exit code: {:?}",
            "Error:".bright_red(),
            status.code()
        );
    }

    Ok(())
}

fn run_test_mode(test_mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running in test mode: {}", test_mode);

    let mut cmd = Command::new("thag");

    match test_mode {
        "repl" => {
            cmd.arg("--repl");
        }
        "expr" => {
            cmd.arg("--expr").arg("2 + 2");
        }
        "expr_string" => {
            cmd.arg("--expr").arg("\"Hello world\"");
        }
        "expr_complex" => {
            cmd.arg("--expr")
                .arg("std::env::args().collect::<Vec<_>>()");
        }
        "stdin" => {
            cmd.arg("--stdin");
        }
        "script_with_args" => {
            cmd.arg("demo/hello.rs")
                .arg("--")
                .arg("--name")
                .arg("World")
                .arg("--verbose");
        }
        "filter_simple" => {
            cmd.arg("--loop").arg("line.len() > 3");
        }
        "filter_with_options" => {
            cmd.arg("--loop")
                .arg("if line.len() > 3 { count += 1; true } else { false }")
                .arg("--begin")
                .arg("let mut count = 0;")
                .arg("--end")
                .arg("println!(\"Total: {}\", count);")
                .arg("--toml")
                .arg("[dependencies]\nregex = \"1.0\"");
        }
        "debug_groups" => {
            // Test the option grouping
            let option_groups = extract_clap_metadata();
            println!("=== DEBUG: Option Groups ===");
            for group in &option_groups {
                println!("Group: {} (multiple: {})", group.name, group.multiple);
                for option in &group.options {
                    println!(
                        "  - {}: takes_value={}, help={}",
                        option.name, option.takes_value, option.help
                    );
                }
                println!();
            }
            return Ok(());
        }
        _ => {
            eprintln!("Unknown test mode: {}", test_mode);
            eprintln!(
                "Available modes: repl, expr, expr_string, expr_complex, stdin, script_with_args, filter_simple, filter_with_options, debug_groups"
            );
            std::process::exit(1);
        }
    }

    let cmd_display = format_command_display(&cmd);
    println!("Would execute: {}", cmd_display);

    Ok(())
}

fn format_command_display(cmd: &Command) -> String {
    let mut display = String::from("thag");

    for arg in cmd.get_args() {
        let arg_str = arg.to_string_lossy();
        display.push(' ');

        // Quote arguments that contain spaces or special characters
        if arg_str.contains(' ') || arg_str.contains('"') || arg_str.contains('\'') {
            display.push('\'');
            display.push_str(&arg_str.replace('\'', "'\"'\"'"));
            display.push('\'');
        } else {
            display.push_str(&arg_str);
        }
    }

    display
}
