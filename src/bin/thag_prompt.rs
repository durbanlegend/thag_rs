/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["tools"] }
*/

/// Basic prompted front-end to build and run a `thag` command.
//# Purpose: Simplify running `thag`.
//# Categories: cli, interactive, thag_front_ends, tools
#[cfg(feature = "clipboard")]
use arboard::Clipboard;
use clap::CommandFactory;
use inquire::{set_global_render_config, MultiSelect};
use std::collections::HashMap;
use std::fmt::Write as _; // import without risk of name clashing
use std::process::Command;
use std::string::ToString;
use thag_proc_macros::file_navigator;
use thag_rs::{
    auto_help, cprtln, help_system::check_help_and_exit, themed_inquire_config, Role, Style,
};
use thag_styling::Styler;

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

#[allow(clippy::too_many_lines)]
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
            help: arg.get_help().map_or_else(String::new, ToString::to_string),
            takes_value: arg.get_action().takes_values(),
            group: arg.get_help_heading().map(ToString::to_string),
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
        let _ = writeln!(display, "-{}", short);
        if !option.long.is_empty() {
            let _ = writeln!(display, ", --{}", option.long);
        }
    } else if !option.long.is_empty() {
        let _ = writeln!(display, "--{}", option.long);
    }

    if !option.help.is_empty() {
        // Truncate help text to fit better in the display
        let help_text = if option.help.len() > 60 {
            format!("{}...", &option.help[..57])
        } else {
            option.help.clone()
        };
        let _ = writeln!(display, " - {help_text}");
    }

    display
}

