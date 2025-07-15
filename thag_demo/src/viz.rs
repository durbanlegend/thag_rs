//! Interactive profile visualization module for thag_demo
//!
//! This module provides text-based interactive visualization of profiling data
//! within the terminal, eliminating the need for external tools and providing
//! immediate feedback for demo users.

use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

/// Represents profiling data for a single function
#[derive(Debug, Clone)]
pub struct FunctionProfile {
    pub name: String,
    pub total_time_us: u128,
    pub call_count: u64,
    pub avg_time_us: u128,
    pub percentage: f64,
}

/// Container for all profiling data
#[derive(Debug)]
pub struct ProfileData {
    pub functions: Vec<FunctionProfile>,
    pub total_duration_us: u128,
    pub title: String,
    pub timestamp: String,
}

/// Interactive profile viewer
pub struct ProfileViewer {
    data: ProfileData,
    current_sort: SortMode,
    show_details: bool,
}

#[derive(Debug, Clone, Copy)]
enum SortMode {
    TotalTime,
    CallCount,
    AverageTime,
    Name,
}

impl ProfileViewer {
    /// Create a new profile viewer from profile data
    pub fn new(data: ProfileData) -> Self {
        Self {
            data,
            current_sort: SortMode::TotalTime,
            show_details: false,
        }
    }

    /// Create a profile viewer by parsing the most recent profile files
    pub fn from_recent_profiles(profile_prefix: &str) -> Result<Self> {
        let profile_data = parse_recent_profile_files(profile_prefix)?;
        Ok(Self::new(profile_data))
    }

