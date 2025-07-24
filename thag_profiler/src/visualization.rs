//! Demo visualization utilities for `thag_profiler`
//!
//! This module provides reusable visualization functions for analyzing profiling data,
//! including flamegraphs, flamecharts, and profile analysis. It's designed to be used
//! in demo scripts and examples.
//!
//! This module is only available when the `demo` feature is enabled.

use crate::{enhance_svg_accessibility, file_stem_from_path_str, timing, ProfileType};
use chrono::Local;
use inferno::flamegraph::color::BasicPalette::Mem;
use inferno::flamegraph::Palette::Basic;
use inferno::flamegraph::{self, color::MultiPalette, Options, Palette};
use smol;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::string::ToString;
use strum::Display;

#[derive(Debug, Default, Clone, Copy, Display, PartialEq, Eq)]
/// Type of analysis visualization to generate
pub enum AnalysisType {
    /// Differential analysis comparing two profiles
    Differential,
    /// Timeline-based flamechart showing execution over time
    #[default]
    Flamechart,
    /// Aggregated flamegraph showing cumulative execution time
    Flamegraph,
}

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
    pub analysis_type: AnalysisType,
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
            analysis_type: AnalysisType::Flamechart,
        }
    }
}

/// Profile analysis results
#[derive(Debug, Clone)]
pub struct ProfileAnalysis {
    /// Total duration of the profiling session in microseconds, or total of allocations in bytes
    pub metric_total: u128,
    /// Function names paired with their metric values (execution times in microseconds or allocation values in bytes)
    pub function_value_pairs: Vec<(String, u128)>,
    /// Top functions with name, metric value, and percentage of metric total value
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
#[timing]
pub fn generate_flamegraph_svg(
    output_path: &str,
    config: VisualizationConfig,
    content: String,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let stacks: Vec<&str> = content.lines().collect();
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
    opts.flame_chart = matches!(config.analysis_type, AnalysisType::Flamechart);

    // Generate flamegraph
    let output = std::fs::File::create(output_path)?;

    flamegraph::from_lines(&mut opts, stacks.iter().rev().copied(), output)?;

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
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let content = std::fs::read_to_string(folded_file)?;
    let stacks: Vec<&str> = content.lines().collect();
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
    opts.flame_chart = matches!(config.analysis_type, AnalysisType::Flamechart);

    // Generate flamegraph
    let output = std::fs::File::create(output_path)?;

    flamegraph::from_lines(&mut opts, stacks.iter().rev().copied(), output)?;

    // Enhance accessibility
    enhance_svg_accessibility(output_path)?;

    Ok(())
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
pub fn analyze_profile(
    profile_type: ProfileType,
    file_path: &PathBuf,
) -> Result<ProfileAnalysis, Box<dyn Error + Send + Sync + 'static>> {
    let content = std::fs::read_to_string(file_path)?;
    // eprintln!("content=\n{content}");
    let mut function_value_map: HashMap<String, u128> = HashMap::new();
    let mut metric_total = 0u128;

    // Parse folded stack format
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Handle spaces in function names by splitting on last whitespace only
        if let Some(last_space_pos) = line.rfind(' ') {
            let stack = &line[..last_space_pos];
            let value_str = &line[last_space_pos + 1..];

            if let Ok(value) = value_str.parse::<u128>() {
                metric_total += value;

                // Extract function names from the stack
                let functions: Vec<&str> = stack.split(';').collect();

                // Skip root function
                for &func_name in &functions[1..] {
                    let clean_name = clean_function_name(func_name);
                    // eprintln!("clean_name={clean_name}, value={value}");
                    *function_value_map.entry(clean_name).or_insert(0) += value;
                }
            }
        }
    }

    let mut function_value_pairs: Vec<_> = function_value_map.into_iter().collect();
    function_value_pairs.sort_by(|a, b| b.1.cmp(&a.1));

    let top_functions: Vec<_> = function_value_pairs
        .iter()
        .take(10)
        .map(|(name, value)| {
            let percentage = (*value as f64 / metric_total as f64) * 100.0;
            (name.clone(), *value, percentage)
        })
        .collect();

