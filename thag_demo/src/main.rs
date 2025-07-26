//! `thag_demo` - Interactive demos for `thag_rs` and `thag_profiler`
//!
//! This crate provides a simple way to run profiling demos without installing `thag_rs`.
//! It acts as a lightweight facade over `thag_rs` functionality.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use inquire::{Confirm, Select, Text};
use std::path::{Path, PathBuf};
use std::process;
use std::{env, fs, io};
use thag_rs::{builder::execute, configure_log, Cli};
use thag_rs::{get_verbosity, set_global_verbosity, V};

pub mod visualization;

/// Metadata about a demo file
#[derive(Debug, Clone)]
struct DemoFile {
    name: String,
    path: PathBuf,
    description: String,
    categories: Vec<String>,
}

/// Represents possible locations for the demo directory
#[derive(Debug)]
enum DemoLocation {
    Sibling(PathBuf),
    UserConfig(PathBuf),
    Environment(PathBuf),
    Current(PathBuf),
}

/// Demo runner for `thag_rs` and `thag_profiler` examples
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "thag_demo")]
struct DemoArgs {
    /// Demo to run
    #[command(subcommand)]
    demo: Option<DemoCommand>,

    /// List all available demos
    #[arg(short, long)]
    list: bool,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum DemoCommand {
    /// Basic profiling demo showing function timing
    BasicProfiling,

    /// Memory allocation tracking demo
    MemoryProfiling,

    /// Async function profiling demo
    AsyncProfiling,

    /// Before/after comparison demo
    Comparison,

    /// Differential comparison demo with before/after analysis
    DifferentialComparison,

    /// Interactive flamegraph demo
    Flamegraph,

    /// Full benchmark profiling demo
    Benchmark,

    /// Interactive profiling demo with embedded analysis
    InteractiveProfiling,

    /// Run a specific demo script by name
    Script {
        /// Name of the demo script to run
        name: String,
    },

    /// Interactive browser for demo scripts
    Browse,

    /// Manage demo directory (download, update, set location)
    Manage,

    /// List all available demo scripts
    ListScripts,
}

/// Discover demo directory in various locations
fn discover_demo_dir() -> Option<PathBuf> {
    let locations = vec![
        // Sibling to thag_demo (original behavior)
        DemoLocation::Sibling(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("demo"),
        ),
        // User's home directory
        DemoLocation::UserConfig(
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".thag")
                .join("demo"),
        ),
        // Environment variable
        DemoLocation::Environment(
            env::var("THAG_DEMO_DIR")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("")),
        ),
        // Current working directory
        DemoLocation::Current(PathBuf::from("demo")),
    ];

    for location in locations {
        let path = match location {
            DemoLocation::Sibling(p) | DemoLocation::UserConfig(p) | DemoLocation::Current(p) => p,
            DemoLocation::Environment(p) => {
                if p.as_os_str().is_empty() {
                    continue;
                }
                p
            }
        };

        if path.exists() && path.is_dir() {
            return Some(path);
        }
    }

    None
}

/// Find demo directory quietly (for listing purposes)
fn find_demo_dir_quietly() -> Result<PathBuf> {
    discover_demo_dir()
        .ok_or_else(|| anyhow::anyhow!("Demo directory not found in any standard location"))
}

/// Find or setup demo directory with user interaction
fn find_or_setup_demo_dir() -> Result<PathBuf> {
    if let Some(dir) = discover_demo_dir() {
        return Ok(dir);
    }

    println!(
        "{}",
        "Demo directory not found in any standard location.".yellow()
    );
    println!("Checked locations:");
    println!("  • Sibling to thag_demo");
    println!("  • ~/.thag/demo");
    println!("  • $THAG_DEMO_DIR");
    println!("  • ./demo");
    println!();

    match Confirm::new("Would you like to download the demo directory?")
        .with_default(true)
        .prompt()
    {
        Ok(should_download) => {
            if should_download {
                download_demo_dir()
            } else {
                Err(anyhow::anyhow!("Demo directory required but not available"))
            }
        }
        Err(e) => {
            println!("❌ Interactive prompt failed: {}", e);
            println!("💡 You can manually download with: thag thag_get_demo_dir");
            Err(anyhow::anyhow!(
                "Demo directory setup required but interactive prompt failed"
            ))
        }
    }
}

