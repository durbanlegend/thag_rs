//! `thag_demo` - Interactive demos for `thag_rs` and `thag_profiler`
//!
//! This crate provides a simple way to run profiling demos without installing `thag_rs`.
//! It acts as a lightweight facade over `thag_rs` functionality.

use anyhow::Result;
use clap::{Parser, Subcommand};
use regex;

use inquire::{set_global_render_config, Confirm, Select, Text};

use std::path::{Path, PathBuf};
use std::process;
use std::{env, fs, io};
use thag_rs::{
    builder::execute, configure_log, get_verbosity, re, set_global_verbosity, sprtln, svprtln,
    themed_inquire_config, Cli, Role, Styleable, StyledPrint, V,
};

pub mod visualization;

/// Metadata about a demo file
#[derive(Debug, Clone)]
struct DemoFile {
    name: String,
    path: PathBuf,
    description: String,
    categories: Vec<String>,
    sample_arguments: Option<String>,
    usage_example: Option<String>,
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
        /// Force rebuild even if script unchanged
        #[arg(short, long)]
        force: bool,
        /// Display timings
        #[arg(short, long)]
        timings: bool,
        /// Features to enable (comma separated)
        #[arg(long)]
        features: Option<String>,
        /// Environment variables to set (format: KEY=value, can be repeated)
        #[arg(short = 'E', long = "env")]
        env_vars: Vec<String>,
        /// Just generate, don't run
        #[arg(short, long)]
        generate: bool,
        /// Just build, don't run
        #[arg(short, long)]
        build: bool,
        /// Just check, don't run
        #[arg(short, long)]
        check: bool,
        /// Arguments to pass to the script (optionally use -- to separate from options)
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Interactive browser for demo scripts
    Browse,

    /// Manage demo directory (download, update, set location)
    Manage,

    /// List all available demo scripts
    ListScripts,
}

/// Find the thag_rs root directory reliably
/// Looks for the root Cargo.toml with workspace or thag_rs package
fn find_thag_rs_root() -> Result<PathBuf> {
    // Start from current directory or manifest directory
    let mut current =
        env::current_dir().unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    // Walk up the directory tree looking for thag_rs root
    for _ in 0..10 {
        // Look for Cargo.toml
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Read and check if it's the thag_rs workspace or package
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                // Check for workspace with thag_rs members or package named thag_rs
                if contents.contains("members") && contents.contains("thag_rs")
                    || contents.contains(r#"name = "thag_rs""#)
                {
                    return Ok(current);
                }
            }
        }

        // Go up one directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    // Fallback: use manifest dir's parent (works when running from thag_demo)
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot find thag_rs root"))?
        .to_path_buf())
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
                .map_or_else(|| PathBuf::from(""), PathBuf::from),
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

    "Demo directory not found in any standard location."
        .warning()
        .println();
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

