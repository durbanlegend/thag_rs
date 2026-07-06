/*[toml]
[dependencies]
thag_profiler = { version = "1, thag-auto", features = ["time_profiling"] }
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
use std::process::{exit, Command};

// Inline visualization functionality for this demo
mod visualization {
    use chrono::Local;
    use inferno::flamegraph::{color::MultiPalette, Palette};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use thag_profiler::enhance_svg_accessibility;

    pub mod differential {
        use super::*;
        use inferno::flamegraph::Options;
        use std::fs::File;
        // use std::io::Write;
        use std::path::PathBuf;
        use thag_profiler::ProfileError;

        pub fn generate_differential_visualization(
            before_file: &PathBuf,
            after_file: &PathBuf,
            output_path: &str,
            config: super::VisualizationConfig,
        ) -> Result<(), Box<dyn std::error::Error>> {
            // First, generate the differential data
            let mut diff_data = Vec::new();
            inferno::differential::from_files(
                inferno::differential::Options::default(), // Options for differential processing
                before_file,
                after_file,
                &mut diff_data,
            )
            .map_err(|e| ProfileError::General(e.to_string()))?;

            let mut opts = Options::default();
            opts.title = config.title;
            opts.subtitle = config.subtitle;
            opts.colors = config.palette;
            opts.count_name = config.count_name;
            opts.min_width = config.min_width;
            opts.flame_chart = config.flame_chart;

            // Convert diff_data to lines
            let diff_lines =
                String::from_utf8(diff_data).map_err(|e| ProfileError::General(e.to_string()))?;
            let lines: Vec<&str> = diff_lines.lines().collect();

            let output_file = File::create(output_path)?;
            inferno::flamegraph::from_lines(&mut opts, lines.iter().copied(), output_file)?;

            enhance_svg_accessibility(output_path)?;

            // // Clean up temp file
            // let _ = std::fs::remove_file(&temp_path);

            Ok(())
        }
    }

    pub mod profile_analysis {
        use super::*;

        #[derive(Debug, Clone)]
        pub struct ProfileAnalysis {
            pub total_duration_us: u128,
            pub function_times: Vec<(String, u128)>,
            // pub top_functions: Vec<(String, u128, f64)>,
            // pub insights: Vec<String>,
        }

        #[derive(Debug, Clone)]
        pub struct DifferentialAnalysis {
            // pub before_analysis: ProfileAnalysis,
            // pub after_analysis: ProfileAnalysis,
            pub improvements: Vec<(String, i128, f64)>,
            pub regressions: Vec<(String, i128, f64)>,
            // pub new_functions: Vec<(String, u128)>,
            // pub removed_functions: Vec<(String, u128)>,
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

            Ok(ProfileAnalysis {
                total_duration_us,
                function_times: functions,
                // top_functions,
                // insights,
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
                // before_analysis,
                // after_analysis,
                improvements,
                regressions,
                // new_functions,
                // removed_functions,
                summary,
            })
        }

        pub fn display_differential_analysis(analysis: &DifferentialAnalysis) {
            println!("📊 Differential Analysis Results");
            println!("════════════════════════════════");
            println!();

            println!("📈 Performance Summary:");
            println!("{}", "─".repeat(23));
            println!("{}", analysis.summary);
            println!();

            if !analysis.improvements.is_empty() {
                println!("🚀 Top Improvements (faster):");
                println!("{}", "─".repeat(29));
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
                println!("{}", "─".repeat(28));
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
        // pub open_browser: bool,
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
                // open_browser: false,
            }
        }
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
}

use visualization::*;

fn main() {
    println!("🔥 Differential Comparison Demo");
    println!("{}", "═".repeat(31));
    println!();

    println!("This demo will:");
    println!("1. Run the inefficient 'before' version");
    println!("2. Run the efficient 'after' version");
    println!("3. Generate differential analysis comparing both");
    println!();

    // Step 1: Run the before version
    println!("🔄 Step 1: Running BEFORE version (inefficient implementations)...");
    println!("──────────────────────────────────────────────────────────────────");

    if let Err(e) = run_before_version() {
        eprintln!("❌ Failed to run before version: {}", e);
        exit(1);
    }

    // // Short pause to ensure profile files are written
    // thread::sleep(Duration::from_secs(1));

    // Step 2: Run the after version
    println!();
    println!("🚀 Step 2: Running AFTER version (efficient implementations)...");
    println!("───────────────────────────────────────────────────────────────");

    if let Err(e) = run_after_version() {
        eprintln!("❌ Failed to run after version: {}", e);
        exit(1);
    }

    // // Short pause to ensure profile files are written
    // thread::sleep(Duration::from_secs(1));

    // Step 3: Generate differential analysis
    println!();
    println!("📊 Step 3: Generating differential analysis...");
    println!("─────────────────────────────────────────────");

    if let Err(e) = generate_differential_analysis() {
        eprintln!("❌ Failed to generate differential analysis: {}", e);
        exit(1);
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
thag_profiler = { version = "1, thag-auto", features = ["time_profiling", "demo"] }

[profile.release]
debug = true
strip = false
*/

