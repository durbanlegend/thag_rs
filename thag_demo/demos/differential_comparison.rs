/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }
inferno = "0.11"
chrono = { version = "0.4", features = ["serde"] }

[profile.release]
debug = true
strip = false
*/

/// Differential comparison demo - orchestrates before/after performance analysis
/// This demo demonstrates true differential profiling using thag_profiler/inferno
//# Purpose: Demonstrate differential profiling with before/after comparison
//# Categories: profiling, demo, comparison, optimization, differential
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

// Inline visualization functionality for this demo
mod visualization {
    use chrono::Local;
    use inferno::flamegraph::{color::MultiPalette, Palette};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use thag_profiler::enhance_svg_accessibility;

    pub mod differential {
        use super::*;
        use inferno::flamegraph::{self, Options};
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;

        pub fn generate_differential_visualization(
            before_file: &PathBuf,
            after_file: &PathBuf,
            output_path: &str,
            config: super::VisualizationConfig,
            before_name: &str,
            after_name: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            // Generate manual differential by computing differences
            let before_stacks = parse_folded_file(before_file)?;
            let after_stacks = parse_folded_file(after_file)?;

            let diff_stacks = compute_stack_diff(&before_stacks, &after_stacks)?;

            if diff_stacks.is_empty() {
                return Err("No differential data to visualize".into());
            }

            // Create temporary differential file
            let temp_path = std::env::temp_dir().join("temp_differential.folded");
            let mut temp_file = File::create(&temp_path)?;

            for (stack, count) in diff_stacks {
                writeln!(temp_file, "{} {}", stack, count)?;
            }
            temp_file.flush()?;
            drop(temp_file);

            // Generate flamegraph from differential data
            let diff_content = std::fs::read_to_string(&temp_path)?;
            let stacks: Vec<String> = diff_content.lines().map(|line| line.to_string()).collect();

            let mut opts = Options::default();
            opts.title = config.title;
            opts.subtitle = config.subtitle;
            opts.colors = config.palette;
            opts.count_name = config.count_name;
            opts.min_width = config.min_width;
            opts.flame_chart = config.flame_chart;

            let output_file = File::create(output_path)?;
            inferno::flamegraph::from_lines(
                &mut opts,
                stacks.iter().map(String::as_str),
                output_file,
            )?;

            enhance_svg_accessibility(output_path)?;

            // Clean up temp file
            let _ = std::fs::remove_file(&temp_path);

            Ok(())
        }

        fn parse_folded_file(
            file_path: &PathBuf,
        ) -> Result<HashMap<String, i64>, Box<dyn std::error::Error>> {
            let content = std::fs::read_to_string(file_path)?;
            let mut stacks = HashMap::new();

            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let stack = parts[0].to_string();
                    let count: i64 = parts[1].parse().unwrap_or(0);
                    *stacks.entry(stack).or_insert(0) += count;
                }
            }