/// Download demo directory using `thag_get_demo_dir`
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

    "Downloading demo directory...".info().println();

    // Call thag_get_demo_dir subprocess
    let output = std::process::Command::new("thag")
        .args(["thag_get_demo_dir"])
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
            "Failed to run thag_get_demo_dir. Make sure thag_rs is installed with tools feature."
                .error()
                .println();
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
    let mut sample_arguments = None;
    let mut usage_example = None;

    let lines: Vec<&str> = content.lines().collect();
    // eprintln!("lines={lines:#?}");
    let mut in_doc_comment = false;
    let mut in_block_doc_comment = false;

    for line in lines {
        let trimmed = line.trim();
        // eprintln!(
        //     r#"trimmed={trimmed}, trimmed.starts_with("///")={}"#,
        //     trimmed.starts_with("///")
        // );

        // Look for block doc comments (/** ... */)
        if trimmed.starts_with("/**") {
            in_block_doc_comment = true;
        } else if in_block_doc_comment && trimmed.starts_with("*/") {
            in_block_doc_comment = false;
            in_doc_comment = true;
        }
        // Look for doc comments (///)
        else if trimmed.starts_with("///") {
            in_doc_comment = true;
            let comment_text = trimmed.trim_start_matches("///").trim();
            if !comment_text.is_empty() && description.is_none() {
                description = Some(comment_text.to_string());
            }

            // Look for usage examples in doc comments
            if usage_example.is_none() {
                let first = extract_thag_commands(comment_text).iter().next().cloned();
                if let Some(ref cmd) = first {
                    if cmd.contains(' ') {
                        usage_example = first;
                        // eprintln!("Found command: {cmd}");
                    }
                }
            }
        } else if in_block_doc_comment {
            if !trimmed.starts_with("/**") {
                let comment_text = trimmed;
                if !comment_text.is_empty() && description.is_none() {
                    description = Some(comment_text.to_string());
                }
                // Look for usage examples in doc comments
                if usage_example.is_none() {
                    let first = extract_thag_commands(comment_text).iter().next().cloned();
                    if let Some(ref cmd) = first {
                        if cmd.contains(' ') {
                            usage_example = first;
                            // eprintln!("Found command: {cmd}");
                        }
                    }
                }
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
        // Look for sample arguments comment (//# Sample arguments:)
        else if trimmed.starts_with("//# Sample arguments:") {
            let args_text = trimmed.trim_start_matches("//# Sample arguments:").trim();
            sample_arguments = Some(args_text.to_string());
        }
        // Stop at first non-comment line
        else if in_doc_comment && !trimmed.starts_with("//") && !trimmed.is_empty() {
            break;
        }
    }

    let final_description = description.unwrap_or_else(|| "No description available".to_string());

    Ok(Some(DemoFile {
        name,
        path: path.to_path_buf(),
        description: final_description,
        categories,
        sample_arguments,
        usage_example,
    }))
}

/// Scan demo directory for .rs files and extract metadata
fn scan_demo_files(demo_dir: &Path) -> Result<Vec<DemoFile>> {
    let mut demos = Vec::new();

    for entry in fs::read_dir(demo_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "rs") {
            if let Some(demo) = extract_demo_metadata(&path)? {
                demos.push(demo);
            }
        }
    }

    demos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(demos)
}