/// Comparison demo - BEFORE version with inefficient implementations
use thag_profiler::{enable_profiling, profiled};

// Not profiled directly - wrapped by demonstrate_sorting to avoid recursion detection
#[profiled]
fn sort(mut arr: Vec<i32>) -> Vec<i32> {
    // Inefficient bubble sort - O(n²)
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
fn string_concat() {
    let words = vec!["hello", "world", "test"];
    let test_words: Vec<&str> = words.iter().cycle().take(1000).copied().collect();
    for _ in 0..100 {
        let mut result = String::new();
        for word in &test_words {
            result = result + word + " ";
        }
        let _result = result;
    }
}

#[profiled]
fn lookup() {
    let mut vector_data: Vec<(String, i32)> = Vec::new();
    for i in 0..1000 {
        vector_data.push((format!("key_{}", i), i * 2));
    }
    for _ in 0..1000 {
        let _result = vector_data.iter().find(|(k, _)| k == "key_500").map(|(_, v)| *v);
    }
}

fn run_all_tests() {
    let test_data: Vec<i32> = (0..5000).rev().collect();
    let _sorted = sort(test_data);
    string_concat();
    lookup();
}

#[enable_profiling(time)]
fn main() {
    println!("Running BEFORE version (inefficient)...");
    run_all_tests();
    println!("Before version completed!");
}
"#;

    // Distinct from the outer demo script; same stem as after script so thag_profile
    // can group the two runs together for differential comparison
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("thag_diff_comparison.rs");
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");
    println!("✅ Before version completed successfully");

    // // Wait a moment to ensure all profile files are written
    // thread::sleep(Duration::from_secs(2));

    Ok(())
}

fn run_after_version() -> Result<(), Box<dyn std::error::Error>> {
    let after_script = r#"/*[toml]
[dependencies]
thag_profiler = { version = "1, thag-auto", features = ["time_profiling", "demo"] }

[profile.release]
debug = true
strip = false
*/

/// Comparison demo - AFTER version with efficient implementations
use std::collections::HashMap;
use thag_profiler::{enable_profiling, profiled};

// Not profiled directly - wrapped by demonstrate_sorting to avoid recursion detection
#[profiled]
fn sort(mut arr: Vec<i32>) -> Vec<i32> {
    // Rust stdlib introsort - O(n log n), in-place, no extra allocations
    arr.sort_unstable();
    arr
}

#[profiled]
fn string_concat() {
    let words = vec!["hello", "world", "test"];
    let test_words: Vec<&str> = words.iter().cycle().take(1000).copied().collect();
    for _ in 0..100 {
        let mut result = String::with_capacity(test_words.len() * 10);
        for word in &test_words {
            result.push_str(word);
            result.push(' ');
        }
        let _result = result;
    }
}

#[profiled]
fn lookup() {
    let mut hashmap_data: HashMap<String, i32> = HashMap::new();
    for i in 0..1000 {
        hashmap_data.insert(format!("key_{}", i), i * 2);
    }
    for _ in 0..1000 {
        let _result = hashmap_data.get("key_500").copied();
    }
}

fn run_all_tests() {
    let test_data: Vec<i32> = (0..5000).rev().collect();
    let _sorted = sort(test_data);
    string_concat();
    lookup();
}

#[enable_profiling(time)]
fn main() {
    println!("Running AFTER version (efficient)...");
    run_all_tests();
    println!("After version completed!");
}
"#;

    // Same stem as before script so thag_profile can group the two runs for differential
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("thag_diff_comparison.rs");
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");
    println!("✅ After version completed successfully");

    // // Wait a moment to ensure all profile files are written
    // thread::sleep(Duration::from_secs(2));

    Ok(())
}

fn generate_differential_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // Find the most recent profile files - they should have the same base name now
    let all_files = find_latest_profile_files("thag_diff_comparison", 10)?;

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
    let all_files = find_latest_profile_files("thag_diff_comparison", 10)?;

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
        // "inefficient",
        // "efficient",
    )?;

    println!("✅ Differential flamegraph generated: {}", output_path);

    // Try to open in browser
    if let Err(e) = open_in_browser(output_path) {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("💡 You can manually open: {}", output_path);
    } else {
        println!("🌐 Differential flamegraph opened in your default browser");
        println!(
            "🔍 Red bars show functions that got slower, blue bars show functions that got faster"
        );
        println!("📊 Bar width represents the 'after' performance");
        println!("💡 You should see dramatic improvements in sorting, string ops, and lookups!");
    }

    Ok(())
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
