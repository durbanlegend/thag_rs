/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling"] }
chrono = { version = "0.4", features = ["serde"] }
inferno = "0.11"

[profile.release]
debug = true
strip = false
*/

/// Memory profiling demo - shows how to use thag_profiler for memory allocation tracking
/// This demo demonstrates memory profiling features of thag_profiler
//# Purpose: Demonstrate memory allocation tracking with thag_profiler
//# Categories: profiling, demo, memory
use std::collections::HashMap;
use std::io::Write;
use thag_demo_proc_macros::timing;
use thag_profiler::{enable_profiling, enhance_svg_accessibility, profiled};

// Memory-specific visualization inline module

#[timing]
#[profiled(mem_summary)]
fn allocate_vectors() -> Vec<Vec<u64>> {
    let mut outer = Vec::new();

    for i in 0..100 {
        let mut inner = Vec::with_capacity(1000);
        for j in 0..1000 {
            inner.push(i * 1000 + j);
        }
        outer.push(inner);
    }

    outer
}

#[timing]
#[profiled(mem_detail)]
fn process_strings_detail_profile() -> HashMap<String, usize> {
    let mut map = HashMap::new();

    for i in 0..1_000 {
        let key = format!("key_{}", i);
        let value = format!("value_{}_with_some_longer_content", i);
        map.insert(key, value.len());
    }

    map
}

#[timing]
#[profiled(mem_summary)]
fn memory_intensive_computation() -> Vec<String> {
    let mut results = Vec::new();

    // Simulate processing that creates many temporary allocations
    for i in 0..100 {
        let mut temp = String::new();
        for j in 0..100 {
            temp.push_str(&format!("item_{}_{} ", i, j));
        }

        // Keep only every 10th result to show deallocation
        if i % 10 == 0 {
            results.push(temp);
        }
        // temp is dropped here for other iterations
    }

    results
}

#[timing]
#[profiled(mem_summary)]
fn nested_allocations() {
    println!("Starting nested allocations...");

    let vectors = allocate_vectors();
    println!("Allocated {} vectors", vectors.len());

    let map = process_strings_detail_profile();
    println!("Created map with {} entries", map.len());

    let results = memory_intensive_computation();
    println!("Generated {} results", results.len());

    // All data structures will be deallocated when this function ends
}

#[enable_profiling(memory)]
fn main() {
    println!("🧠 Memory Profiling Demo");
    println!("========================");
    println!();

    println!("Running memory-intensive operations with profiling...");
    nested_allocations();

    println!();
    println!("✅ Demo completed!");
    println!("📊 Check the generated memory flamegraph files for allocation analysis.");
    println!("🔍 Use 'thag_profile' command to analyze memory usage patterns.");
    println!("💡 Notice the difference between mem_summary and mem_detail profiling.");

    // Add interactive memory visualization
    if let Err(e) = show_interactive_memory_visualization() {
        eprintln!("⚠️ Could not show interactive memory visualization: {}", e);
    }
}

fn show_interactive_memory_visualization() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("🎯 Would you like to view an interactive memory flamegraph?");
    println!(
        "This will generate a visual memory allocation flamechart and open it in your browser."
    );
    print!("Enter 'y' for yes, or any other key to skip: ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        if input.trim().to_lowercase() == "y" {
            println!();
            println!("🔥 Generating interactive memory flamechart...");

            // Try to load and display the memory profile data
            match load_and_show_memory_profile() {
                Ok(()) => {
                    println!("✅ Memory flamechart generation completed!");
                }
                Err(e) => {
                    println!("⚠️  Could not generate memory flamechart: {}", e);
                    println!(
                        "💡 Make sure the demo completed successfully and generated memory profile files."
                    );
                }
            }
        }
    }

    Ok(())
}

fn load_and_show_memory_profile() -> Result<(), Box<dyn std::error::Error>> {
    // Wait a moment for profile files to be written
    std::thread::sleep(std::time::Duration::from_millis(500));

    let current_dir = std::env::current_dir()?;
    let mut memory_files = Vec::new();

    for entry in std::fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("thag_demo_memory_profiling")
                && name.ends_with(".folded")
                && name.contains("memory")
            {
                memory_files.push(path);
            }
        }
    }

    if memory_files.is_empty() {
        return Err("No memory profile files found".into());
    }

    // Sort by modification time, most recent first
    memory_files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    let memory_file = &memory_files[0];
    show_memory_analysis(memory_file)?;
    generate_memory_flamechart(memory_file)?;

    Ok(())
}