/// Interactive demo browser similar to `thag_show_themes`
fn interactive_demo_browser(verbose: bool) -> Result<()> {
    use inquire::error::InquireResult;
    use inquire::list_option::ListOption;

    let demo_dir = find_or_setup_demo_dir()?;
    let demo_files = scan_demo_files(&demo_dir)?;

    if demo_files.is_empty() {
        "No demo files found in directory.".warning().println();
        return Ok(());
    }

    // Keep current directory as-is for relative paths
    // Users can reference demo scripts and their own files from CWD

    // Create demo options with descriptions
    let demo_options: Vec<String> = demo_files
        .iter()
        .map(|demo| {
            let cats = if demo.categories.is_empty() {
                String::new()
            } else {
                format!(" [{}]", demo.categories.join(", "))
            };
            let args_hint = if demo.sample_arguments.is_some() || demo.usage_example.is_some() {
                " 📝"
            } else {
                ""
            };
            format!("{} - {}{}{}", demo.name, demo.description, cats, args_hint)
        })
        .collect();

    // Clear screen initially
    print!("\x1b[2J\x1b[H");

    let mut cursor = 0_usize;

    loop {
        println!("\n🚀 Interactive Demo Browser");
        println!("{}", "═".repeat(80));
        println!("📚 {} demo scripts available", demo_files.len());
        println!("💡 Start typing to filter demos by name • 📝 = accepts arguments");
        sprtln!(
            Role::EMPH,
            "📂 Current shell directory: {}",
            env::current_dir()?.display().heading3()
        );
        println!("   💡 All file paths are relative to your current shell directory");
        println!("   📁 Demo scripts are in: {}", demo_dir.display().subtle());
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

                format!("Running demo script: {demo_name}").info().println();

                // Show additional info if available
                if let Some(demo_file) = demo_files.iter().find(|d| d.name == demo_name) {
                    if let Some(ref usage) = demo_file.usage_example {
                        println!("💡 Usage: {}", usage.subtle());
                    }
                    if let Some(ref args) = demo_file.sample_arguments {
                        let cleaned_sample = args.trim_matches('`').trim();
                        let display_args =
                            cleaned_sample.strip_prefix("-- ").unwrap_or(cleaned_sample);
                        println!("💡 Sample arguments: {}", display_args.subtle());
                        println!("   (don't type '--' in interactive mode)");
                    }
                }
                println!();

                match run_selected_demo(&demo_dir, demo_name, verbose, None, None) {
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

/// Options for running a demo
struct DemoOptions {
    force: bool,
    timings: bool,
    features: Option<String>,
    env_vars: Vec<String>,
    generate: bool,
    build: bool,
    check: bool,
}

/// Run a selected demo from the browser
fn run_selected_demo(
    demo_dir: &Path,
    demo_name: &str,
    verbose: bool,
    cli_args: Option<Vec<String>>,
    cli_options: Option<DemoOptions>,
) -> Result<()> {
    let demo_path = demo_dir.join(format!("{}.rs", demo_name));

    if !demo_path.exists() {
        return Err(anyhow::anyhow!(
            "Demo file not found: {}",
            demo_path.display()
        ));
    }

    // Extract demo metadata to get sample arguments
    let demo_metadata = extract_demo_metadata(&demo_path)?;
    let demo_file = demo_metadata
        .ok_or_else(|| anyhow::anyhow!("Could not extract metadata from demo file"))?;

    // Check if this is a profiling demo and suggest environment variable
    if (demo_file.name.contains("profile")
        || demo_file.categories.contains(&"profiling".to_string()))
        && env::var("THAG_PROFILER").is_err()
    {
        println!("\n📊 This appears to be a profiling demo.");
        println!("💡 For profiling output (.folded files), set: export THAG_PROFILER=both");
        println!("   Other options: THAG_PROFILER=time or THAG_PROFILER=memory");
        println!("   Without this, the demo runs but no profiling data is collected.");
    }

    // Collect options and arguments interactively if not provided via CLI
    let (options, args) = if let Some(opts) = cli_options {
        // CLI mode - use provided options and args
        let args = cli_args.unwrap_or_default();
        (opts, args)
    } else {
        // Interactive mode - collect in order: options, arguments, then env vars
        let mut opts = collect_demo_options(demo_name);
        let args = collect_demo_arguments(&demo_file);
        let env_vars = collect_env_vars(&demo_file);
        opts.env_vars = env_vars;
        (opts, args)
    };

    // Set THAG_DEV_PATH for local development - point to thag_rs root
    let thag_rs_root = find_thag_rs_root()
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    env::set_var("THAG_DEV_PATH", &thag_rs_root);

    // Set any custom environment variables
    for env_var in &options.env_vars {
        if let Some((key, value)) = env_var.split_once('=') {
            env::set_var(key, value);
            if verbose {
                println!("🌍 Set environment variable: {}={}", key, value);
            }
        } else {
            eprintln!(
                "⚠️  Warning: Invalid env var format '{}' (expected KEY=value)",
                env_var
            );
        }
    }

    let mut cli = create_demo_cli_with_args(&demo_path, verbose, args, Some(options));

    set_global_verbosity(if verbose { V::D } else { V::N });

    configure_log();

    execute(&mut cli)?;
    Ok(())
}

/// List all available demos (built-in and scripts)
fn list_all_demos() -> Result<()> {
    "Built-in demos:".info().println();
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
        sprtln!(Role::Subtle, "  {} - {}", name.info(), description.subtle());
    }

    println!();

    if let Ok(demo_dir) = find_demo_dir_quietly() {
        "Script demos:".info().println();
        let demo_files = scan_demo_files(&demo_dir)?;

        if demo_files.is_empty() {
            "No script demos found".subtle().println();
        } else {
            for demo in &demo_files {
                let args_hint = if demo.sample_arguments.is_some() || demo.usage_example.is_some() {
                    " 📝"
                } else {
                    ""
                };
                println!(
                    "  {} - {}{}",
                    demo.name.info(),
                    demo.description.subtle(),
                    args_hint
                );
                if !demo.categories.is_empty() {
                    println!("    Categories: {}", demo.categories.join(", ").subtle());
                }
                if let Some(ref args) = demo.sample_arguments {
                    println!("    Sample args: {}", args.subtle());
                }
            }
        }
        println!("\nTotal script demos: {}", demo_files.len());
    } else {
        "Script demos: Not available".warning().println();
        println!("  Use 'thag_demo manage' to download demo directory");
    }

    Ok(())
}

/// Manage demo directory (download, update, set location)
fn manage_demo_directory() -> Result<()> {
    "Demo Directory Management".info().println();
    println!("{}", "═".repeat(30));

    // Check current status
    if let Some(path) = discover_demo_dir() {
        println!("✅ Demo directory found at: {}", path.display().info());

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
                println!("   Use 'thag_demo browse' or 'thag_demo list-scripts' to explore demos");
            }
        }
    } else {
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

    Ok(())
}

fn main() -> Result<()> {
    set_global_render_config(themed_inquire_config());

    let args = DemoArgs::parse();

    svprtln!(
        Role::Heading1,
        V::QQ,
        "🔥 thag_demo v{}",
        env!("CARGO_PKG_VERSION")
    );
    println!("Interactive demos for thag_rs and thag_profiler");
    println!();

    if args.list {
        list_demos();
        return Ok(());
    }

    let Some(demo) = args.demo else {
        "No demo specified. Use --list to see available demos or --help for usage."
            .warning()
            .println();
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
    svprtln!(Role::Heading2, V::QQ, "Available demos:");
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
        svprtln!(Role::Heading3, V::QQ, "  {name} - {}", description.subtle());
    }

    println!();
    svprtln!(Role::Heading2, V::QQ, "Interactive Commands:");
    println!();
    svprtln!(
        Role::Heading3,
        V::QQ,
        "  browse - {}",
        "Interactive demo script browser".subtle()
    );
    svprtln!(
        Role::Heading3,
        V::QQ,
        "  manage - {}",
        "Manage demo directory (download/update)".subtle()
    );
    svprtln!(
        Role::Heading3,
        V::QQ,
        "  list-scripts - {}",
        "List all available demo scripts".subtle()
    );

    println!();
    "Usage:".emphasis().println();
    "  thag_demo <demo_name>".heading3().println();
    "  thag_demo script <script_name>".heading3().println();
    "  thag_demo browse".heading3().println();
    "  thag_demo manage".heading3().println();
    "  thag_demo list-scripts".heading3().println();
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
        DemoCommand::Script {
            name,
            force,
            timings,
            features,
            env_vars,
            generate,
            build,
            check,
            args,
        } => {
            return run_script_demo(
                &name,
                verbose,
                force,
                timings,
                features,
                env_vars,
                generate,
                build,
                check,
                Some(args),
            );
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

    format!("Running {demo_name} demo...").success().println();
    // println!();

    // Create a temporary file for the demo script
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("thag_demo_{demo_name}.rs"));

    std::fs::write(&script_path, demo_script)?;

    // Set THAG_DEV_PATH for local development - point to thag_rs root
    let thag_rs_root = find_thag_rs_root()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    std::env::set_var("THAG_DEV_PATH", &thag_rs_root);

    // Configure CLI args for thag_rs
    let mut cli = create_demo_cli(&script_path, verbose);

    set_global_verbosity(if verbose { V::D } else { V::N });
    eprintln!("verbosity={}", get_verbosity());

    configure_log();

    // Execute the demo using thag_rs
    match execute(&mut cli) {
        Ok(()) => {
            println!();
            "✅ Demo completed successfully!".success().println();
            print_demo_info(demo_name);
        }
        Err(e) => {
            format!("❌ Demo failed: {e}").error().println();
            process::exit(1);
        }
    }

    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn run_script_demo(
    script_name: &str,
    verbose: bool,
    force: bool,
    timings: bool,
    features: Option<String>,
    env_vars: Vec<String>,
    generate: bool,
    build: bool,
    check: bool,
    args: Option<Vec<String>>,
) -> Result<()> {
    let demo_dir = find_or_setup_demo_dir()?;
    let demo_path = demo_dir.join(format!("{script_name}.rs"));

    if !demo_path.exists() {
        eprintln!(
            "{}",
            format!("❌ Demo script '{script_name}' not found").error()
        );
        eprintln!(
            "{}",
            "Use 'thag_demo list-scripts' to see available scripts".subtle()
        );
        process::exit(1);
    }

    format!("Running script demo: {script_name}")
        .info()
        .println();

    if let Some(ref arg_list) = args {
        if !arg_list.is_empty() {
            format!("With arguments: {}", arg_list.join(" "))
                .subtle()
                .println();
            let cwd = env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string());
            format!("   (paths relative to: {})", cwd)
                .subtle()
                .println();
        }
    }

    let options = DemoOptions {
        force,
        timings,
        features,
        env_vars,
        generate,
        build,
        check,
    };

    run_selected_demo(&demo_dir, script_name, verbose, args, Some(options))?;

    println!();
    "✅ Script demo completed successfully!".success().println();

    Ok(())
}

fn create_demo_cli(script_path: &Path, verbose: bool) -> Cli {
    create_demo_cli_with_args(script_path, verbose, Vec::new(), None)
}

fn create_demo_cli_with_args(
    script_path: &Path,
    verbose: bool,
    args: Vec<String>,
    options: Option<DemoOptions>,
) -> Cli {
    let opts = options.unwrap_or(DemoOptions {
        force: false,
        timings: false,
        features: None,
        env_vars: Vec::new(),
        generate: false,
        build: false,
        check: false,
    });

    Cli {
        script: Some(script_path.to_string_lossy().to_string()),
        features: opts.features,
        args,
        force: opts.force,
        expression: None,
        repl: false,
        stdin: false,
        edit: false,
        filter: None,
        toml: None,
        begin: None,
        end: None,
        multimain: false,
        timings: opts.timings,
        verbose: if verbose { 2 } else { 0 },
        normal_verbosity: false,
        quiet: 0,
        generate: opts.generate,
        build: opts.build,
        executable: false,
        check: opts.check,
        expand: false,
        unquote: None,
        config: false,
        infer: None,
        cargo: false,
        test_only: false,
        clean: None,
    }
}

/// Collect thag options for a demo interactively
fn collect_demo_options(_demo_name: &str) -> DemoOptions {
    "\n⚙️  Thag options (press Enter to skip):"
        .emphasis()
        .println();

    let force = Confirm::new("Force rebuild?")
        .with_default(false)
        .with_help_message("Rebuild even if script unchanged (like -f flag)")
        .prompt()
        .unwrap_or(false);

    let timings = Confirm::new("Display timings?")
        .with_default(false)
        .with_help_message("Show execution timings (like -t flag)")
        .prompt()
        .unwrap_or(false);

    let features = Text::new("Features to enable (comma-separated):")
        .with_default("")
        .with_help_message("Optional features to enable when building")
        .prompt()
        .ok()
        .filter(|s| !s.trim().is_empty());

    let generate = Confirm::new("Just generate, don't run?")
        .with_default(false)
        .prompt()
        .unwrap_or(false);

    let build = if !generate {
        Confirm::new("Just build, don't run?")
            .with_default(false)
            .prompt()
            .unwrap_or(false)
    } else {
        false
    };

    let check = if !generate && !build {
        Confirm::new("Just check, don't run?")
            .with_default(false)
            .prompt()
            .unwrap_or(false)
    } else {
        false
    };

    DemoOptions {
        force,
        timings,
        features,
        env_vars: Vec::new(), // Will be collected after showing demo info
        generate,
        build,
        check,
    }
}

/// Collect arguments for a demo if needed
fn collect_demo_arguments(demo_file: &DemoFile) -> Vec<String> {
    // Check if this demo needs arguments by looking for usage patterns
    let needs_args = demo_file.sample_arguments.is_some()
        || demo_file
            .usage_example
            .as_ref()
            .is_some_and(|ex| ex.contains("--"))
        || demo_file.description.contains("Usage:")
        || demo_file.description.contains("E.g.");

    if !needs_args {
        return Vec::new();
    }

    "\n📝 This demo accepts command-line arguments."
        .info()
        .println();

    if let Some(ref sample_args) = demo_file.sample_arguments {
        // Strip the -- prefix from sample args for interactive display
        let cleaned_sample = sample_args.trim_matches('`').trim();
        let display_args = cleaned_sample.strip_prefix("-- ").unwrap_or(cleaned_sample);
        println!("💡 Sample arguments: {}", display_args.emphasis());
        "   (don't type '--' in interactive mode)"
            .warning()
            .println();
    }

    if let Some(ref usage) = demo_file.usage_example {
        println!("💡 Usage example: {}", usage.commentary());
    }

    // Show environment variable suggestions if any
    if demo_file.name.contains("profile") || demo_file.categories.contains(&"profiling".to_string())
    {
        println!("\n💡 Suggested environment variables:");
        println!("   THAG_PROFILER=both    (enable profiling output)");
        println!("   THAG_PROFILER=time    (time profiling only)");
        println!("   THAG_PROFILER=memory  (memory profiling only)");
    }

    let cwd = env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".to_string());

    match Text::new(&format!(
        "Enter arguments (paths relative to {}):",
        cwd.subtle()
    ))
    .with_default("")
    .with_help_message(
        "Arguments for the demo script. File paths are relative to your shell's current directory",
    )
    .prompt()
    {
        Ok(input) => {
            if input.trim().is_empty() {
                Vec::new()
            } else {
                // Strip leading -- if user typed it (they shouldn't in interactive mode)
                let cleaned_input = input.trim().strip_prefix("-- ").unwrap_or(input.trim());

                // Simple argument parsing - split by spaces but preserve quoted strings
                shell_words::split(cleaned_input).unwrap_or_else(|_| {
                    // Fallback to simple split if shell_words fails
                    cleaned_input.split_whitespace().map(String::from).collect()
                })
            }
        }
        Err(e) => {
            println!("❌ Interactive prompt failed: {}", e);
            // Try using sample arguments as fallback
            if let Some(ref sample_args) = demo_file.sample_arguments {
                // Clean up the arguments by removing backticks and extracting content after --
                let cleaned = sample_args.trim_matches('`').trim();
                let args_str = cleaned.strip_prefix("-- ").map_or_else(
                    || {
                        if cleaned.contains("-- ") {
                            cleaned.split("-- ").nth(1).unwrap_or(cleaned)
                        } else {
                            cleaned
                        }
                    },
                    |stripped| stripped,
                );
                println!("💡 Using sample arguments as fallback: {}", args_str);
                return shell_words::split(args_str)
                    .unwrap_or_else(|_| args_str.split_whitespace().map(String::from).collect());
            }
            println!("💡 Running demo without arguments");
            Vec::new()
        }
    }
}

/// Collect environment variables for a demo interactively
fn collect_env_vars(demo_file: &DemoFile) -> Vec<String> {
    // Show suggestions based on demo type
    if demo_file.name.contains("profile") || demo_file.categories.contains(&"profiling".to_string())
    {
        println!("\n🌍 Environment Variables (press Enter to skip):");
        println!("💡 Suggested for profiling demos: THAG_PROFILER=both");
    } else {
        println!("\n🌍 Environment Variables (optional, press Enter to skip):");
    }

    match Text::new(
        "Enter environment variables (format: KEY=value, separate multiple with spaces):",
    )
    .with_default("")
    .with_help_message("Example: THAG_PROFILER=both RUST_LOG=debug")
    .prompt()
    {
        Ok(input) => {
            if input.trim().is_empty() {
                Vec::new()
            } else {
                // Split by spaces, preserving quoted strings if needed
                shell_words::split(&input)
                    .unwrap_or_else(|_| input.split_whitespace().map(String::from).collect())
                    .into_iter()
                    .filter(|s| s.contains('='))
                    .collect()
            }
        }
        Err(_) => Vec::new(),
    }
}

fn print_demo_info(demo_name: &str) {
    println!();
    "📚 Learn more:".info().println();

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
    "🔗 Resources:".info().println();
    println!("  • thag_profiler documentation: https://docs.rs/thag_profiler");
    println!("  • thag_rs repository: https://github.com/durbanlegend/thag_rs");
    println!("  • More examples: thag_demo --list");
    println!();
}

/// Extract full bash-style invocations involving thag or thag_*
///
/// Examples matched:
/// - `thag demo/hello.rs`
/// - `FOO=bar BAR=baz thag -x`
/// - `cat demo.rs | thag_url --debug`
/// - `MULTI=1 thag -f arg1 arg2`
pub fn extract_thag_commands(text: &str) -> Vec<String> {
    let re = re!(r#"(?mx)
        (?P<cmd>                                  # capture the entire invocation
            (?:\b\w+=\S+\s+)*                     # optional env vars
            (?:[^\s|]+?\s*\|\s*)?                 # optional piped input
            thag(?:_[a-z0-9]+)?                   # thag or thag_something
            (?:\s+[^\s`";)\]]+)*                  # args and options
        )
        "#,);

    re.captures_iter(text)
        .map(|c| c["cmd"].trim().to_string())
        .collect()
}