            Ok(stacks)
        }

        fn compute_stack_diff(
            before: &HashMap<String, i64>,
            after: &HashMap<String, i64>,
        ) -> Result<Vec<(String, i64)>, Box<dyn std::error::Error>> {
            let mut diff_stacks = Vec::new();
            let mut all_stacks = std::collections::HashSet::new();

            // Collect all unique stacks
            for stack in before.keys() {
                all_stacks.insert(stack.clone());
            }
            for stack in after.keys() {
                all_stacks.insert(stack.clone());
            }

            // Calculate differences
            for stack in all_stacks {
                let before_count = before.get(&stack).copied().unwrap_or(0);
                let after_count = after.get(&stack).copied().unwrap_or(0);
                let diff = after_count - before_count;

                // Only include stacks with significant differences
                if diff != 0 {
                    diff_stacks.push((stack, diff));
                }
            }

            // Sort by absolute difference (largest changes first)
            diff_stacks.sort_by(|a, b| b.1.abs().cmp(&a.1.abs()));

            Ok(diff_stacks)
        }
    }

    pub mod profile_analysis {
        use super::*;

        #[derive(Debug, Clone)]
        pub struct ProfileAnalysis {
            pub total_duration_us: u128,
            pub function_times: Vec<(String, u128)>,
            pub top_functions: Vec<(String, u128, f64)>,
            pub insights: Vec<String>,
        }

        #[derive(Debug, Clone)]
        pub struct DifferentialAnalysis {
            pub before_analysis: ProfileAnalysis,
            pub after_analysis: ProfileAnalysis,
            pub improvements: Vec<(String, i128, f64)>,
            pub regressions: Vec<(String, i128, f64)>,
            pub new_functions: Vec<(String, u128)>,
            pub removed_functions: Vec<(String, u128)>,
            pub summary: String,
        }

        pub fn analyze_profile(
            file_path: &PathBuf,
        ) -> Result<ProfileAnalysis, Box<dyn std::error::Error>> {
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

            let insights = vec!["Analysis completed".to_string()];

            Ok(ProfileAnalysis {
                total_duration_us,
                function_times: functions,
                top_functions,
                insights,
            })
        }

        pub fn analyze_differential(
            before_file: &PathBuf,
            after_file: &PathBuf,
        ) -> Result<DifferentialAnalysis, Box<dyn std::error::Error>> {
            let before_analysis = analyze_profile(before_file)?;
            let after_analysis = analyze_profile(after_file)?;

            let before_map: HashMap<String, u128> = before_analysis
                .function_times
                .iter()
                .map(|(name, time)| (name.clone(), *time))
                .collect();

            let after_map: HashMap<String, u128> = after_analysis
                .function_times
                .iter()
                .map(|(name, time)| (name.clone(), *time))
                .collect();

            let mut improvements = Vec::new();
            let mut regressions = Vec::new();
            let mut new_functions = Vec::new();
            let mut removed_functions = Vec::new();

            let mut all_functions = std::collections::HashSet::new();
            for name in before_map.keys() {
                all_functions.insert(name.clone());
            }
            for name in after_map.keys() {
                all_functions.insert(name.clone());
            }

            for func_name in all_functions {
                let before_time = before_map.get(&func_name).copied().unwrap_or(0);
                let after_time = after_map.get(&func_name).copied().unwrap_or(0);

                match (before_time, after_time) {
                    (0, after) if after > 0 => {
                        new_functions.push((func_name, after));
                    }
                    (before, 0) if before > 0 => {
                        removed_functions.push((func_name, before));
                    }
                    (before, after) if before > 0 && after > 0 => {
                        let time_diff = after as i128 - before as i128;
                        if time_diff != 0 {
                            let percentage_change = (time_diff as f64 / before as f64) * 100.0;

                            if time_diff < 0 {
                                improvements.push((func_name, time_diff, percentage_change));
                            } else {
                                regressions.push((func_name, time_diff, percentage_change));
                            }
                        }
                    }
                    _ => {}
                }
            }

            improvements.sort_by(|a, b| a.1.cmp(&b.1));
            regressions.sort_by(|a, b| b.1.cmp(&a.1));
            new_functions.sort_by(|a, b| b.1.cmp(&a.1));
            removed_functions.sort_by(|a, b| b.1.cmp(&a.1));

            let summary = generate_differential_summary(
                &before_analysis,
                &after_analysis,
                &improvements,
                &regressions,
            );

            Ok(DifferentialAnalysis {
                before_analysis,
                after_analysis,
                improvements,
                regressions,
                new_functions,
                removed_functions,
                summary,
            })
        }

        pub fn display_differential_analysis(analysis: &DifferentialAnalysis) {
            println!("📊 Differential Analysis Results");
            println!("═══════════════════════════════════");
            println!();

            println!("📈 Performance Summary:");
            println!("──────────────────────");
            println!("{}", analysis.summary);
            println!();

            if !analysis.improvements.is_empty() {
                println!("🚀 Top Improvements (faster):");
                println!("────────────────────────────");
                for (i, (name, time_diff, percentage)) in
                    analysis.improvements.iter().enumerate().take(5)
                {
                    let time_saved_ms = (-time_diff) as f64 / 1000.0;
                    println!(
                        "{}. {} - {:.3}ms saved ({:.1}% faster)",
                        i + 1,
                        name,
                        time_saved_ms,
                        -percentage
                    );
                }
                println!();
            }

            if !analysis.regressions.is_empty() {
                println!("🐌 Top Regressions (slower):");
                println!("───────────────────────────");
                for (i, (name, time_diff, percentage)) in
                    analysis.regressions.iter().enumerate().take(5)
                {
                    let time_added_ms = *time_diff as f64 / 1000.0;
                    println!(
                        "{}. {} - {:.3}ms slower ({:.1}% slower)",
                        i + 1,
                        name,
                        time_added_ms,
                        percentage
                    );
                }
                println!();
            }
        }

        fn clean_function_name(name: &str) -> String {
            let clean = name.split("::").last().unwrap_or(name);
            let clean = clean.split('<').next().unwrap_or(clean);
            let clean = clean.split('(').next().unwrap_or(clean);

            match clean {
                s if s.contains("sort") => "sort".to_string(),
                s if s.contains("string_concat") => "string_concat".to_string(),
                s if s.contains("lookup") => "lookup".to_string(),
                s => s.to_string(),
            }
        }

        fn generate_differential_summary(
            before: &ProfileAnalysis,
            after: &ProfileAnalysis,
            improvements: &[(String, i128, f64)],
            regressions: &[(String, i128, f64)],
        ) -> String {
            let before_total_ms = before.total_duration_us as f64 / 1000.0;
            let after_total_ms = after.total_duration_us as f64 / 1000.0;
            let total_change_ms = after_total_ms - before_total_ms;
            let total_change_percent = (total_change_ms / before_total_ms) * 100.0;

            let mut summary = String::new();

            if total_change_ms < 0.0 {
                summary.push_str(&format!(
                    "🎉 Overall Performance Improved by {:.3}ms ({:.1}% faster)\n",
                    -total_change_ms, -total_change_percent
                ));
            } else if total_change_ms > 0.0 {
                summary.push_str(&format!(
                    "⚠️  Overall Performance Declined by {:.3}ms ({:.1}% slower)\n",
                    total_change_ms, total_change_percent
                ));
            } else {
                summary.push_str("➡️  Overall Performance Unchanged\n");
            }

            summary.push_str(&format!(
                "📏 Before: {:.3}ms | After: {:.3}ms\n",
                before_total_ms, after_total_ms
            ));

            summary.push_str(&format!(
                "📈 {} improvements, {} regressions",
                improvements.len(),
                regressions.len()
            ));

            summary
        }
    }

    #[derive(Debug, Clone)]
    pub struct VisualizationConfig {
        pub title: String,
        pub subtitle: Option<String>,
        pub palette: Palette,
        pub count_name: String,
        pub min_width: f64,
        pub flame_chart: bool,
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

    #[derive(Debug, Clone)]
    pub enum AnalysisType {
        Single,
        Differential {
            before_name: String,
            after_name: String,
        },
    }

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
}