/// Download demo directory using thag_get_demo_dir
fn download_demo_dir() -> Result<PathBuf> {
    let default_location = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".thag");

    let location_str = match Text::new("Where should the demo directory be installed?")
        .with_default(&default_location.to_string_lossy())
        .with_help_message("The 'demo' subdirectory will be created here")
        .prompt()
    {
        Ok(location) => location,
        Err(e) => {
            println!("❌ Interactive prompt failed: {}", e);
            println!("💡 Using default location: {}", default_location.display());
            default_location.to_string_lossy().to_string()
        }
    };

    let location = PathBuf::from(&location_str);

    // Ensure parent directory exists
    if !location.exists() {
        fs::create_dir_all(&location)?;
    }

    println!("{}", "Downloading demo directory...".cyan());

    // Call thag_get_demo_dir subprocess
    let output = std::process::Command::new("thag")
        .args(&["thag_get_demo_dir"])
        .env("THAG_DEMO_TARGET", &location)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let demo_path = location.join("demo");
                println!("✅ Demo directory downloaded to: {}", demo_path.display());
                Ok(demo_path)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!(
                    "Failed to download demo directory: {}",
                    stderr
                ))
            }
        }
        Err(e) => {
            eprintln!("{}", "Failed to run thag_get_demo_dir. Make sure thag_rs is installed with tools feature.".red());
            Err(anyhow::anyhow!("Command execution failed: {}", e))
        }
    }
}

/// Extract description and categories from demo file content
fn extract_demo_metadata(path: &Path) -> Result<Option<DemoFile>> {
    let content = fs::read_to_string(path)?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut description = None;
    let mut categories = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut _in_doc_comment = false;

    for line in lines {
        let trimmed = line.trim();

        // Look for doc comments (///)
        if trimmed.starts_with("///") {
            _in_doc_comment = true;
            let comment_text = trimmed.trim_start_matches("///").trim();
            if !comment_text.is_empty() && description.is_none() {
                description = Some(comment_text.to_string());
            }
        }
        // Look for categories comment (//# Categories:)
        else if trimmed.starts_with("//# Categories:") {
            let cats_text = trimmed.trim_start_matches("//# Categories:").trim();
            categories = cats_text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        // Stop at first non-comment line
        else if !trimmed.starts_with("//") && !trimmed.is_empty() {
            break;
        }
    }

    let final_description = description.unwrap_or_else(|| "No description available".to_string());

    Ok(Some(DemoFile {
        name,
        path: path.to_path_buf(),
        description: final_description,
        categories,
    }))
}

/// Scan demo directory for .rs files and extract metadata
fn scan_demo_files(demo_dir: &Path) -> Result<Vec<DemoFile>> {
    let mut demos = Vec::new();

    for entry in fs::read_dir(demo_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "rs") {
            if let Some(demo) = extract_demo_metadata(&path)? {
                demos.push(demo);
            }
        }
    }

    demos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(demos)
}

