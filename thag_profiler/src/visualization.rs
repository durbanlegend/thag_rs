//! Demo visualization utilities for `thag_profiler`
//!
//! This module provides reusable visualization functions for analyzing profiling data,
//! including flamegraphs, flamecharts, and profile analysis. It's designed to be used
//! in demo scripts and examples.
//!
//! This module is only available when the `demo` feature is enabled.

use crate::enhance_svg_accessibility;
use chrono::Local;
use inferno::flamegraph::{self, color::MultiPalette, Options, Palette};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::string::ToString;

/// Configuration for visualization generation
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// Title for the visualization
    pub title: String,
    /// Subtitle (optional)
    pub subtitle: Option<String>,
    /// Color palette to use
    pub palette: Palette,
    /// Count name (e.g., "μs", "bytes")
    pub count_name: String,
    /// Minimum bar width to display
    pub min_width: f64,
    /// Whether to generate flamechart (timeline) vs flamegraph (aggregated)
    pub flame_chart: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            title: "Profiling Analysis".to_string(),
            subtitle: Some(format!(
                "Generated: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            )),
            palette: Palette::Multi(MultiPalette::Rust),
            count_name: "μs".to_string(),
            min_width: 0.0,
            flame_chart: true,
        }
    }
}

/// Profile analysis results
#[derive(Debug, Clone)]
pub struct ProfileAnalysis {
    /// Total duration of the profiling session in microseconds
    pub total_duration_us: u128,
    /// Function names paired with their execution times in microseconds
    pub function_times: Vec<(String, u128)>,
    /// Top functions with name, execution time in microseconds, and percentage of total time
    pub top_functions: Vec<(String, u128, f64)>,
    /// Generated insights and observations about the profiling data
    pub insights: Vec<String>,
}

/// Generate a flamegraph SVG from folded stack data
///
/// # Errors
///
/// Returns an error if:
/// - The stack data is empty
/// - File creation or writing fails
/// - Flamegraph generation fails
pub fn generate_flamegraph_svg(
    stacks: &[String],
    output_path: &str,
    config: VisualizationConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if stacks.is_empty() {
        return Err("No profile data found".into());
    }

    // Create flamegraph options
    let mut opts = Options::default();
    opts.title = config.title;
    opts.subtitle = config.subtitle;
    opts.colors = config.palette;
    opts.count_name = config.count_name;
    opts.min_width = config.min_width;
    opts.flame_chart = config.flame_chart;

    // Generate flamegraph
    let output = std::fs::File::create(output_path)?;

    flamegraph::from_lines(&mut opts, stacks.iter().rev().map(String::as_str), output)?;

    // Enhance accessibility
    enhance_svg_accessibility(output_path)?;

    Ok(())
}

/// Generate a flamegraph from a single folded file
///
/// # Errors
///
/// Returns an error if:
/// - The folded file cannot be read
/// - The file is empty or contains no valid stack data
/// - Flamegraph SVG generation fails
pub fn generate_flamegraph_from_file(
    folded_file: &std::path::Path,
    output_path: &str,
    config: VisualizationConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(folded_file)?;
    // eprintln!("content={content}");
    let stacks: Vec<String> = content.lines().map(ToString::to_string).collect();

    generate_flamegraph_svg(&stacks, output_path, config)
}

/// Analyze a profile file and extract function timing data
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file contains invalid profile data format
/// - Duration parsing fails for any line
#[allow(clippy::cast_precision_loss)]
pub fn analyze_profile(file_path: &PathBuf) -> Result<ProfileAnalysis, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut function_times: HashMap<String, u128> = HashMap::new();
    let mut total_duration_us = 0u128;

    // Parse folded stack format
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let stack = parts[0];
        let time_str = parts[1];

        if let Ok(time_us) = time_str.parse::<u128>() {
            total_duration_us += time_us;

            // Extract function names from the stack
            let functions: Vec<&str> = stack.split(';').collect();
            for func_name in functions {
                let clean_name = clean_function_name(func_name);
                *function_times.entry(clean_name).or_insert(0) += time_us;
            }
        }
    }

    let mut functions: Vec<_> = function_times.into_iter().collect();
    functions.sort_by(|a, b| b.1.cmp(&a.1));

    let top_functions: Vec<_> = functions
        .iter()
        .take(10)
        .map(|(name, time)| {
            let percentage = (*time as f64 / total_duration_us as f64) * 100.0;
            (name.clone(), *time, percentage)
        })
        .collect();

    let insights = generate_insights(&functions, total_duration_us);

    Ok(ProfileAnalysis {
        total_duration_us,
        function_times: functions,
        top_functions,
        insights,
    })
}