use visualization::*;

fn main() {
    println!("🔥 Differential Comparison Demo");
    println!("===============================");
    println!();

    println!("This demo will:");
    println!("1. Run the inefficient 'before' version");
    println!("2. Run the efficient 'after' version");
    println!("3. Generate differential analysis comparing both");
    println!();

    // Step 1: Run the before version
    println!("🔄 Step 1: Running BEFORE version (inefficient implementations)...");
    println!("────────────────────────────────────────────────────────────────");

    if let Err(e) = run_before_version() {
        eprintln!("❌ Failed to run before version: {}", e);
        return;
    }

    // Short pause to ensure profile files are written
    thread::sleep(Duration::from_secs(1));

    // Step 2: Run the after version
    println!();
    println!("🚀 Step 2: Running AFTER version (efficient implementations)...");
    println!("───────────────────────────────────────────────────────────────");

    if let Err(e) = run_after_version() {
        eprintln!("❌ Failed to run after version: {}", e);
        return;
    }

    // Short pause to ensure profile files are written
    thread::sleep(Duration::from_secs(1));

    // Step 3: Generate differential analysis
    println!();
    println!("📊 Step 3: Generating differential analysis...");
    println!("─────────────────────────────────────────────");

    if let Err(e) = generate_differential_analysis() {
        eprintln!("❌ Failed to generate differential analysis: {}", e);
        return;
    }

    println!();
    println!("✅ Differential comparison demo completed!");
    println!("📈 The results should show dramatic improvements in:");
    println!("   • Sorting: O(n²) → O(n log n)");
    println!("   • String concatenation: Multiple reallocations → Pre-allocated");
    println!("   • Lookup operations: O(n) → O(1)");
    println!();

    // Offer interactive visualization
    show_interactive_differential_prompt();
}

fn run_before_version() -> Result<(), Box<dyn std::error::Error>> {
    let before_script = r#"/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Comparison demo - BEFORE version with inefficient implementations
use std::collections::HashMap;
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn sort(mut arr: Vec<i32>) -> Vec<i32> {
    let n = arr.len();
    for i in 0..n {
        for j in 0..n - 1 - i {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
            }
        }
    }
    arr
}

#[profiled]
fn string_concat(words: &[&str]) -> String {
    let mut result = String::new();
    for word in words {
        result = result + word + " ";
    }
    result
}

#[profiled]
fn lookup(data: &[(String, i32)], key: &str) -> Option<i32> {
    for (k, v) in data {
        if k == key {
            return Some(*v);
        }
    }
    None
}