    let insights = generate_insights(profile_type, &function_value_pairs, metric_total);

    Ok(ProfileAnalysis {
        metric_total,
        function_value_pairs,
        top_functions,
        insights,
    })
}

/// Display profile analysis results
///
/// # Panics
///
/// Will panic if profile type isn't one of Time or Memory
#[allow(clippy::cast_precision_loss)]
pub fn display_analysis(profile_type: ProfileType, analysis: &ProfileAnalysis) {
    let (title1, title2, title3, metric_desc, thousands) = match profile_type {
        ProfileType::Memory => (
            "Memory Allocation",
            "Memory Allocation",
            "Memory Allocated",
            "Memory Allocations",
            "kB",
        ),
        ProfileType::Time => (
            "Execution Timeline",
            "Execution Time",
            "Duration",
            "Execution Times",
            "ms",
        ),
        ProfileType::Both | ProfileType::None => {
            panic!("Profile type must be Time or Memory")
        }
    };

    println!();
    println!("📊 {metric_desc} Analysis Results");
    println!("{}", "═".repeat(20 + metric_desc.len()));
    println!(
        "Total {title3}: {:.3}{thousands}",
        analysis.metric_total as f64 / 1000.0
    );
    println!();

    println!("🏆 Top Functions by {title2}:");
    println!("{}", "─".repeat(21 + title2.len()));

    for (i, (name, metric_value, percentage)) in analysis.top_functions.iter().enumerate() {
        let value = *metric_value as f64 / 1000.0;

        let icon = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "🏅",
        };

        println!(
            "{} {}. {} - {:.0}{thousands} ({:.1}%)",
            icon,
            i + 1,
            name,
            value,
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
    let _ = std::io::stdout().flush();
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
    is_memory: bool,
    count: usize,
) -> Result<Vec<PathBuf>, Box<dyn Error + Send + Sync + 'static>> {
    let current_dir = std::env::current_dir()?;
    let mut files = Vec::new();

    for entry in std::fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let is_memory_file = name.ends_with("-memory.folded");
            if name.contains(pattern) && name.ends_with(".folded") && is_memory == is_memory_file {
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

    if files.is_empty() {
        println!(
            "⚠️  No {} profile .folded files found for {pattern}",
            if is_memory { "memory" } else { "time" }
        );
    } else {
        files.truncate(count);
    }
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
pub fn open_in_browser(file_path: &str) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
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

/// Run prompted analysis for profiling visualization
///
/// This function provides an interactive prompt to the user asking if they want to
/// generate and view a profiling visualization. It handles the async execution
/// of the interactive prompt and error handling.
///
/// # Arguments
/// * `demo_name` - Name of the demo being profiled
/// * `profile_type` - Type of profiling (Time or Memory)
/// * `analysis_type` - Type of analysis visualization to generate
pub fn prompted_analysis(
    file_path_str: &str,
    profile_type: ProfileType,
    analysis_type: AnalysisType,
) {
    let run_analysis = async || {
        // Interactive visualization: must run AFTER function with `enable_profiling` profiling attribute,
        // because profile output is only available after that function completes.
        if let Err(e) = show_interactive_prompt(file_path_str, profile_type, analysis_type).await {
            eprintln!("⚠️ Could not show interactive memory visualization: {e}");
        }
    };

    smol::block_on(run_analysis());
}

/// Show interactive visualization prompt
///
/// # Errors
///
/// Returns an error if:
/// - Cannot read user input from stdin
/// - System command to open browser fails
pub async fn show_interactive_prompt(
    file_path_str: &str,
    profile_type: ProfileType,
    analysis_type: AnalysisType,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let analysis_type_lower = analysis_type.to_string().to_lowercase();

    println!();
    println!("🎯 Would you like to view an interactive {profile_type} {analysis_type_lower}?");
    println!(
        "This will generate a visual {profile_type} {analysis_type_lower} and open it in your browser.");

    print!("Enter 'y' for yes, or any other key to skip: ");
    let _ = std::io::stdout().flush();

    let mut input = String::new();
    let show_graph =
        std::io::stdin().read_line(&mut input).is_ok() && input.trim().to_lowercase() == "y";

    let file_stem = file_stem_from_path_str(file_path_str);
    generate_and_show_visualization(&file_stem, profile_type, analysis_type, show_graph).await?;

    Ok(())
}

/// Generate visualization
///
/// # Errors
///
/// Returns an error if:
/// - Profile files cannot be found or read
/// - Flamegraph generation fails
/// - Browser cannot be opened to display results
pub async fn generate_and_show_visualization(
    file_stem: &str,
    profile_type: ProfileType,
    analysis_type: AnalysisType,
    show_graph: bool,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let profile_type_title = profile_type.to_string().to_title_case();
    let analysis_type_lower = analysis_type.to_string().to_lowercase();

    let is_memory = ProfileType::Memory == profile_type;
    let files = find_latest_profile_files(file_stem, is_memory, 1)?;

    if files.is_empty() {
        println!("💡 Make sure the demo completed successfully and generated profile files.");
        return Ok(());
    }

    // Show analysis first
    let analysis = analyze_profile(profile_type, &files[0])?;
    let demo_name = file_stem.to_string();

    if show_graph {
        println!("🔥 Generating interactive {analysis_type_lower} in background...");
        let _ = std::io::stdout().flush();

        let bg_task = smol::unblock(move || {
            generate_and_show_flamegraph(demo_name, profile_type, analysis_type, &files)
        });

        // Show analysis immediately while flamegraph generates in background
        display_analysis(profile_type, &analysis);

        println!("\n⏳ Waiting for flamegraph generation to complete...");
        let _ = std::io::stdout().flush();

        // Await the background task and handle any errors
        match bg_task.await {
            Ok(()) => println!("✅ Flamegraph generation completed!"),
            Err(e) => {
                eprintln!("⚠️ Flamegraph generation failed: {}", e);
                println!("💡 Analysis results are still available above.");
            }
        }
    } else {
        display_analysis(profile_type, &analysis);
    }

    Ok(())
}

#[timing]
fn generate_and_show_flamegraph(
    demo_name: String,
    profile_type: ProfileType,
    analysis_type: AnalysisType,
    files: &[PathBuf],
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let profile_type_title = profile_type.to_string().to_title_case();
    let analysis_type_lower = &analysis_type.to_string().to_lowercase();
    let (title, metric_desc) = match &profile_type {
        ProfileType::Memory => ("Memory Allocation", "memory allocations"),
        ProfileType::Time => ("Execution Timeline", "execution times"),
        &ProfileType::Both | &ProfileType::None => {
            panic!("Profile type must be Time or Memory")
        }
    };
    let graph_desc = match analysis_type {
        AnalysisType::Flamechart => "Flamechart (Individual)",
        AnalysisType::Flamegraph => "Flamegraph (Aggregated)",
        AnalysisType::Differential => {
            panic!("Analysis type must be Flamegraph or Flamechart")
        }
    };

    println!("🔥 Generating interactive {profile_type} {analysis_type_lower}...");
    println!();

    let filename = files[0].file_stem().unwrap().display().to_string();
    // eprintln!(
    //     "filename={filename}, extracted timestamp={}",
    //     extract_filename_timestamp(&filename)
    //         .format("%Y-%m-%d %H:%M:%S")
    //         .to_string()
    // );

    let config = VisualizationConfig {
        title: format!("{title} {graph_desc}"),
        subtitle: Some(format!(
            "{filename} | Hover over and click on the bars to explore, or use Search ↗️"
        )),
        palette: match &profile_type {
            ProfileType::Time => Palette::Multi(MultiPalette::Rust),
            ProfileType::Memory => Basic(Mem),
            &ProfileType::Both | &ProfileType::None => {
                panic!("Profile type must be Time or Memory")
            }
        },
        count_name: match &profile_type {
            ProfileType::Memory => "bytes".to_string(),
            ProfileType::Time => "μs".to_string(),
            &ProfileType::Both | &ProfileType::None => {
                panic!("Profile type must be Time or Memory")
            }
        },
        analysis_type,
        ..Default::default()
    };
    let output_path = format!("{demo_name}_{profile_type}_{analysis_type_lower}.svg");
    eprintln!("\nprofile_type={profile_type}, analysis_type_lower={analysis_type_lower}, output_path={output_path}\n");
    generate_flamegraph_from_file(&files[0], &output_path, config)?;
    println!("✅ {profile_type_title} {analysis_type} generated: {output_path}");

    if let Err(e) = open_in_browser(&output_path) {
        println!("⚠️ Could not open browser automatically: {e}");
        println!("💡 You can manually open: {output_path}");
    } else {
        println!("🌐 {profile_type_title} {analysis_type} opened in your default browser!");
        println!(
            "🔍 Hover over and click on the bars to explore {}",
            metric_desc
        );
        println!("📊 Function width = {metric_desc}, height = call stack depth");
    }
    Ok(())
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
        // Memory profiling patterns - be more specific to avoid collisions
        s if s.contains("allocate_vectors") => "allocate_vectors".to_string(),
        s if s.contains("process_strings_detail_profile") => {
            "process_strings_detail_profile".to_string()
        }
        s if s.contains("process_strings_detail") => "process_strings_detail".to_string(),
        s if s.contains("process_strings") => "process_strings".to_string(),
        s if s.contains("memory_intensive") => "memory_intensive_computation".to_string(),
        s if s.contains("nested_allocations") => "nested_allocations".to_string(),
        s => s.to_string(),
    }
}

/// Generate insights from function timing data
#[allow(clippy::cast_precision_loss)]
fn generate_insights(
    profile_type: ProfileType,
    functions: &[(String, u128)],
    metric_total: u128,
) -> Vec<String> {
    let (greatest_desc, least_desc, better_desc, units, thousands, threshold) = match &profile_type
    {
        ProfileType::Memory => ("Largest", "Smallest", "leaner", "bytes", "kB", 1.0),
        ProfileType::Time => ("Slowest", "Fastest", "faster", "μs", "ms", 1000.0),
        &ProfileType::Both | &ProfileType::None => {
            panic!("Profile type must be Time or Memory")
        }
    };
    let mut insights = Vec::new();

    if functions.len() >= 2 {
        let biggest = &functions[0];
        let smallest = &functions[functions.len() - 1];

        if smallest.1 > 0 {
            let ratio = biggest.1 as f64 / smallest.1 as f64;
            insights.push(format!(
                "🐌 {greatest_desc}: {} ({:.3}{thousands})",
                biggest.0,
                biggest.1 as f64 / 1000.0
            ));
            insights.push(format!(
                "🚀 {least_desc}: {} ({:.3}{thousands})",
                smallest.0,
                smallest.1 as f64 / 1000.0
            ));
            insights.push(format!("⚡ Performance difference: {:.1}x", ratio));

            if ratio > threshold {
                insights.push(format!(
                    "🎯 Consider using the {better_desc} algorithm(s) in production, if applicable"
                ));
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
        insights.push("🔧 Tip: Caching can dramatically improve recursive algorithms".to_string());
    }

    if has_iter {
        insights.push(
            "🔄 Tip: Iterative approaches often outperform recursion for large inputs".to_string(),
        );
    }

    // Memory-specific insights
    if has_vectors {
        insights.push(
            "📋 Tip: Vector allocations detected - consider pre-allocating with capacity"
                .to_string(),
        );
    }

    if has_strings {
        insights.push("📝 Tip: String operations found - consider using String::with_capacity() for better performance".to_string());
    }

    if has_hash_map {
        insights.push(
            "🗺️ Tip: HashMap usage detected - consider pre-sizing for known data sizes".to_string(),
        );
    }

    // General performance advice based on total duration
    let total_thousands = metric_total as f64 / 1000.0;
    if total_thousands > 1000.0 {
        insights.push(
            "⏱️ Consider profiling with release mode for production performance analysis"
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
        assert_eq!(config.analysis_type, AnalysisType::Flamechart);
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