    /// Start the interactive viewer
    pub fn run(&mut self) -> Result<()> {
        println!("{}", "🔍 Interactive Profile Viewer".bold().cyan());
        println!("{}", "═".repeat(40).cyan());
        println!();

        loop {
            self.display_profile();
            self.display_menu();

            match self.get_user_input()? {
                UserChoice::SortByTime => {
                    self.current_sort = SortMode::TotalTime;
                    println!("{}", "📊 Sorted by total time".green());
                }
                UserChoice::SortByCalls => {
                    self.current_sort = SortMode::CallCount;
                    println!("{}", "📊 Sorted by call count".green());
                }
                UserChoice::SortByAverage => {
                    self.current_sort = SortMode::AverageTime;
                    println!("{}", "📊 Sorted by average time".green());
                }
                UserChoice::SortByName => {
                    self.current_sort = SortMode::Name;
                    println!("{}", "📊 Sorted by function name".green());
                }
                UserChoice::ToggleDetails => {
                    self.show_details = !self.show_details;
                    let status = if self.show_details {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    println!("{}", format!("🔍 Details view {}", status).green());
                }
                UserChoice::ShowTop5 => {
                    self.show_top_functions(5);
                }
                UserChoice::ShowTop10 => {
                    self.show_top_functions(10);
                }
                UserChoice::ShowComparison => {
                    self.show_performance_comparison();
                }
                UserChoice::Quit => {
                    println!("{}", "👋 Goodbye!".cyan());
                    break;
                }
                UserChoice::Invalid => {
                    println!("{}", "❌ Invalid choice. Please try again.".red());
                }
            }

            println!();
            self.pause_for_user();
        }

        Ok(())
    }

    fn display_profile(&mut self) {
        // Sort functions based on current sort mode
        self.sort_functions();

        println!("{}", self.data.title.bold().green());
        println!("{}", format!("Timestamp: {}", self.data.timestamp).dimmed());
        println!(
            "{}",
            format!(
                "Total Duration: {:.3}ms",
                self.data.total_duration_us as f64 / 1000.0
            )
            .dimmed()
        );
        println!();

        // Header
        println!("{}", "Function Performance Analysis".bold().yellow());
        println!("{}", "─".repeat(80).yellow());

        if self.show_details {
            println!(
                "{:<30} {:>12} {:>12} {:>12} {:>8}",
                "Function".bold(),
                "Total (μs)".bold(),
                "Calls".bold(),
                "Avg (μs)".bold(),
                "% Time".bold()
            );
            println!("{}", "─".repeat(80));
        }

        for (i, func) in self.data.functions.iter().enumerate() {
            if self.show_details {
                self.display_function_detailed(func);
            } else {
                self.display_function_summary(func, i + 1);
            }
        }

        println!();
    }

    fn display_function_detailed(&self, func: &FunctionProfile) {
        let time_bar = self.create_time_bar(func.percentage);
        println!(
            "{:<30} {:>12} {:>12} {:>12} {:>7.1}% {}",
            func.name.truncate_ellipsis(30),
            format_number(func.total_time_us),
            format_number(func.call_count),
            format_number(func.avg_time_us),
            func.percentage,
            time_bar
        );
    }

    fn display_function_summary(&self, func: &FunctionProfile, rank: usize) {
        let time_color = match func.percentage {
            p if p > 50.0 => "red",
            p if p > 20.0 => "yellow",
            p if p > 5.0 => "green",
            _ => "cyan",
        };

        println!(
            "{}. {} - {:.3}ms ({:.1}%, {} calls)",
            rank.to_string().bold(),
            func.name.color(time_color).bold(),
            func.total_time_us as f64 / 1000.0,
            func.percentage,
            format_number(func.call_count)
        );
    }

    fn create_time_bar(&self, percentage: f64) -> String {
        let bar_length = 20;
        let filled_length = ((percentage / 100.0) * bar_length as f64) as usize;
        let bar = "█".repeat(filled_length.min(bar_length));
        let empty = "░".repeat(bar_length - filled_length.min(bar_length));
        format!("{}{}", bar.green(), empty.dimmed())
    }

    fn sort_functions(&mut self) {
        match self.current_sort {
            SortMode::TotalTime => {
                self.data
                    .functions
                    .sort_by(|a, b| b.total_time_us.cmp(&a.total_time_us));
            }
            SortMode::CallCount => {
                self.data
                    .functions
                    .sort_by(|a, b| b.call_count.cmp(&a.call_count));
            }
            SortMode::AverageTime => {
                self.data
                    .functions
                    .sort_by(|a, b| b.avg_time_us.cmp(&a.avg_time_us));
            }
            SortMode::Name => {
                self.data.functions.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
    }

    fn display_menu(&self) {
        println!("{}", "Interactive Options:".bold().blue());
        println!("{}", "────────────────────".blue());
        println!("  {} - Sort by total time", "1".bold());
        println!("  {} - Sort by call count", "2".bold());
        println!("  {} - Sort by average time", "3".bold());
        println!("  {} - Sort by function name", "4".bold());
        println!("  {} - Toggle detailed view", "d".bold());
        println!("  {} - Show top 5 functions", "5".bold());
        println!("  {} - Show top 10 functions", "0".bold());
        println!("  {} - Show performance comparison", "c".bold());
        println!("  {} - Quit", "q".bold());
        println!();
        print!("{}", "Enter your choice: ".yellow());
        io::stdout().flush().unwrap();
    }

    fn get_user_input(&self) -> Result<UserChoice> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => Ok(UserChoice::SortByTime),
            "2" => Ok(UserChoice::SortByCalls),
            "3" => Ok(UserChoice::SortByAverage),
            "4" => Ok(UserChoice::SortByName),
            "d" | "D" => Ok(UserChoice::ToggleDetails),
            "5" => Ok(UserChoice::ShowTop5),
            "0" => Ok(UserChoice::ShowTop10),
            "c" | "C" => Ok(UserChoice::ShowComparison),
            "q" | "Q" | "quit" | "exit" => Ok(UserChoice::Quit),
            _ => Ok(UserChoice::Invalid),
        }
    }

    fn show_top_functions(&self, count: usize) {
        println!(
            "{}",
            format!("🏆 Top {} Functions by Total Time", count)
                .bold()
                .green()
        );
        println!("{}", "═".repeat(50).green());

        for (i, func) in self.data.functions.iter().take(count).enumerate() {
            let medal = match i {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "🏅",
            };

            println!(
                "{} {}. {} - {:.3}ms ({:.1}%)",
                medal,
                i + 1,
                func.name.bold(),
                func.total_time_us as f64 / 1000.0,
                func.percentage
            );
        }
        println!();
    }

    fn show_performance_comparison(&self) {
        println!("{}", "⚡ Performance Comparison".bold().green());
        println!("{}", "═".repeat(50).green());

        if self.data.functions.len() < 2 {
            println!("{}", "Need at least 2 functions for comparison".yellow());
            return;
        }

        let fastest = &self.data.functions[self.data.functions.len() - 1];
        let slowest = &self.data.functions[0];

        if fastest.total_time_us > 0 {
            let speedup = slowest.total_time_us as f64 / fastest.total_time_us as f64;
            println!(
                "🐌 Slowest: {} ({:.3}ms)",
                slowest.name.red().bold(),
                slowest.total_time_us as f64 / 1000.0
            );
            println!(
                "🚀 Fastest: {} ({:.3}ms)",
                fastest.name.green().bold(),
                fastest.total_time_us as f64 / 1000.0
            );
            println!(
                "⚡ Speedup: {:.1}x faster",
                speedup.to_string().bold().yellow()
            );

            if speedup > 1000.0 {
                println!(
                    "{}",
                    "💡 Consider using the faster algorithm in production!".bright_blue()
                );
            }
        }
        println!();
    }

    fn pause_for_user(&self) {
        print!("{}", "Press Enter to continue...".dimmed());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }
}

#[derive(Debug)]
enum UserChoice {
    SortByTime,
    SortByCalls,
    SortByAverage,
    SortByName,
    ToggleDetails,
    ShowTop5,
    ShowTop10,
    ShowComparison,
    Quit,
    Invalid,
}

/// Parse the most recent profile files to extract function statistics
pub fn parse_recent_profile_files(profile_prefix: &str) -> Result<ProfileData> {
    // Find the most recent profile files
    let current_dir = std::env::current_dir()?;
    let mut profile_files = Vec::new();

    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(profile_prefix) && name.ends_with(".folded") {
                profile_files.push(path);
            }
        }
    }

    if profile_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No profile files found with prefix '{}'",
            profile_prefix
        ));
    }

    // Sort by modification time, most recent first
    profile_files.sort_by(|a, b| {
        let time_a = fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    let profile_file = &profile_files[0];
    parse_profile_file(profile_file)
}

/// Parse a single profile file to extract function statistics
fn parse_profile_file(file_path: &PathBuf) -> Result<ProfileData> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read profile file: {}", file_path.display()))?;

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

    // Create function profiles
    let mut functions = Vec::new();
    for (name, total_time) in function_times {
        let percentage = if total_duration_us > 0 {
            (total_time as f64 / total_duration_us as f64) * 100.0
        } else {
            0.0
        };

        functions.push(FunctionProfile {
            name: name.clone(),
            total_time_us: total_time,
            call_count: 1, // We can't determine call count from folded format
            avg_time_us: total_time,
            percentage,
        });
    }

    // Sort by total time descending
    functions.sort_by(|a, b| b.total_time_us.cmp(&a.total_time_us));

    Ok(ProfileData {
        functions,
        total_duration_us,
        title: format!(
            "Profile Analysis: {}",
            file_path.file_name().unwrap().to_string_lossy()
        ),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

/// Clean up function names for better display
fn clean_function_name(name: &str) -> String {
    // Remove module paths and keep just the function name
    let clean = name.split("::").last().unwrap_or(name);

    // Remove template parameters and other noise
    let clean = clean.split('<').next().unwrap_or(clean);
    let clean = clean.split('(').next().unwrap_or(clean);

    // Handle common patterns
    match clean {
        s if s.starts_with("thag_demo_") => s.strip_prefix("thag_demo_").unwrap_or(s).to_string(),
        s if s.contains("fibonacci") => s.to_string(),
        s if s.contains("cpu_intensive") => "cpu_intensive_work".to_string(),
        s if s.contains("simulated_io") => "simulated_io_work".to_string(),
        s if s.contains("nested_function") => "nested_function_calls".to_string(),
        s => s.to_string(),
    }
}

/// Format large numbers with thousands separators
fn format_number<T: std::fmt::Display>(n: T) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

/// Extension trait for string truncation
trait StringExt {
    fn truncate_ellipsis(&self, max_len: usize) -> String;
}

impl StringExt for str {
    fn truncate_ellipsis(&self, max_len: usize) -> String {
        if self.len() <= max_len {
            self.to_string()
        } else {
            format!("{}...", &self[..max_len.saturating_sub(3)])
        }
    }
}