#[profiled]
fn run_all_tests() {
    let test_data: Vec<i32> = (0..1000).rev().collect();
    let _sorted = sort(test_data);

    let words = vec!["hello", "world", "test"];
    let test_words: Vec<&str> = words.iter().cycle().take(1000).copied().collect();
    let _result = string_concat(&test_words);

    let mut vector_data = Vec::new();
    for i in 0..1000 {
        vector_data.push((format!("key_{}", i), i * 2));
    }
    for _ in 0..100 {
        let _result = lookup(&vector_data, "key_500");
    }
}

#[enable_profiling(time)]
fn main() {
    println!("Running BEFORE version (inefficient)...");
    run_all_tests();
    println!("Before version completed!");
}
"#;

    // Create temporary file for before script - use same name for both runs
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("thag_demo_differential_comparison.rs");
    std::fs::write(&script_path, before_script)?;

    // Run the before script using thag command
    let output = Command::new("thag")
        .arg(script_path.to_str().unwrap())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Before script failed with exit code: {}\nStdout: {}\nStderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        )
        .into());
    }

    println!("✅ Before version completed successfully");

    // Wait a moment to ensure all profile files are written
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

fn run_after_version() -> Result<(), Box<dyn std::error::Error>> {
    let after_script = r#"/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Comparison demo - AFTER version with efficient implementations
use std::collections::HashMap;
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn sort(arr: Vec<i32>) -> Vec<i32> {
    quicksort(arr)
}

fn quicksort(mut arr: Vec<i32>) -> Vec<i32> {
    if arr.len() <= 1 {
        return arr;
    }
    let pivot = arr.len() / 2;
    let pivot_value = arr[pivot];
    let len = arr.len();
    arr.swap(pivot, len - 1);

    let mut i = 0;
    for j in 0..len - 1 {
        if arr[j] < pivot_value {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, len - 1);

    let (left, right) = arr.split_at_mut(i);
    let (pivot_slice, right) = right.split_at_mut(1);

    let mut left_sorted = quicksort(left.to_vec());
    let mut right_sorted = quicksort(right.to_vec());

    left_sorted.extend_from_slice(pivot_slice);
    left_sorted.extend_from_slice(&right_sorted);
    left_sorted
}

#[profiled]
fn string_concat(words: &[&str]) -> String {
    let mut result = String::with_capacity(words.len() * 10);
    for word in words {
        result.push_str(word);
        result.push(' ');
    }
    result
}

#[profiled]
fn lookup(data: &HashMap<String, i32>, key: &str) -> Option<i32> {
    data.get(key).copied()
}

#[profiled]
fn run_all_tests() {
    let test_data: Vec<i32> = (0..1000).rev().collect();
    let _sorted = sort(test_data);

    let words = vec!["hello", "world", "test"];
    let test_words: Vec<&str> = words.iter().cycle().take(1000).copied().collect();
    let _result = string_concat(&test_words);

    let mut hashmap_data = HashMap::new();
    for i in 0..1000 {
        hashmap_data.insert(format!("key_{}", i), i * 2);
    }
    for _ in 0..100 {
        let _result = lookup(&hashmap_data, "key_500");
    }
}

#[enable_profiling(time)]
fn main() {
    println!("Running AFTER version (efficient)...");
    run_all_tests();
    println!("After version completed!");
}
"#;

    // Create temporary file for after script - use same name for both runs
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("thag_demo_differential_comparison.rs");
    std::fs::write(&script_path, after_script)?;

    // Run the after script using thag command
    let output = Command::new("thag")
        .arg(script_path.to_str().unwrap())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "After script failed with exit code: {}\nStdout: {}\nStderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        )
        .into());
    }

    println!("✅ After version completed successfully");

    // Wait a moment to ensure all profile files are written
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

fn generate_differential_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // Find the most recent profile files - they should have the same base name now
    let all_files = find_latest_profile_files("thag_demo_differential_comparison", 10)?;

    // Separate exclusive files (for differential comparison)
    let mut exclusive_files = Vec::new();

    for file in all_files {
        if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
            if !name.contains("inclusive") && name.ends_with(".folded") {
                exclusive_files.push(file);
            }
        }
    }

    println!(
        "📁 Found {} exclusive profile files:",
        exclusive_files.len()
    );

    for (i, file) in exclusive_files.iter().enumerate() {
        println!("  {}. {}", i + 1, file.display());
    }

    if exclusive_files.len() < 2 {
        println!("❌ Error: Need at least 2 exclusive profile files for differential analysis");
        println!("💡 This usually means one of the runs didn't complete properly");
        return Err("Need at least 2 exclusive profile files for differential analysis".into());
    }

    // Sort by modification time, most recent first
    exclusive_files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    // Use the two most recent files as after (newest) and before (second newest)
    let after_file = &exclusive_files[0];
    let before_file = &exclusive_files[1];

    println!("📋 Found profile files:");
    println!("  Before: {}", before_file.display());
    println!("  After:  {}", after_file.display());

    // Perform text-based analysis
    println!();
    println!("📊 Performing differential analysis...");

    let differential_analysis =
        visualization::profile_analysis::analyze_differential(before_file, after_file)?;

    println!();
    visualization::profile_analysis::display_differential_analysis(&differential_analysis);

    Ok(())
}