/// Display profile analysis results
#[allow(clippy::cast_precision_loss)]
pub fn display_profile_analysis(analysis: &ProfileAnalysis) {
    println!("📊 Profile Analysis Results");
    println!("═══════════════════════════");
    println!(
        "Total Duration: {:.3}ms",
        analysis.total_duration_us as f64 / 1000.0
    );
    println!();

    println!("🏆 Top Functions by Execution Time:");
    println!("────────────────────────────────────");

    for (i, (name, time_us, percentage)) in analysis.top_functions.iter().enumerate() {
        let time_ms = *time_us as f64 / 1000.0;

        let icon = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "🏅",
        };

        println!(
            "{} {}. {} - {:.3}ms ({:.1}%)",
            icon,
            i + 1,
            name,
            time_ms,
            percentage
        );
    }

    println!();
    println!("💡 Performance Insights:");
    println!("────────────────────────");
    for insight in &analysis.insights {
        println!("{}", insight);
    }
    println!();
}

/// Find the most recent profile files matching a pattern
///
/// # Errors
///
/// Returns an error if:
/// - Cannot get current working directory
/// - Cannot read directory contents
/// - File metadata cannot be accessed
pub fn find_latest_profile_files(
    pattern: &str,
    count: usize,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let mut files = Vec::new();

    for entry in std::fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains(pattern) && name.ends_with(".folded") {
                files.push(path);
            }
        }
    }

    // Sort by modification time, most recent first
    files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    files.truncate(count);
    Ok(files)
}

