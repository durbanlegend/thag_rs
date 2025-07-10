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

        // Check if this option belongs to a clap argument group
        let mut in_group = false;
        for (group_name, members) in &clap_groups {
            if members.contains(&option_info.name) {
                in_group = true;
                match group_name.as_str() {
                    "commands" => dynamic_options.push(option_info.clone()),
                    "verbosity" => verbosity_options.push(option_info.clone()),
                    "norun_options" => norun_options.push(option_info.clone()),
                    _ => processing_options.push(option_info.clone()),
                }
                break;
            }
        }

        // If not in a clap group, categorize by help heading
        if !in_group {
            match option_info.group.as_deref() {
                Some("Output Options") => output_options.push(option_info),
                Some("Processing Options") => processing_options.push(option_info),
                Some("Dynamic Options (no script)") => dynamic_options.push(option_info),
                Some("No-run Options") => norun_options.push(option_info),
                _ => {
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

fn needs_script(selected_options: &[String]) -> bool {
    // Check if any dynamic options are selected that don't need a script
    let no_script_options = ["expression", "repl", "stdin", "edit", "filter", "config"];

    !selected_options
        .iter()
        .any(|opt| no_script_options.contains(&opt.as_str()))
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "🚀 {} - Interactive Thag Builder",
        "Thag Prompt".bright_cyan()
    );
    println!("=========================================\n");

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
                if selected_option.takes_value {
                    match selected_option.name.as_str() {
                        "expression" => {
                            let expr = Text::new("Enter Rust expression:").prompt()?;
                            selected_values.insert(selected_option.name.clone(), expr);
                        }
                        "filter" => {
                            let filter = Text::new("Enter filter expression:").prompt()?;
                            selected_values.insert(selected_option.name.clone(), filter);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Step 3: Select other options
    for group in &option_groups {
        if group.name == "Command Type" {
            continue; // Already handled
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
                }
            }
        }
    }

    // Step 4: Handle script arguments if script is selected
    let script_args = if script_path.is_some() {
        let args_input = Text::new("Enter script arguments (optional):")
            .with_help_message("Arguments to pass to the script")
            .prompt_skippable()?;

        args_input
            .map(|s| {
                s.split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
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
    let mut cmd_str = format!("{:?}", cmd);
    cmd_str.retain(|c| c != '"');
    println!("\n{} {}", "Running:".bright_green(), cmd_str.bright_cyan());

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