fn show_memory_analysis(file_path: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut function_allocations: std::collections::HashMap<String, u128> =
        std::collections::HashMap::new();
    let mut total_bytes_allocated = 0u128;

    // Parse folded stack format for memory data
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let stack = parts[0];
        let bytes_str = parts[1];

        if let Ok(bytes) = bytes_str.parse::<u128>() {
            total_bytes_allocated += bytes;

            // Extract function names from the stack
            let functions: Vec<&str> = stack.split(';').collect();
            for func_name in functions {
                let clean_name = clean_memory_function_name(func_name);
                *function_allocations.entry(clean_name).or_insert(0) += bytes;
            }
        }
    }

    // Create and display memory analysis
    let mut functions: Vec<_> = function_allocations.into_iter().collect();
    functions.sort_by(|a, b| b.1.cmp(&a.1));

    println!("📊 Memory Profile Analysis Results");
    println!("═══════════════════════════════════");
    println!(
        "Total Memory Allocated: {:.2} KB",
        total_bytes_allocated as f64 / 1024.0
    );
    println!();

    println!("🏆 Top Functions by Memory Allocation:");
    println!("──────────────────────────────────────");

    for (i, (name, bytes)) in functions.iter().enumerate().take(10) {
        let percentage = (*bytes as f64 / total_bytes_allocated as f64) * 100.0;
        let size_kb = *bytes as f64 / 1024.0;

        let icon = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "🏅",
        };

        println!(
            "{} {}. {} - {:.2} KB ({:.1}%)",
            icon,
            i + 1,
            name,
            size_kb,
            percentage
        );
    }

    println!();
    show_memory_insights(&functions);

    Ok(())
}

fn clean_memory_function_name(name: &str) -> String {
    let clean = name.split("::").last().unwrap_or(name);
    let clean = clean.split('<').next().unwrap_or(clean);
    let clean = clean.split('(').next().unwrap_or(clean);

    match clean {
        s if s.starts_with("thag_demo_") => s.strip_prefix("thag_demo_").unwrap_or(s).to_string(),
        s if s.contains("allocate_vectors") => "allocate_vectors".to_string(),
        s if s.contains("process_strings") => "process_strings_detail".to_string(),
        s if s.contains("memory_intensive") => "memory_intensive_computation".to_string(),
        s if s.contains("nested_allocations") => "nested_allocations".to_string(),
        s => s.to_string(),
    }
}

fn show_memory_insights(functions: &[(String, u128)]) {
    println!("💡 Memory Allocation Insights:");
    println!("─────────────────────────────");

    if functions.len() >= 2 {
        let largest = &functions[0];
        let smallest = &functions[functions.len() - 1];

        if smallest.1 > 0 {
            let ratio = largest.1 as f64 / smallest.1 as f64;
            println!(
                "🐘 Largest allocator: {} ({:.2} KB)",
                largest.0,
                largest.1 as f64 / 1024.0
            );
            println!(
                "🐁 Smallest allocator: {} ({:.2} KB)",
                smallest.0,
                smallest.1 as f64 / 1024.0
            );
            println!("📏 Memory usage difference: {:.1}x", ratio);
        }
    }

    // Look for memory-specific patterns
    let has_vectors = functions.iter().any(|(name, _)| name.contains("vector"));
    let has_strings = functions.iter().any(|(name, _)| name.contains("string"));
    let has_hash_map = functions
        .iter()
        .any(|(name, _)| name.contains("HashMap") || name.contains("map"));

    if has_vectors {
        println!("📋 Tip: Vector allocations detected - consider pre-allocating with capacity!");
    }

    if has_strings {
        println!("📝 Tip: String operations found - consider using String::with_capacity() for better performance!");
    }

    if has_hash_map {
        println!("🗺️  Tip: HashMap usage detected - consider pre-sizing for known data sizes!");
    }

    println!();
}

fn generate_memory_flamechart(
    profile_file: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 Generating Interactive Memory Flamechart...");

    let content = std::fs::read_to_string(profile_file)?;
    let stacks: Vec<String> = content.lines().map(|line| line.to_string()).collect();

    if stacks.is_empty() {
        println!("⚠️  No memory profile data found in file");
        return Ok(());
    }

    // Create memory flamechart options with memory-specific color scheme
    let mut opts = inferno::flamegraph::Options::default();
    opts.title = "Memory Profiling Demo - Memory Allocation Flamechart".to_string();
    opts.subtitle = Some(format!(
        "Generated: {} | Hover over and click on the bars to explore memory allocations, or use Search ↗️",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    opts.colors =
        inferno::flamegraph::Palette::Basic(inferno::flamegraph::color::BasicPalette::Mem);
    opts.count_name = "bytes".to_string();
    opts.min_width = 0.0;
    opts.flame_chart = true; // Use aggregated flamechart

    // Generate memory flamechart
    let svg_path = "memory_profiling_flamechart.svg";
    let output = std::fs::File::create(svg_path)?;

    inferno::flamegraph::from_lines(&mut opts, stacks.iter().rev().map(String::as_str), output)?;

    thag_profiler::enhance_svg_accessibility(svg_path)?;

    println!("✅ Memory flamechart generated: {}", svg_path);

    // Open in browser
    if let Err(e) = open_in_browser(svg_path) {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("💡 You can manually open: {}", svg_path);
    } else {
        println!("🌐 Memory flamechart opened in your default browser!");
        println!("🔍 Hover over and click on the bars to explore memory allocation patterns");
        println!("📊 Bar width = bytes allocated, height = call stack depth");
        println!("🎨 Color scheme optimized for memory visualization");
        println!("💡 Notice the memory allocation patterns in different functions!");
    }

    Ok(())
}

fn open_in_browser(svg_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let full_path = std::env::current_dir()?.join(svg_path);
    let url = format!("file://{}", full_path.display());

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &url])
            .spawn()?;
    }

    Ok(())
}
