//! thag_demo - Interactive demos for thag_rs and thag_profiler
//!
//! This crate provides a simple way to run profiling demos without installing thag_rs.
//! It acts as a lightweight facade over thag_rs functionality.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::process;
use thag_rs::{builder::execute, Cli};

/// Demo runner for thag_rs and thag_profiler examples
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

    /// Interactive flamegraph demo
    Flamegraph,

    /// Full benchmark profiling demo
    Benchmark,

    /// Run a specific demo script by name
    Script {
        /// Name of the demo script to run
        name: String,
    },
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

    let demo = match args.demo {
        Some(demo) => demo,
        None => {
            println!(
                "{}",
                "No demo specified. Use --list to see available demos or --help for usage."
                    .yellow()
            );
            list_demos();
            return Ok(());
        }
    };

    run_demo(demo, args.verbose)
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
    ];

    for (name, description) in demos {
        println!("  {} - {}", name.bold().cyan(), description.dimmed());
    }

    println!();
    println!("{}", "Usage:".bold());
    println!("  thag_demo <demo_name>");
    println!("  thag_demo script <script_name>");
    println!();
}

fn run_demo(demo: DemoCommand, verbose: bool) -> Result<()> {
    let demo_script = match demo {
        DemoCommand::BasicProfiling => include_str!("../demos/basic_profiling.rs"),
        DemoCommand::MemoryProfiling => include_str!("../demos/memory_profiling.rs"),
        DemoCommand::AsyncProfiling => include_str!("../demos/async_profiling.rs"),
        DemoCommand::Comparison => include_str!("../demos/comparison.rs"),
        DemoCommand::Flamegraph => include_str!("../demos/flamegraph.rs"),
        DemoCommand::Benchmark => include_str!("../demos/benchmark.rs"),
        DemoCommand::Script { name } => {
            return run_script_demo(&name, verbose);
        }
    };

    let demo_name = match demo {
        DemoCommand::BasicProfiling => "basic_profiling",
        DemoCommand::MemoryProfiling => "memory_profiling",
        DemoCommand::AsyncProfiling => "async_profiling",
        DemoCommand::Comparison => "comparison",
        DemoCommand::Flamegraph => "flamegraph",
        DemoCommand::Benchmark => "benchmark",
        DemoCommand::Script { .. } => unreachable!(),
    };

    println!(
        "{}",
        format!("Running {} demo...", demo_name).bold().green()
    );
    println!();

    // Create a temporary file for the demo script
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("thag_demo_{}.rs", demo_name));

    std::fs::write(&script_path, demo_script)?;

    // Set THAG_DEV_PATH for local development - point to thag_rs root
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let thag_rs_root = current_dir.parent().unwrap_or(&current_dir);
    std::env::set_var("THAG_DEV_PATH", thag_rs_root);

    // Configure CLI args for thag_rs
    let mut cli = create_demo_cli(script_path, verbose);

    // Execute the demo using thag_rs
    match execute(&mut cli) {
        Ok(()) => {
            println!();
            println!("{}", "✅ Demo completed successfully!".bold().green());
            print_demo_info(demo_name);
        }
        Err(e) => {
            eprintln!("{}", format!("❌ Demo failed: {}", e).bold().red());
            process::exit(1);
        }
    }

    Ok(())
}

fn run_script_demo(script_name: &str, verbose: bool) -> Result<()> {
    // Look for the script in the parent thag_rs demo directory
    let demo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("demo")
        .join(format!("{}.rs", script_name));

    if !demo_path.exists() {
        eprintln!(
            "{}",
            format!("❌ Demo script '{}' not found", script_name)
                .bold()
                .red()
        );
        eprintln!(
            "{}",
            "Available scripts can be found in the thag_rs/demo directory".dimmed()
        );
        process::exit(1);
    }

    println!(
        "{}",
        format!("Running script demo: {}", script_name)
            .bold()
            .green()
    );
    println!();

    // Set THAG_DEV_PATH for local development - point to thag_rs root
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let thag_rs_root = current_dir.parent().unwrap_or(&current_dir);
    std::env::set_var("THAG_DEV_PATH", thag_rs_root);

    let mut cli = create_demo_cli(demo_path, verbose);

    match execute(&mut cli) {
        Ok(()) => {
            println!();
            println!(
                "{}",
                "✅ Script demo completed successfully!".bold().green()
            );
        }
        Err(e) => {
            eprintln!("{}", format!("❌ Script demo failed: {}", e).bold().red());
            process::exit(1);
        }
    }

    Ok(())
}

fn create_demo_cli(script_path: PathBuf, verbose: bool) -> Cli {
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
        verbose: if verbose { 2 } else { 1 },
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
            println!("  • Differential flamegraph analysis");
        }
        "flamegraph" => {
            println!("  • Interactive flamegraph generation");
            println!("  • Visual performance analysis");
        }
        "benchmark" => {
            println!("  • Comprehensive benchmark profiling");
            println!("  • Detailed performance metrics");
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