/// Interactive demo browser similar to thag_show_themes
fn interactive_demo_browser(verbose: bool) -> Result<()> {
    let demo_dir = find_or_setup_demo_dir()?;
    let demo_files = scan_demo_files(&demo_dir)?;

    if demo_files.is_empty() {
        println!("{}", "No demo files found in directory.".yellow());
        return Ok(());
    }

    // Create demo options with descriptions
    let demo_options: Vec<String> = demo_files
        .iter()
        .map(|demo| {
            let cats = if demo.categories.is_empty() {
                String::new()
            } else {
                format!(" [{}]", demo.categories.join(", "))
            };
            format!("{} - {}{}", demo.name, demo.description, cats)
        })
        .collect();

    // Clear screen initially
    print!("\x1b[2J\x1b[H");

    let mut cursor = 0_usize;
    use inquire::error::InquireResult;
    use inquire::list_option::ListOption;

    loop {
        println!("\n🚀 Interactive Demo Browser");
        println!("{}", "═".repeat(80));
        println!("📚 {} demo scripts available", demo_files.len());
        println!("💡 Start typing to filter demos by name");
        println!("{}", "═".repeat(80));

        let selection: InquireResult<ListOption<String>> =
            Select::new("🔍 Select a demo script to run:", demo_options.clone())
                .with_page_size(20)
                .with_help_message("↑↓ navigate • type to filter • Enter to run • Esc to quit")
                .with_reset_cursor(false)
                .with_starting_cursor(cursor)
                .raw_prompt();

        match selection {
            Ok(selected) => {
                cursor = selected.index;

                // Extract demo name from selection (before the " - " separator)
                let demo_name = selected
                    .value
                    .split(" - ")
                    .next()
                    .unwrap_or(&selected.value);

                // Clear screen for better presentation
                print!("\x1b[2J\x1b[H");

                println!(
                    "{}",
                    format!("Running demo script: {}", demo_name).bold().green()
                );
                println!();

                match run_selected_demo(&demo_dir, demo_name, verbose) {
                    Ok(()) => {
                        println!("\n{}", "═".repeat(80));
                        println!("🔙 Press Enter to return to demo browser, or Ctrl+C to exit...");
                        let _ = io::stdin().read_line(&mut String::new());
                        // Clear screen before returning to menu
                        print!("\x1b[2J\x1b[H");
                    }
                    Err(e) => {
                        println!("❌ Error running demo '{}': {}", demo_name, e);
                        println!("Press Enter to continue...");
                        let _ = io::stdin().read_line(&mut String::new());
                        print!("\x1b[2J\x1b[H");
                    }
                }
            }
            Err(inquire::InquireError::OperationCanceled) => {
                print!("\x1b[2J\x1b[H");
                println!("👋 Thanks for using the demo browser!");
                break;
            }
            Err(e) => {
                println!("❌ Interactive prompt failed: {}", e);
                println!("💡 Fallback: Use 'thag_demo list-scripts' to see all available demos");
                println!("   Then run: 'thag_demo script <demo_name>'");
                break;
            }
        }
    }

    Ok(())
}

/// Run a selected demo from the browser
fn run_selected_demo(demo_dir: &Path, demo_name: &str, verbose: bool) -> Result<()> {
    let demo_path = demo_dir.join(format!("{}.rs", demo_name));

    if !demo_path.exists() {
        return Err(anyhow::anyhow!(
            "Demo file not found: {}",
            demo_path.display()
        ));
    }

    // Set THAG_DEV_PATH for local development - point to thag_rs root
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let thag_rs_root = current_dir.parent().unwrap_or(&current_dir);
    env::set_var("THAG_DEV_PATH", thag_rs_root);

    let mut cli = create_demo_cli(&demo_path, verbose);

    set_global_verbosity(if verbose { V::D } else { V::N })?;

    configure_log();

    execute(&mut cli)?;
    Ok(())
}

/// List all available demos (built-in and scripts)
fn list_all_demos() -> Result<()> {
    println!("{}", "Built-in demos:".bold().green());
    println!();

    let demos = vec![
        ("basic-profiling", "Basic function timing and profiling"),
        (
            "memory-profiling",
            "Memory allocation tracking and analysis",
        ),
        ("async-profiling", "Profiling async functions and futures"),
        ("comparison", "Before/after performance comparison"),
        ("flamegraph", "Interactive flamegraph generation"),
        ("benchmark", "Full benchmark profiling with detailed output"),
        (
            "interactive-profiling",
            "Interactive profiling with embedded analysis",
        ),
        (
            "differential-comparison",
            "True differential comparison with before/after analysis",
        ),
    ];

    for (name, description) in demos {
        println!("  {} - {}", name.bold().cyan(), description.dimmed());
    }

    println!();

    match find_demo_dir_quietly() {
        Ok(demo_dir) => {
            println!("{}", "Script demos:".bold().green());
            let demo_files = scan_demo_files(&demo_dir)?;

            if demo_files.is_empty() {
                println!("  {}", "No script demos found".dimmed());
            } else {
                for demo in &demo_files {
                    println!(
                        "  {} - {}",
                        demo.name.bold().cyan(),
                        demo.description.dimmed()
                    );
                    if !demo.categories.is_empty() {
                        println!("    Categories: {}", demo.categories.join(", ").dimmed());
                    }
                }
            }
            println!("\nTotal script demos: {}", demo_files.len());
        }
        Err(_) => {
            println!("{}", "Script demos: Not available".yellow());
            println!("  Use 'thag_demo manage' to download demo directory");
        }
    }

    Ok(())
}