fn is_interactive() -> bool {
    use std::io::{self, IsTerminal};
    io::stdin().is_terminal()
}

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_prompt");
    check_help_and_exit(&help);

    set_global_render_config(themed_inquire_config());

    cprtln!(
        Style::for_role(Role::Heading3),
        "🚀 Thag Prompt - Interactive Thag Builder",
    );
    println!("{}\n", "═".repeat(41));

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
        cprtln!(
            Style::for_role(Role::Emphasis),
            "\nStep: Select a script file"
        );

        if let Ok(path) = select_file(&mut navigator, Some("rs"), false) {
            (Some(path), false)
        } else {
            println!("No file selected. Exiting.");
            return Ok(());
        }
    };

    // Step 2: If dynamic mode, select the dynamic option
    if use_dynamic_mode {
        let dynamic_group = option_groups
            .iter()
            .find(|g| g.name == "Command Type")
            .unwrap();

        eprintln!("dynamic_group={dynamic_group:#?}");

        if !dynamic_group.options.is_empty() {
            let dynamic_choices: Vec<String> = dynamic_group
                .options
                .iter()
                .filter(|v| v.name != "script")
                .map(format_option_display)
                .collect();

            if let Ok(choice) = Select::new("Select dynamic option:", dynamic_choices.clone())
                .with_help_message("Choose what type of dynamic execution you want")
                .prompt()
            {
                let idx = dynamic_choices.iter().position(|c| c == &choice).unwrap();
                let selected_option = &dynamic_group.options[idx];
                dbg!(selected_option.name.clone());
                selected_options.push(selected_option.name.clone());

                // Handle options that take values
                match selected_option.name.as_str() {
                    "expression" => {
                        let expr = Text::new("Enter Rust expression:")
                            .with_help_message(r#"e.g. 5 + 3, "Hi", println!("Hello world!");, std::env::args().collect::<Vec<_>>(), '(1..=20).product::<usize>()' "#)
                            .prompt()?;
                        selected_values.insert(selected_option.name.clone(), expr);
                    }
                    "filter" => {
                        let filter = Text::new("Enter filter expression:")
                            .with_help_message(r#"e.g. line.contains("error"), line.len() > 10"#)
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
        if group.name == "Command Type" || group.name == "Output Options" {
            continue; // Already handled
        }

        // Skip Filter Options if filter is not selected
        if group.name == "Filter Options" && !selected_options.contains(&"filter".to_string()) {
            continue;
        }

        // Handle verbosity specially - it's a single choice with count options
        if group.name == "Verbosity" {
            let verbosity_choices = vec![
                "Default: Normal verbosity (-n)",
                "Verbose (-v)",
                "Debug (-vv)",
                "Quiet (-q)",
                "Very quiet (-qq)",
            ];

            if let Ok(Some(selection)) = Select::new("Select verbosity level:", verbosity_choices)
                .with_help_message("Choose output verbosity level")
                .prompt_skippable()
            {
                match selection {
                    "Verbose (-v)" => {
                        selected_options.push("verbose".to_string());
                        selected_values.insert("verbose".to_string(), "1".to_string());
                    }
                    "Debug (-vv)" => {
                        selected_options.push("verbose".to_string());
                        selected_values.insert("verbose".to_string(), "2".to_string());
                    }
                    "Quiet (-q)" => {
                        selected_options.push("quiet".to_string());
                        selected_values.insert("quiet".to_string(), "1".to_string());
                    }
                    "Very quiet (-qq)" => {
                        selected_options.push("quiet".to_string());
                        selected_values.insert("quiet".to_string(), "2".to_string());
                    }
                    "Normal verbosity" => {
                        selected_options.push("normal_verbosity".to_string());
                    }
                    _ => {}
                }
            }
            continue;
        }

        // Handle input and environment setup (thag_prompt-specific features)
        if group.name == "Processing Options" {
            let choices: Vec<String> = group.options.iter().map(format_option_display).collect();

            // Add input file and environment variable options to the choices
            let mut extended_choices = choices.clone();
            extended_choices.push("📁 Input file (pipe from file)".to_string());
            extended_choices.push("🌍 Environment variables".to_string());

            if let Ok(Some(selections)) =
                MultiSelect::new(&format!("Select {}:", group.name), extended_choices.clone())
                    .with_help_message("Use space to select, enter to confirm, ESC to skip")
                    .prompt_skippable()
            {
                for selection in selections {
                    if selection == "📁 Input file (pipe from file)" {
                        let input_file = Text::new("Input file to pipe to stdin:")
                            .with_help_message(
                                "File path (e.g. data.txt) - alternative to shell redirection",
                            )
                            .prompt()?;
                        selected_options.push("input_file".to_string());
                        selected_values.insert("input_file".to_string(), input_file);
                    } else if selection == "🌍 Environment variables" {
                        let env_vars =
                            Text::new("Environment variables (KEY=VALUE, comma-separated):")
                                .with_help_message(
                                    "e.g. RUST_LOG=debug,MY_VAR=$PWD (supports $VAR expansion)",
                                )
                                .prompt()?;
                        selected_options.push("env_vars".to_string());
                        selected_values.insert("env_vars".to_string(), env_vars);
                    } else {
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
            continue;
        }

        if group.options.is_empty() {
            continue;
        }

        let choices: Vec<String> = group.options.iter().map(format_option_display).collect();

        if group.multiple {
            if let Ok(Some(selections)) =
                MultiSelect::new(&format!("Select {}:", group.name), choices.clone())
                    .with_help_message("Use space to select, enter to confirm, ESC to skip")
                    .prompt_skippable()
            {
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
                                selected_values
                                    .insert(selected_option.name.clone(), infer_choice.to_string());
                            }
                            "toml" => {
                                let toml_input =
                                    Text::new("Enter manifest info (Cargo.toml format):")
                                        .with_help_message(
                                            r#"e.g. [dependencies]
serde = "1.0""#,
                                        )
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
            } else if let Ok(Some(selection)) =
                Select::new(&format!("Select {}:", group.name), choices.clone())
                    .with_help_message("Press ESC to skip")
                    .prompt_skippable()
            {
                let idx = choices.iter().position(|c| c == &selection).unwrap();
                let selected_option = &group.options[idx];
                selected_options.push(selected_option.name.clone());

                // Handle options that take values
                if selected_option.takes_value {
                    match selected_option.name.as_str() {
                        "filter" => {
                            let filter = Text::new("Enter filter expression:")
                                .with_help_message("e.g. line.contains(\"error\"), line.len() > 10")
                                .prompt()?;
                            selected_values.insert(selected_option.name.clone(), filter);
                        }
                        "toml" => {
                            let toml_input = Text::new("Enter manifest info (Cargo.toml format):")
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
                        "features" => {
                            let features =
                                Text::new("Enter features (comma-separated):").prompt()?;
                            selected_values.insert(selected_option.name.clone(), features);
                        }
                        "infer" => {
                            let infer_options = ["none", "min", "config", "max"];
                            let infer_choice =
                                Select::new("Dependency inference level:", infer_options.to_vec())
                                    .prompt()?;
                            selected_values
                                .insert(selected_option.name.clone(), infer_choice.to_string());
                        }
                        _ => {}
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
            if args_input.trim().is_empty() {
                Vec::new()
            } else {
                let mut args = vec!["--".to_string()];
                args.extend(args_input.split_whitespace().map(ToString::to_string));
                args
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Step 5: Handle additional input and environment options
    let mut input_file_option = None;
    let mut env_vars_option = None;

    // Ask for input file if not already selected
    if !selected_values.contains_key("input_file") {
        if let Ok(Some(input_file)) = Text::new("Input file (optional):")
            .with_help_message("File to pipe to stdin (leave empty to skip)")
            .prompt_skippable()
        {
            if !input_file.trim().is_empty() {
                input_file_option = Some(input_file);
            }
        }
    }

    // Ask for environment variables if not already selected
    if !selected_values.contains_key("env_vars") {
        if let Ok(Some(env_vars)) = Text::new("Environment variables (optional):")
            .with_help_message("KEY=VALUE pairs, comma-separated (supports $VAR expansion)")
            .prompt_skippable()
        {
            if !env_vars.trim().is_empty() {
                env_vars_option = Some(env_vars);
            }
        }
    }

    // Step 6: Ask about output format
    let output_choices = vec![
        "Execute command",
        "Copy command to clipboard",
        "Print command to stdout",
    ];

    let output_choice = Select::new("How would you like to proceed?", output_choices)
        .with_help_message("Choose execution method")
        .prompt()?;

    // Step 7: Build the command
    let mut cmd = Command::new("thag");

    // eprintln!("selected_options={selected_options:#?}");

    // Add selected options as arguments
    for option in &selected_options {
        match option.as_str() {
            "expression" => {
                cmd.arg("-e");
                if let Some(expr) = selected_values.get(option) {
                    eprintln!("expr={expr}");
                    // cmd.arg(expr);
                    // cmd.arg(expr.strip_prefix("'").unwrap_or(expr).strip_suffix("'"));
                    cmd.arg(expr.trim_matches('\''));
                }
            }
            "filter" => {
                cmd.arg("-l");
                if let Some(filter) = selected_values.get(option) {
                    cmd.arg(filter);
                }
            }
            "toml" => {
                cmd.arg("-M");
                if let Some(toml_val) = selected_values.get(option) {
                    cmd.arg(toml_val);
                }
            }
            "begin" => {
                cmd.arg("-B");
                if let Some(begin_val) = selected_values.get(option) {
                    cmd.arg(begin_val);
                }
            }
            "end" => {
                cmd.arg("-E");
                if let Some(end_val) = selected_values.get(option) {
                    cmd.arg(end_val);
                }
            }
            "repl" => {
                cmd.arg("-r");
            }
            "stdin" => {
                cmd.arg("-s");
            }
            "edit" => {
                cmd.arg("-d");
            }
            "config" => {
                cmd.arg("-C");
            }
            "verbose" => {
                if let Some(count) = selected_values.get(option) {
                    let count: u8 = count.parse().unwrap_or(1);
                    // for _ in 0..count {
                    //     cmd.arg("--verbose");
                    // }
                    if count > 1 {
                        cmd.arg("-vv");
                    }
                } else {
                    cmd.arg("-v");
                }
            }
            "quiet" => {
                if let Some(count) = selected_values.get(option) {
                    let count: u8 = count.parse().unwrap_or(1);
                    // for _ in 0..count {
                    //     cmd.arg("--quiet");
                    // }
                    if count > 1 {
                        cmd.arg("-qq");
                    }
                } else {
                    cmd.arg("-q");
                }
            }
            "normal_verbosity" => {
                cmd.arg("-n");
            }
            "force" => {
                cmd.arg("-f");
            }
            "generate" => {
                cmd.arg("-g");
            }
            "build" => {
                cmd.arg("-b");
            }
            "check" => {
                cmd.arg("-c");
            }
            "executable" => {
                cmd.arg("-x");
            }
            "expand" => {
                cmd.arg("-X");
            }
            "cargo" => {
                cmd.arg("-A");
            }
            "test_only" => {
                cmd.arg("-T");
            }
            "multimain" => {
                cmd.arg("-m");
            }
            "timings" => {
                cmd.arg("-t");
            }
            "features" => {
                cmd.arg("--features");
                if let Some(features) = selected_values.get(option) {
                    cmd.arg(features);
                }
            }
            "infer" => {
                cmd.arg("-i");
                if let Some(infer) = selected_values.get(option) {
                    cmd.arg(infer);
                }
            }
            "unquote" => {
                cmd.arg("-u");
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

    // Handle input file` - either from selection or prompt
    let input_file_path = selected_values
        .get("input_file")
        .cloned()
        .or(input_file_option);
    let input_file_info = input_file_path
        .as_ref()
        .map(|input_file| format!(" < {}", input_file));

    // Handle environment variables - either from selection or prompt
    let env_input = selected_values.get("env_vars").cloned().or(env_vars_option);
    let mut env_vars_display = Vec::new();

    if let Some(env_input) = &env_input {
        for env_pair in env_input.split(',') {
            let env_pair = env_pair.trim();
            if let Some((key, value)) = env_pair.split_once('=') {
                let expanded_value = expand_env_vars(value.trim());
                env_vars_display.push(format!("{}={}", key.trim(), expanded_value));
            } else {
                eprintln!("Warning: Invalid environment variable format: {}", env_pair);
                eprintln!("Expected format: KEY=VALUE");
            }
        }
    }

    let env_vars_info = if env_vars_display.is_empty() {
        None
    } else {
        Some(format!(" (env: {})", env_vars_display.join(", ")))
    };

    // Build command display string
    let mut cmd_display = format_command_display(&cmd);
    if let Some(input_info) = input_file_info {
        cmd_display.push_str(&input_info);
    }
    if let Some(env_info) = env_vars_info {
        cmd_display.push_str(&env_info);
    }

    // Handle environment variables prefix for shell execution
    let env_prefix = if env_vars_display.is_empty() {
        String::new()
    } else {
        format!("{} ", env_vars_display.join(" "))
    };

    match output_choice {
        "Execute command" => {
            // Set up stdin redirection if specified
            if let Some(input_file) = input_file_path {
                use std::fs::File;
                use std::process::Stdio;

                let file = File::open(&input_file)
                    .map_err(|e| format!("Failed to open input file '{}': {}", input_file, e))?;
                cmd.stdin(Stdio::from(file));
            }

            // Set environment variables
            if let Some(env_input) = env_input {
                for env_pair in env_input.split(',') {
                    let env_pair = env_pair.trim();
                    if let Some((key, value)) = env_pair.split_once('=') {
                        let expanded_value = expand_env_vars(value.trim());
                        cmd.env(key.trim(), expanded_value);
                    }
                }
            }

            cprtln!(Style::for_role(Role::Heading3), "\n{}", "Running:".bold());
            cprtln!(Style::for_role(Role::Code), "{cmd_display}");

            let status = cmd.status()?;

            if !status.success() {
                cprtln!(
                    Style::for_role(Role::Error),
                    "\nError: Command failed with exit code: {:?}",
                    status.code()
                );
            }
        }
        "Copy command to clipboard" => {
            let shell_command = format!("{}{}", env_prefix, cmd_display);
            cprtln!(
                Style::for_role(Role::Info),
                "\nInfo: Command copied to clipboard:",
            );
            cprtln!(Style::for_role(Role::Code), "{shell_command}");

            // Try to copy to clipboard (cross-platform)
            if let Err(e) = copy_to_clipboard(&shell_command) {
                cprtln!(
                    Style::for_role(Role::Warning),
                    "Warning: Failed to copy to clipboard: {e}"
                );
                println!("Please copy the command above manually.");
            }
        }
        "Print command to stdout" => {
            let shell_command = format!("{}{}", env_prefix, cmd_display);
            cprtln!(Style::for_role(Role::Code), "{shell_command}");
        }
        _ => {}
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_test_mode(test_mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running in test mode: {test_mode}");

    let mut cmd = Command::new("thag");

    match test_mode {
        "repl" => {
            cmd.arg("--repl");
        }
        "expr" => {
            cmd.arg("--expr").arg("2 + 2");
        }
        "expr_string" => {
            cmd.arg("--expr").arg(r#""Hello world""#);
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
                .arg(r#"println!("Total: {}", count);"#)
                .arg("--toml")
                .arg(
                    r#"[dependencies]
regex = "1.0""#,
                );
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
        "test_input_file" => {
            // Simulate selecting input file and env vars
            let mut test_values = HashMap::new();
            test_values.insert("input_file".to_string(), "demo/hello.rs".to_string());
            test_values.insert(
                "env_vars".to_string(),
                "TEST_VAR=hello,DEBUG=$PWD".to_string(),
            );

            cmd.arg("--loop").arg("line.len() > 0");

            // Apply input file
            if let Some(input_file) = test_values.get("input_file") {
                use std::fs::File;
                use std::process::Stdio;
                let file = File::open(input_file)?;
                cmd.stdin(Stdio::from(file));
            }

            // Apply env vars with expansion
            if let Some(env_input) = test_values.get("env_vars") {
                for env_pair in env_input.split(',') {
                    let env_pair = env_pair.trim();
                    if let Some((key, value)) = env_pair.split_once('=') {
                        let expanded_value = expand_env_vars(value.trim());
                        cmd.env(key.trim(), expanded_value);
                    }
                }
            }
        }
        "test_env_vars" => {
            cmd.arg("--expr")
                .arg(r#"std::env::var("CUSTOM_VAR").unwrap_or_else(|_| "not set".to_string())"#)
                .env("CUSTOM_VAR", "hello_world")
                .env("DEBUG", "1");
        }
        "test_env_expansion" => {
            // Test environment variable expansion like $PWD
            std::env::set_var("TEST_EXPAND", "expanded_value");
            cmd.arg("--expr")
                .arg("println!(\"Environment variable resolved\")")
                .env("SIMPLE_VAR", expand_env_vars("$PWD"))
                .env(
                    "COMPLEX_VAR",
                    expand_env_vars("prefix_${TEST_EXPAND}_suffix"),
                );
        }
        "test_display_enhanced" => {
            // Test enhanced command display with input file and env vars
            cmd.arg("--loop").arg("line.contains(\"hello\")");

            // Simulate input file redirection
            if let Ok(file) = std::fs::File::open("demo/hello.rs") {
                cmd.stdin(std::process::Stdio::from(file));
            }

            // Add environment variables
            cmd.env("RUST_LOG", "debug");
            cmd.env("MY_PATH", "/custom/path");

            // This would show: thag --loop 'line.contains("hello")' < demo/hello.rs (env: RUST_LOG=debug, MY_PATH=/custom/path)
        }
        "test_verbosity_double" => {
            cmd.arg("--expr")
                .arg("println!(\"Testing debug verbosity\")")
                .arg("--verbose")
                .arg("--verbose"); // Test -vv
        }
        "test_no_script_args" => {
            cmd.arg("demo/hello.rs");
            // Test that no -- is added when script_args is empty
        }
        "test_clipboard" => {
            // Test clipboard functionality
            let test_text = "thag --expr 'println!(\"Hello from clipboard test!\")'";
            match copy_to_clipboard(test_text) {
                Ok(()) => println!("Clipboard test successful"),
                Err(e) => println!("Clipboard test failed: {}", e),
            }
            return Ok(());
        }
        _ => {
            eprintln!("Unknown test mode: {}", test_mode);
            eprintln!(
                "Available modes: repl, expr, expr_string, expr_complex, stdin, script_with_args, filter_simple, filter_with_options, debug_groups, test_input_file, test_env_vars, test_env_expansion, test_verbosity_double, test_no_script_args, test_display_enhanced"
            );
            std::process::exit(1);
        }
    }

    let cmd_display = format_command_display(&cmd);
    println!("Would execute: {}", cmd_display);

    Ok(())
}

/// Copy text to clipboard using arboard (cross-platform)
fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "clipboard")]
    {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }

    #[cfg(not(feature = "clipboard"))]
    {
        // Fallback to thag_copy if clipboard feature not available
        let mut child = std::process::Command::new("thag_copy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().ok_or("Failed to get stdin")?;
            use std::io::Write;
            stdin.write_all(text.as_bytes())?;
        }

        let status = child.wait()?;
        if !status.success() {
            return Err(format!(
                "thag_copy failed with exit code: {}",
                status.code().unwrap_or(-1)
            )
            .into());
        }
        Ok(())
    }
}

/// Expand environment variables in a string (e.g., $PWD, ${HOME})
fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();

    // Handle ${VAR} format
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 2..start + end];
            let replacement = std::env::var(var_name).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Environment variable '{}' not found, using empty string",
                    var_name
                );
                String::new()
            });
            result.replace_range(start..=(start + end), &replacement);
        } else {
            break; // Malformed ${...} - stop processing
        }
    }

    // Handle $VAR format (stops at word boundaries)
    let re = regex::Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    let result = re.replace_all(&result, |caps: &regex::Captures| {
        let var_name = &caps[1];
        std::env::var(var_name).unwrap_or_else(|_| {
            eprintln!(
                "Warning: Environment variable '{}' not found, using empty string",
                var_name
            );
            String::new()
        })
    });

    result.to_string()
}

fn format_command_display(cmd: &Command) -> String {
    let mut display = String::from("thag");

    for arg in cmd.get_args() {
        let arg_str = arg.to_string_lossy();
        display.push(' ');

        // eprintln!("arg_str={arg_str}");

        // Quote arguments that contain spaces or special characters
        let in_single_quotes = arg_str.starts_with('\'') && arg_str.ends_with('\'');
        if in_single_quotes {
            // let arg_str = arg_str.trim_matches('\'');
            let _ = writeln!(display, "'{arg_str}'");
            // eprintln!("1. display={display}");
        } else if arg_str.contains(' ') || arg_str.contains('"') || arg_str.contains('\'') {
            display.push('\'');
            display.push_str(&arg_str.replace('\'', r#"'"'"'"#));
            display.push('\'');
            // eprintln!("2. display={display}");
        } else {
            let _ = writeln!(display, "'{arg_str}'");
            // eprintln!("3. display={display}");
        }
    }

    display
}