fn generate_and_show_differential_flamegraph() -> Result<(), Box<dyn std::error::Error>> {
    // Find the profile files again
    let all_files = find_latest_profile_files("thag_demo_differential_comparison", 10)?;

    // Separate exclusive files (for differential comparison)
    let mut exclusive_files = Vec::new();

    for file in all_files {
        if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
            if !name.contains("inclusive") && name.ends_with(".folded") {
                exclusive_files.push(file);
            }
        }
    }

    if exclusive_files.len() < 2 {
        println!("❌ Error: Could not find both before and after profile files");
        println!(
            "💡 Found {} exclusive files, need at least 2",
            exclusive_files.len()
        );
        return Err("Could not find both before and after profile files".into());
    }

    // Sort by modification time, most recent first
    exclusive_files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    // Use the two most recent files as after (newest) and before (second newest)
    let after_file = &exclusive_files[0];
    let before_file = &exclusive_files[1];

    eprintln!("before_file={before_file:?}; after_file={after_file:?}");

    let config = VisualizationConfig {
        title: "Differential Comparison Demo - Performance Improvements".to_string(),
        subtitle: Some(format!(
            "Differential Analysis: Inefficient vs Efficient | Generated: {} | Red=Slower, Blue=Faster",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )),
        palette: inferno::flamegraph::color::Palette::Multi(inferno::flamegraph::color::MultiPalette::Java),
        ..Default::default()
    };

    let output_path = "differential_comparison.svg";

    // Generate differential visualization
    visualization::differential::generate_differential_visualization(
        before_file,
        after_file,
        output_path,
        config,
        "inefficient",
        "efficient",
    )?;

    println!("✅ Differential flamegraph generated: {}", output_path);

    // Try to open in browser
    if let Err(e) = open_in_browser(output_path) {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("💡 You can manually open: {}", output_path);
    } else {
        println!("🌐 Differential flamegraph opened in your default browser!");
        println!(
            "🔍 Red bars show functions that got slower, blue bars show functions that got faster"
        );
        println!("📊 Bar width represents the performance difference magnitude");
        println!("💡 You should see dramatic improvements in sorting, string ops, and lookups!");
    }

    Ok(())
}

fn create_exclusive_from_inclusive(
    inclusive_file: &std::path::PathBuf,
    exclusive_file: &std::path::PathBuf,
) -> Result<bool, Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};

    let file = File::open(inclusive_file)?;
    let reader = BufReader::new(file);

    let mut stack_times: HashMap<String, u64> = HashMap::new();

    // Parse the inclusive file and disaggregate
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let stack = parts[0];
        let time: u64 = parts[1].parse().unwrap_or(0);

        // For each stack, add the time to itself and subtract from all ancestors
        let stack_parts: Vec<&str> = stack.split(';').collect();

        for i in 0..stack_parts.len() {
            let current_stack = stack_parts[0..=i].join(";");

            if i == stack_parts.len() - 1 {
                // This is the leaf stack - add the time
                *stack_times.entry(current_stack).or_insert(0) += time;
            } else {
                // This is an ancestor stack - subtract the time
                let current_time = stack_times.entry(current_stack.clone()).or_insert(0);
                *current_time = current_time.saturating_sub(time);
            }
        }
    }

    // Write the exclusive file
    let mut output = File::create(exclusive_file)?;
    let mut entries: Vec<_> = stack_times.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by time descending

    for (stack, time) in entries {
        if time > 0 {
            writeln!(output, "{} {}", stack, time)?;
        }
    }

    Ok(true)
}

fn open_in_browser(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

fn show_interactive_differential_prompt() {
    println!("🎯 Would you like to view an interactive differential flamegraph?");
    println!("This will generate a visual comparison showing performance improvements.");
    print!("Enter 'y' for yes, or any other key to skip: ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        if input.trim().to_lowercase() == "y" {
            println!();
            println!("🔥 Generating interactive differential flamegraph...");

            match generate_and_show_differential_flamegraph() {
                Ok(()) => {
                    println!("✅ Differential flamegraph generated successfully!");
                }
                Err(e) => {
                    println!("⚠️  Could not generate differential flamegraph: {}", e);
                    println!("💡 You can still view the text-based analysis above.");
                }
            }
        }
    }
}