/// Manage demo directory (download, update, set location)
fn manage_demo_directory() -> Result<()> {
    println!("{}", "Demo Directory Management".bold().cyan());
    println!("{}", "═".repeat(30));

    // Check current status
    match discover_demo_dir() {
        Some(path) => {
            println!(
                "✅ Demo directory found at: {}",
                path.display().to_string().green()
            );

            let demo_files = scan_demo_files(&path)?;
            println!("📁 Contains {} demo scripts", demo_files.len());

            let options = vec![
                "Browse demos".to_string(),
                "Re-download/update demos".to_string(),
                "Show directory info".to_string(),
                "Exit".to_string(),
            ];

            match Select::new("What would you like to do?", options).prompt() {
                Ok(choice) => {
                    match choice.as_str() {
                        "Browse demos" => interactive_demo_browser(false)?,
                        "Re-download/update demos" => {
                            println!("Re-downloading demo directory...");
                            download_demo_dir()?;
                        }
                        "Show directory info" => {
                            println!("\nDemo Directory Information:");
                            println!("Location: {}", path.display());
                            println!("Files: {}", demo_files.len());

                            // Show first few files as examples
                            if !demo_files.is_empty() {
                                println!("\nSample files:");
                                for demo in demo_files.iter().take(5) {
                                    println!("  {} -> {}", demo.name, demo.path.display());
                                }
                                if demo_files.len() > 5 {
                                    println!("  ... and {} more", demo_files.len() - 5);
                                }
                            }

                            // Show some stats
                            let mut categories: std::collections::HashMap<String, usize> =
                                std::collections::HashMap::new();
                            for demo in &demo_files {
                                for cat in &demo.categories {
                                    *categories.entry(cat.clone()).or_insert(0) += 1;
                                }
                            }

                            if !categories.is_empty() {
                                println!("\nCategories:");
                                let mut cat_list: Vec<_> = categories.into_iter().collect();
                                cat_list.sort_by(|a, b| b.1.cmp(&a.1));
                                for (cat, count) in cat_list {
                                    println!("  {} ({})", cat, count);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    println!("❌ Interactive prompt failed: {}", e);
                    println!("💡 Demo directory is available at: {}", path.display());
                    println!(
                        "   Use 'thag_demo browse' or 'thag_demo list-scripts' to explore demos"
                    );
                }
            }
        }
        None => {
            println!("❌ Demo directory not found");
            println!("Would you like to download it?");

            match Confirm::new("Download demo directory?")
                .with_default(true)
                .prompt()
            {
                Ok(should_download) => {
                    if should_download {
                        download_demo_dir()?;
                    }
                }
                Err(e) => {
                    println!("❌ Interactive prompt failed: {}", e);
                    println!("💡 You can manually download with: thag thag_get_demo_dir");
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = DemoArgs::parse();

    println!(
        "{}",
        format!("🔥 thag_demo v{}", env!("CARGO_PKG_VERSION"))
            .bold()
            .cyan()
    );
    println!(
        "{}",
        "Interactive demos for thag_rs and thag_profiler".dimmed()
    );
    println!();

    if args.list {
        list_demos();
        return Ok(());
    }

    let Some(demo) = args.demo else {
        println!(
            "{}",
            "No demo specified. Use --list to see available demos or --help for usage.".yellow()
        );
        list_demos();
        return Ok(());
    };

    match demo {
        DemoCommand::Browse => interactive_demo_browser(args.verbose),
        DemoCommand::Manage => manage_demo_directory(),
        DemoCommand::ListScripts => list_all_demos(),
        _ => {
            // eprintln!("args.verbose={}", args.verbose);
            run_demo(demo, args.verbose)
        }
    }
}

fn list_demos() {
    println!("{}", "Available demos:".bold().green());
    println!();

    let demos = vec![
        ("basic-profiling", "Basic function timing and profiling"),
        (
            "memory-profiling",
            "Memory allocation tracking and analysis",
        ),
        ("async-profiling", "Profiling async functions and futures"),
        ("comparison", "Before/after performance comparison"),
        ("flamegraph", "Interactive flamegraph generation"),
        ("benchmark", "Full benchmark profiling with detailed output"),
        (
            "interactive-profiling",
            "Interactive profiling with embedded analysis",
        ),
        (
            "differential-comparison",
            "True differential comparison with before/after analysis",
        ),
    ];

    for (name, description) in demos {
        println!("  {} - {}", name.bold().cyan(), description.dimmed());
    }

    println!();
    println!("{}", "Interactive Commands:".bold().green());
    println!(
        "  {} - {}",
        "browse".bold().cyan(),
        "Interactive demo script browser".dimmed()
    );
    println!(
        "  {} - {}",
        "manage".bold().cyan(),
        "Manage demo directory (download/update)".dimmed()
    );
    println!(
        "  {} - {}",
        "list-scripts".bold().cyan(),
        "List all available demo scripts".dimmed()
    );

    println!();
    println!("{}", "Usage:".bold());
    println!("  thag_demo <demo_name>");
    println!("  thag_demo script <script_name>");
    println!("  thag_demo browse");
    println!("  thag_demo manage");
    println!("  thag_demo list-scripts");
    println!();
}

fn run_demo(demo: DemoCommand, verbose: bool) -> Result<()> {
    let demo_script = match demo {
        DemoCommand::BasicProfiling => include_str!("../demos/basic_profiling.rs"),
        DemoCommand::MemoryProfiling => include_str!("../demos/memory_profiling.rs"),
        DemoCommand::AsyncProfiling => include_str!("../demos/async_profiling.rs"),
        DemoCommand::Comparison => include_str!("../demos/comparison.rs"),
        DemoCommand::DifferentialComparison => include_str!("../demos/differential_comparison.rs"),
        DemoCommand::Flamegraph => include_str!("../demos/flamegraph.rs"),
        DemoCommand::Benchmark => include_str!("../demos/benchmark.rs"),
        DemoCommand::InteractiveProfiling => include_str!("../demos/interactive_profiling.rs"),
        DemoCommand::Script { name } => {
            return run_script_demo(&name, verbose);
        }
        DemoCommand::Browse | DemoCommand::Manage | DemoCommand::ListScripts => {
            unreachable!("These commands are handled earlier in main")
        }
    };

    let demo_name = match demo {
        DemoCommand::BasicProfiling => "basic_profiling",
        DemoCommand::MemoryProfiling => "memory_profiling",
        DemoCommand::AsyncProfiling => "async_profiling",
        DemoCommand::Comparison => "comparison",
        DemoCommand::DifferentialComparison => "differential_comparison",
        DemoCommand::Flamegraph => "flamegraph",
        DemoCommand::Benchmark => "benchmark",
        DemoCommand::InteractiveProfiling => "interactive_profiling",
        DemoCommand::Script { .. } => unreachable!(),
        DemoCommand::Browse | DemoCommand::Manage | DemoCommand::ListScripts => {
            unreachable!("These commands are handled earlier in main")
        }
    };

    println!("{}", format!("Running {demo_name} demo...").bold().green());
    // println!();

    // Create a temporary file for the demo script
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("thag_demo_{demo_name}.rs"));

    std::fs::write(&script_path, demo_script)?;

    // Set THAG_DEV_PATH for local development - point to thag_rs root
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let thag_rs_root = current_dir.parent().unwrap_or(&current_dir);
    std::env::set_var("THAG_DEV_PATH", thag_rs_root);

    // Configure CLI args for thag_rs
    let mut cli = create_demo_cli(&script_path, verbose);

    set_global_verbosity(if verbose { V::D } else { V::N })?;
    eprintln!("verbosity={}", get_verbosity());

    configure_log();

    // Execute the demo using thag_rs
    match execute(&mut cli) {
        Ok(()) => {
            println!();
            println!("{}", "✅ Demo completed successfully!".bold().green());
            print_demo_info(demo_name);
        }
        Err(e) => {
            eprintln!("{}", format!("❌ Demo failed: {e}").bold().red());
            process::exit(1);
        }
    }

    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn run_script_demo(script_name: &str, verbose: bool) -> Result<()> {
    let demo_dir = find_or_setup_demo_dir()?;
    let demo_path = demo_dir.join(format!("{script_name}.rs"));

    if !demo_path.exists() {
        eprintln!(
            "{}",
            format!("❌ Demo script '{script_name}' not found")
                .bold()
                .red()
        );
        eprintln!(
            "{}",
            "Use 'thag_demo list-scripts' to see available scripts".dimmed()
        );
        process::exit(1);
    }

    println!(
        "{}",
        format!("Running script demo: {script_name}").bold().green()
    );

    run_selected_demo(&demo_dir, script_name, verbose)?;

    println!();
    println!(
        "{}",
        "✅ Script demo completed successfully!".bold().green()
    );

    Ok(())
}

fn create_demo_cli(script_path: &Path, verbose: bool) -> Cli {
    Cli {
        script: Some(script_path.to_string_lossy().to_string()),
        features: None,
        args: Vec::new(),
        force: false,
        expression: None,
        repl: false,
        stdin: false,
        edit: false,
        filter: None,
        toml: None,
        begin: None,
        end: None,
        multimain: false,
        timings: false,
        verbose: if verbose { 2 } else { 0 },
        normal_verbosity: false,
        quiet: 0,
        generate: false,
        build: false,
        executable: false,
        check: false,
        expand: false,
        unquote: None,
        config: false,
        infer: None,
        cargo: false,
        test_only: false,
    }
}

fn print_demo_info(demo_name: &str) {
    println!();
    println!("{}", "📚 Learn more:".bold().blue());

    match demo_name {
        "basic_profiling" => {
            println!("  • Function profiling with #[profiled] attribute");
            println!("  • Time measurement and flamegraph generation");
        }
        "memory_profiling" => {
            println!("  • Memory allocation tracking");
            println!("  • Heap analysis and memory flamegraphs");
        }
        "async_profiling" => {
            println!("  • Profiling async functions and futures");
            println!("  • Tokio runtime integration");
        }
        "comparison" => {
            println!("  • Before/after performance comparison");
            println!("  • Side-by-side algorithm analysis");
        }
        "differential_comparison" => {
            println!("  • True differential profiling with before/after analysis");
            println!("  • Automated execution of inefficient vs efficient versions");
            println!("  • Interactive differential flamegraph generation");
        }
        "flamegraph" => {
            println!("  • Interactive flamegraph generation");
            println!("  • Visual performance analysis");
        }
        "benchmark" => {
            println!("  • Comprehensive benchmark profiling");
            println!("  • Detailed performance metrics");
        }
        "interactive_profiling" => {
            println!("  • Interactive profiling with embedded analysis");
            println!("  • Real-time performance insights and comparisons");
        }
        _ => {}
    }

    println!();
    println!("{}", "🔗 Resources:".bold().blue());
    println!("  • thag_profiler documentation: https://docs.rs/thag_profiler");
    println!("  • thag_rs repository: https://github.com/durbanlegend/thag_rs");
    println!("  • More examples: thag_demo --list");
    println!();
}
