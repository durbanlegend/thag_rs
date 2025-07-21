/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling", "demo"] }

[profile.release]
debug = true
strip = false
*/

/// Memory profiling demo - shows how to use thag_profiler for memory allocation tracking
/// This demo demonstrates memory profiling features of thag_profiler
//# Purpose: Demonstrate memory allocation tracking with thag_profiler
//# Categories: profiling, demo, memory
use std::collections::HashMap;
// use std::error::Error;
use thag_profiler::{timing, visualization, AnalysisType, ProfileType};

async fn run_analysis() {
    // Interactive visualization: must run AFTER function with `enable_profiling` profiling attribute,
    // because profile output is only available after that function completes.
    if let Err(e) = visualization::show_interactive_prompt(
        "benchmark",
        &ProfileType::Memory,
        &AnalysisType::Flamegraph,
    )
    .await
    {
        eprintln!("⚠️ Could not show interactive memory visualization: {e}");
    }
}

fn main() {
    println!("🧠 Memory Profiling Visualization Demo");
    println!("{}", "═".repeat(37));
    println!();

    smol::block_on(run_analysis());

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated memory flamegraph files for allocation analysis.");
    println!("🔍 Use 'thag_profile' command to analyze memory usage patterns.");
    println!("💡 Notice the difference between mem_summary and mem_detail profiling.");

    // Ok(())
}
