//! Visualization library for thag_demo profiling analysis
//!
//! This module provides reusable visualization functions for analyzing profiling data,
//! including flamegraphs, flamecharts, and differential comparisons.

use chrono::Local;
use inferno::flamegraph::{self, color::MultiPalette, Options, Palette};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use thag_profiler::enhance_svg_accessibility;

pub mod differential;
pub mod flamegraph_gen;
pub mod profile_analysis;

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
    /// Whether to generate flamechart (aggregated) vs flamegraph (timeline)
    pub flame_chart: bool,
    /// Whether to open in browser automatically
    pub open_browser: bool,
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
            open_browser: false,
        }
    }
}

/// Type of analysis to perform
#[derive(Debug, Clone)]
pub enum AnalysisType {
    /// Single profile analysis
    Single,
    /// Differential comparison between two profiles
    Differential {
        before_name: String,
        after_name: String,
    },
}

/// Generate visualization from folded profile data
pub fn generate_visualization(
    folded_files: &[PathBuf],
    output_path: &str,
    config: VisualizationConfig,
    analysis_type: AnalysisType,
) -> Result<(), Box<dyn std::error::Error>> {
    match analysis_type {
        AnalysisType::Single => {
            if folded_files.is_empty() {
                return Err("No profile files provided for single analysis".into());
            }
            generate_single_visualization(&folded_files[0], output_path, config)
        }
        AnalysisType::Differential {
            before_name,
            after_name,
        } => {
            if folded_files.len() < 2 {
                return Err("Need at least two profile files for differential analysis".into());
            }
            differential::generate_differential_visualization(
                &folded_files[0],
                &folded_files[1],
                output_path,
                config,
                &before_name,
                &after_name,
            )
        }
    }
}

/// Generate single profile visualization
fn generate_single_visualization(
    folded_file: &PathBuf,
    output_path: &str,
    config: VisualizationConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(folded_file)?;
    let stacks: Vec<String> = content.lines().map(|line| line.to_string()).collect();

    if stacks.is_empty() {
        return Err("No profile data found in file".into());
    }

    flamegraph_gen::generate_flamegraph_svg(&stacks, output_path, config)
}

/// Find the most recent profile files matching a pattern
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
pub fn show_interactive_prompt(
    demo_name: &str,
    analysis_type: AnalysisType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    match analysis_type {
        AnalysisType::Single => {
            println!("🎯 Would you like to view an interactive flamechart?");
            println!("This will generate a visual flamechart and open it in your browser.");
        }
        AnalysisType::Differential {
            ref before_name,
            ref after_name,
        } => {
            println!("🎯 Would you like to view a differential comparison?");
            println!("This will generate a visual comparison between {} and {} and open it in your browser.", before_name, after_name);
        }
    }

    print!("Enter 'y' for yes, or any other key to skip: ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        if input.trim().to_lowercase() == "y" {
            println!();
            match analysis_type {
                AnalysisType::Single => {
                    println!("🔥 Generating interactive flamechart...");
                    generate_and_show_single_visualization(demo_name)?;
                }
                AnalysisType::Differential {
                    before_name,
                    after_name,
                } => {
                    println!("🔥 Generating differential comparison...");
                    generate_and_show_differential_visualization(
                        demo_name,
                        &before_name,
                        &after_name,
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Generate and show single visualization
fn generate_and_show_single_visualization(
    demo_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let files = find_latest_profile_files(&format!("thag_demo_{}", demo_name), 1)?;

    if files.is_empty() {
        return Err("No profile files found".into());
    }

    let config = VisualizationConfig {
        title: format!("{} Demo - Performance Flamechart", demo_name.replace('_', " ").to_title_case()),
        subtitle: Some(format!(
            "Generated: {} | Hover over and click on the bars to explore the function call hierarchy, or use Search ↗️",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        )),
        ..Default::default()
    };

    let output_path = format!("{}_flamechart.svg", demo_name);

    generate_visualization(&files, &output_path, config, AnalysisType::Single)?;

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

/// Generate and show differential visualization
fn generate_and_show_differential_visualization(
    demo_name: &str,
    before_name: &str,
    after_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let before_files = find_latest_profile_files(&format!("thag_demo_{}_before", demo_name), 1)?;
    let after_files = find_latest_profile_files(&format!("thag_demo_{}_after", demo_name), 1)?;

    if before_files.is_empty() || after_files.is_empty() {
        return Err("Could not find both before and after profile files".into());
    }

    let config = VisualizationConfig {
        title: format!(
            "{} Demo - Performance Comparison",
            demo_name.replace('_', " ").to_title_case()
        ),
        subtitle: Some(format!(
            "Differential Analysis: {} vs {} | Generated: {} | Red=Slower, Blue=Faster",
            before_name,
            after_name,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        )),
        palette: Palette::Multi(MultiPalette::Java),
        ..Default::default()
    };

    let output_path = format!("{}_differential.svg", demo_name);
    let files = vec![before_files[0].clone(), after_files[0].clone()];

    generate_visualization(
        &files,
        &output_path,
        config,
        AnalysisType::Differential {
            before_name: before_name.to_string(),
            after_name: after_name.to_string(),
        },
    )?;

    println!("✅ Differential comparison generated: {}", output_path);

    if let Err(e) = open_in_browser(&output_path) {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("💡 You can manually open: {}", output_path);
    } else {
        println!("🌐 Differential comparison opened in your default browser!");
        println!(
            "🔍 Red bars show functions that got slower, blue bars show functions that got faster"
        );
        println!("📊 Bar width represents the performance difference magnitude");
    }

    Ok(())
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
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
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
}