/// Open a file in the default browser
///
/// # Errors
///
/// Returns an error if:
/// - Cannot get current working directory
/// - System command to open browser fails
/// - Platform is not supported (Windows/macOS/Linux)
pub fn open_in_browser(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let full_path = std::env::current_dir()?.join(file_path);
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

/// Show interactive visualization prompt
///
/// # Errors
///
/// Returns an error if:
/// - Cannot read user input from stdin
/// - System command to open browser fails
pub fn show_interactive_prompt(
    demo_name: &str,
    analysis_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!(
        "🎯 Would you like to view an interactive {}?",
        analysis_type
    );
    println!("This will generate a visual flamechart and open it in your browser.");

    print!("Enter 'y' for yes, or any other key to skip: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() && input.trim().to_lowercase() == "y" {
        println!();
        println!("🔥 Generating interactive {}...", analysis_type);
        generate_and_show_visualization(demo_name)?;
    }

    Ok(())
}

/// Generate and show visualization for a demo
///
/// # Errors
///
/// Returns an error if:
/// - Profile files cannot be found or read
/// - Flamegraph generation fails
/// - Browser cannot be opened to display results
pub fn generate_and_show_visualization(demo_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Wait a moment for profile files to be written
    std::thread::sleep(std::time::Duration::from_millis(500));

    let files = find_latest_profile_files(&format!("thag_demo_{}", demo_name), 1)?;

    if files.is_empty() {
        println!("⚠️  No profile files found");
        println!("💡 Make sure the demo completed successfully and generated profile files.");
        return Ok(());
    }

    // Show profile analysis first
    let analysis = analyze_profile(&files[0])?;
    display_profile_analysis(&analysis);

    let config = VisualizationConfig {
        title: format!(
            "{} Demo - Performance Flamechart",
            demo_name.to_title_case()
        ),
        subtitle: Some(format!(
            "Generated: {} | Hover over and click on the bars to explore the function call hierarchy, or use Search ↗️",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        )),
        ..Default::default()
    };

    let output_path = format!("{}_flamechart.svg", demo_name);

    generate_flamegraph_from_file(&files[0], &output_path, config)?;

    println!("✅ Flamechart generated: {}", output_path);

    if let Err(e) = open_in_browser(&output_path) {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("💡 You can manually open: {}", output_path);
    } else {
        println!("🌐 Flamechart opened in your default browser!");
        println!("🔍 Hover over and click on the bars to explore the performance visualization");
        println!("📊 Function width = time spent, height = call stack depth");
    }

    Ok(())
}

/// Generate memory-specific visualization
///
/// # Errors
///
/// Returns an error if:
/// - Memory profile files cannot be found or read
/// - Flamegraph generation fails
/// - Browser cannot be opened to display results
pub fn generate_and_show_memory_visualization(
    demo_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Wait a moment for profile files to be written
    std::thread::sleep(std::time::Duration::from_millis(500));

    let files = find_latest_profile_files(&format!("thag_demo_{}", demo_name), 1)?;

    if files.is_empty() {
        println!("⚠️  No memory profile files found");
        println!("💡 Make sure the demo completed successfully and generated profile files.");
        return Ok(());
    }

    // Show memory profile analysis first
    let analysis = analyze_profile(&files[0])?;
    display_memory_analysis(&analysis);

    let config = VisualizationConfig {
        title: format!(
            "{} Demo - Memory Allocation Flamechart",
            demo_name.to_title_case()
        ),
        subtitle: Some(format!(
            "Generated: {} | Hover over and click on the bars to explore memory allocations, or use Search ↗️",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        )),
        palette: inferno::flamegraph::Palette::Basic(inferno::flamegraph::color::BasicPalette::Mem),
        count_name: "bytes".to_string(),
        ..Default::default()
    };

    let output_path = format!("{}_memory_flamechart.svg", demo_name);

    generate_flamegraph_from_file(&files[0], &output_path, config)?;

    println!("✅ Memory flamechart generated: {}", output_path);

    if let Err(e) = open_in_browser(&output_path) {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("💡 You can manually open: {}", output_path);
    } else {
        println!("🌐 Memory flamechart opened in your default browser!");
        println!("🔍 Hover over and click on the bars to explore memory allocation patterns");
        println!("📊 Bar width = bytes allocated, height = call stack depth");
        println!("🎨 Color scheme optimized for memory visualization");
    }

    Ok(())
}

/// Display memory-specific analysis
#[allow(clippy::cast_precision_loss)]
pub fn display_memory_analysis(analysis: &ProfileAnalysis) {
    println!("📊 Memory Profile Analysis Results");
    println!("═══════════════════════════════════");
    println!(
        "Total Memory Allocated: {:.2} KB",
        analysis.total_duration_us as f64 / 1024.0
    );
    println!();

    println!("🏆 Top Functions by Memory Allocation:");
    println!("──────────────────────────────────────");

    for (i, (name, bytes, percentage)) in analysis.top_functions.iter().enumerate() {
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
    println!("💡 Memory Allocation Insights:");
    println!("─────────────────────────────");
    for insight in &analysis.insights {
        println!("{}", insight);
    }
    println!();
}

/// Clean function names for better display
fn clean_function_name(name: &str) -> String {
    let clean = name.split("::").last().unwrap_or(name);
    let clean = clean.split('<').next().unwrap_or(clean);
    let clean = clean.split('(').next().unwrap_or(clean);

    match clean {
        s if s.starts_with("thag_demo_") => s.strip_prefix("thag_demo_").unwrap_or(s).to_string(),
        s if s.contains("fibonacci") => {
            if s.contains("cached") {
                "fibonacci_cached"
            } else if s.contains("iter") {
                "fibonacci_iter"
            } else {
                "fibonacci_recursive"
            }
        }
        .to_string(),
        s if s.contains("bubble_sort") => "bubble_sort".to_string(),
        s if s.contains("quicksort") => "quicksort".to_string(),
        s if s.contains("string_concat") => {
            if s.contains("efficient") {
                "string_concat_efficient"
            } else if s.contains("naive") {
                "string_concat_naive"
            } else {
                "string_concat"
            }
        }
        .to_string(),
        s if s.contains("lookup") => {
            if s.contains("hashmap") {
                "lookup_hashmap"
            } else if s.contains("vector") {
                "lookup_vector"
            } else {
                "lookup"
            }
        }
        .to_string(),
        s if s.contains("cpu_intensive") => "cpu_intensive_work".to_string(),
        s if s.contains("simulated_io") => "simulated_io_work".to_string(),
        s if s.contains("nested_function") => "nested_function_calls".to_string(),
        // Memory profiling patterns
        s if s.contains("allocate_vectors") => "allocate_vectors".to_string(),
        s if s.contains("process_strings") => "process_strings_detail".to_string(),
        s if s.contains("memory_intensive") => "memory_intensive_computation".to_string(),
        s if s.contains("nested_allocations") => "nested_allocations".to_string(),
        s => s.to_string(),
    }
}

/// Generate insights from function timing data
#[allow(clippy::cast_precision_loss)]
fn generate_insights(functions: &[(String, u128)], total_duration_us: u128) -> Vec<String> {
    let mut insights = Vec::new();

    if functions.len() >= 2 {
        let slowest = &functions[0];
        let fastest = &functions[functions.len() - 1];

        if fastest.1 > 0 {
            let speedup = slowest.1 as f64 / fastest.1 as f64;
            insights.push(format!(
                "🐌 Slowest: {} ({:.3}ms)",
                slowest.0,
                slowest.1 as f64 / 1000.0
            ));
            insights.push(format!(
                "🚀 Fastest: {} ({:.3}ms)",
                fastest.0,
                fastest.1 as f64 / 1000.0
            ));
            insights.push(format!("⚡ Performance difference: {:.1}x", speedup));

            if speedup > 1000.0 {
                insights.push("🎯 Consider using faster algorithms in production!".to_string());
            }
        }
    }

    // Look for specific patterns
    let has_recursive = functions.iter().any(|(name, _)| name.contains("recursive"));
    let has_cached = functions.iter().any(|(name, _)| name.contains("cached"));
    let has_iter = functions.iter().any(|(name, _)| name.contains("iter"));
    let has_vectors = functions.iter().any(|(name, _)| name.contains("vector"));
    let has_strings = functions.iter().any(|(name, _)| name.contains("string"));
    let has_hash_map = functions
        .iter()
        .any(|(name, _)| name.contains("HashMap") || name.contains("map"));

    if has_recursive && has_cached {
        insights.push("🔧 Tip: Caching can dramatically improve recursive algorithms!".to_string());
    }

    if has_iter {
        insights.push(
            "🔄 Tip: Iterative approaches often outperform recursion for large inputs!".to_string(),
        );
    }

    // Memory-specific insights
    if has_vectors {
        insights.push(
            "📋 Tip: Vector allocations detected - consider pre-allocating with capacity!"
                .to_string(),
        );
    }

    if has_strings {
        insights.push("📝 Tip: String operations found - consider using String::with_capacity() for better performance!".to_string());
    }

    if has_hash_map {
        insights.push(
            "🗺️  Tip: HashMap usage detected - consider pre-sizing for known data sizes!"
                .to_string(),
        );
    }

    // General performance advice based on total duration
    let total_ms = total_duration_us as f64 / 1000.0;
    if total_ms > 1000.0 {
        insights.push(
            "⏱️  Consider profiling with release mode for production performance analysis!"
                .to_string(),
        );
    }

    insights
}

/// Helper trait for string case conversion
trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().map_or_else(String::new, |first| {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                })
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_title_case() {
        assert_eq!("hello_world".to_title_case(), "Hello World");
        assert_eq!("basic_profiling".to_title_case(), "Basic Profiling");
        assert_eq!("single_word".to_title_case(), "Single Word");
    }

    #[test]
    fn test_default_config() {
        let config = VisualizationConfig::default();
        assert_eq!(config.title, "Profiling Analysis");
        assert_eq!(config.count_name, "μs");
        assert_eq!(config.flame_chart, true);
    }

    #[test]
    fn test_clean_function_name() {
        assert_eq!(clean_function_name("thag_demo_fibonacci"), "fibonacci");
        assert_eq!(
            clean_function_name("module::fibonacci_cached"),
            "fibonacci_cached"
        );
        assert_eq!(clean_function_name("allocate_vectors"), "allocate_vectors");
    }
}
