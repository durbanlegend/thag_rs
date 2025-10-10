//! Profile analysis utilities for `thag_demo` profiling analysis
#![allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

/// Analysis results for a single profile
#[derive(Debug, Clone)]
pub struct ProfileAnalysis {
    pub total_duration_us: u128,
    pub function_times: Vec<(String, u128)>,
    pub top_functions: Vec<(String, u128, f64)>, // name, time, percentage
    pub insights: Vec<String>,
}

/// Analysis results for differential comparison
#[derive(Debug, Clone)]
pub struct DifferentialAnalysis {
    pub before_analysis: ProfileAnalysis,
    pub after_analysis: ProfileAnalysis,
    pub improvements: Vec<(String, i128, f64)>, // name, time_diff, percentage_change
    pub regressions: Vec<(String, i128, f64)>,
    pub new_functions: Vec<(String, u128)>,
    pub removed_functions: Vec<(String, u128)>,
    pub summary: String,
}

/// Analyze a single profile from a folded file
///
/// # Errors
///
/// Will bubble up any i/o errors encountered accessing the file.
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

    // Create sorted list of functions by execution time
    let mut functions: Vec<_> = function_times.into_iter().collect();
    functions.sort_by(|a, b| b.1.cmp(&a.1));

    // Create top functions with percentages
    let top_functions: Vec<_> = functions
        .iter()
        .take(10)
        .map(|(name, time)| {
            let percentage = (*time as f64 / total_duration_us as f64) * 100.0;
            (name.clone(), *time, percentage)
        })
        .collect();

    // Generate insights
    let insights = generate_insights(&functions /*, total_duration_us */);

    Ok(ProfileAnalysis {
        total_duration_us,
        function_times: functions,
        top_functions,
        insights,
    })
}

/// Analyze differential comparison between two profiles
///
/// # Errors
///
/// Will bubble up any i/o errors encountered analyzing the profiles.
pub fn analyze_differential(
    before_file: &PathBuf,
    after_file: &PathBuf,
) -> Result<DifferentialAnalysis, Box<dyn std::error::Error>> {
    let before_analysis = analyze_profile(before_file)?;
    let after_analysis = analyze_profile(after_file)?;

    // Convert to maps for easier comparison
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

    // Find all unique function names
    let mut all_functions = std::collections::HashSet::new();
    for name in before_map.keys() {
        all_functions.insert(name.clone());
    }
    for name in after_map.keys() {
        all_functions.insert(name.clone());
    }

    // Analyze changes
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
            _ => {} // No change or both zero
        }
    }

    // Sort by magnitude of change
    improvements.sort_by(|a, b| a.1.cmp(&b.1)); // Most improved first (most negative)
    regressions.sort_by(|a, b| b.1.cmp(&a.1)); // Most regressed first (most positive)
    new_functions.sort_by(|a, b| b.1.cmp(&a.1)); // Highest time first
    removed_functions.sort_by(|a, b| b.1.cmp(&a.1)); // Highest time first

    let summary = generate_differential_summary(
        &before_analysis,
        &after_analysis,
        &improvements,
        &regressions,
        &new_functions,
        &removed_functions,
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

/// Display profile analysis results
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
}

/// Display differential analysis results
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
        for (i, (name, time_diff, percentage)) in analysis.improvements.iter().enumerate().take(5) {
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
        for (i, (name, time_diff, percentage)) in analysis.regressions.iter().enumerate().take(5) {
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

    if !analysis.new_functions.is_empty() {
        println!("🆕 New Functions:");
        println!("───────────────");
        for (i, (name, time)) in analysis.new_functions.iter().enumerate().take(3) {
            let time_ms = *time as f64 / 1000.0;
            println!("{}. {} - {:.3}ms", i + 1, name, time_ms);
        }
        println!();
    }

    if !analysis.removed_functions.is_empty() {
        println!("🗑️  Removed Functions:");
        println!("────────────────────");
        for (i, (name, time)) in analysis.removed_functions.iter().enumerate().take(3) {
            let time_ms = *time as f64 / 1000.0;
            println!("{}. {} - {:.3}ms", i + 1, name, time_ms);
        }
        println!();
    }
}

/// Clean function name for better readability
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
            } else if s.contains("recursive") {
                "fibonacciå_recursive"
            } else {
                "fibonacci"
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

/// Generate insights from profile analysis
fn generate_insights(functions: &[(String, u128)], /* , total_duration_us: u128 */) -> Vec<String> {
    let mut insights = Vec::new();

    // if functions.len() >= 2 {
    //     let slowest = &functions[0];
    //     let fastest = &functions[functions.len() - 1];

    //     if fastest.1 > 0 {
    //         let speedup = slowest.1 as f64 / fastest.1 as f64;
    //         insights.push(format!(
    //             "🐌 Slowest: {} ({:.3}ms)",
    //             slowest.0,
    //             slowest.1 as f64 / 1000.0
    //         ));
    //         insights.push(format!(
    //             "🚀 Fastest: {} ({:.3}ms)",
    //             fastest.0,
    //             fastest.1 as f64 / 1000.0
    //         ));
    //         insights.push(format!("⚡ Performance difference: {:.1}x", speedup));

    //         if speedup > 1000.0 {
    //             insights.push("🎯 Consider using faster algorithms in production!".to_string());
    //         }
    //     }
    // }

    // Look for specific patterns
    let has_recursive = functions.iter().any(|(name, _)| name.contains("recurs"));
    let has_cached = functions.iter().any(|(name, _)| name.contains("cached"));
    let has_iter = functions.iter().any(|(name, _)| name.contains("iter"));
    let has_bubble_sort = functions
        .iter()
        .any(|(name, _)| name.contains("bubble_sort"));
    let has_quicksort = functions.iter().any(|(name, _)| name.contains("quicksort"));
    let has_naive_string = functions.iter().any(|(name, _)| name.contains("naive"));
    let has_efficient_string = functions.iter().any(|(name, _)| name.contains("efficient"));

    if has_recursive && has_cached {
        insights.push("🔧 Tip: Caching can dramatically improve recursive algorithms!".to_string());
    }

    if has_iter {
        insights.push(
            "🔄 Tip: Iterative approaches often outperform recursion for large inputs!".to_string(),
        );
    }

    if has_bubble_sort && has_quicksort {
        insights.push(
            "🏃 Tip: Quicksort (O(n log n)) vastly outperforms bubble sort (O(n²))!".to_string(),
        );
    }

    if has_naive_string && has_efficient_string {
        insights.push(
            "📝 Tip: Pre-allocating string capacity prevents repeated reallocations!".to_string(),
        );
    }

    if insights.is_empty() {
        insights.push("📊 Profile looks good! No obvious performance issues detected.".to_string());
    }

    insights
}

/// Generate summary for differential analysis
fn generate_differential_summary(
    before: &ProfileAnalysis,
    after: &ProfileAnalysis,
    improvements: &[(String, i128, f64)],
    regressions: &[(String, i128, f64)],
    new_functions: &[(String, u128)],
    removed_functions: &[(String, u128)],
) -> String {
    let before_total_ms = before.total_duration_us as f64 / 1000.0;
    let after_total_ms = after.total_duration_us as f64 / 1000.0;
    let total_change_ms = after_total_ms - before_total_ms;
    let total_change_percent = (total_change_ms / before_total_ms) * 100.0;

    let mut summary = String::new();

    if total_change_ms < 0.0 {
        let _ = writeln!(
            summary,
            "🎉 Overall Performance Improved by {:.3}ms ({:.1}% faster)\n",
            -total_change_ms, -total_change_percent
        );
    } else if total_change_ms > 0.0 {
        let _ = writeln!(
            summary,
            "⚠️  Overall Performance Declined by {:.3}ms ({:.1}% slower)\n",
            total_change_ms, total_change_percent
        );
    } else {
        summary.push_str("➡️  Overall Performance Unchanged\n");
    }

    let _ = writeln!(
        summary,
        "📏 Before: {:.3}ms | After: {:.3}ms\n",
        before_total_ms, after_total_ms
    );

    let _ = writeln!(
        summary,
        "📈 {} improvements, {} regressions, {} new functions, {} removed functions",
        improvements.len(),
        regressions.len(),
        new_functions.len(),
        removed_functions.len()
    );

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_clean_function_name() {
        assert_eq!(
            clean_function_name("thag_demo_basic_profiling::fibonacci"),
            "fibonacci"
        );
        assert_eq!(clean_function_name("std::vec::Vec<i32>::new"), "new");
        assert_eq!(
            clean_function_name("main::fibonacci_cached"),
            "fibonacci_cached"
        );
        assert_eq!(
            clean_function_name("bubble_sort_inefficient"),
            "bubble_sort"
        );
        assert_eq!(clean_function_name("quicksort_efficient"), "quicksort");
    }

    #[test]
    fn test_generate_insights() {
        let functions = vec![
            ("fibonacci_recursive".to_string(), 1000000),
            ("fibonacci_cached".to_string(), 1000),
            ("cpu_work".to_string(), 500),
        ];

        let insights = generate_insights(&functions /*, 1001500 */);
        assert!(!insights.is_empty());
        assert!(insights.iter().any(|s| s.contains("Caching")));
    }

    #[test]
    fn test_analyze_profile() {
        let temp_path = std::env::temp_dir().join("test_profile.folded");
        fs::write(
            &temp_path,
            "main;foo;bar 100\nmain;foo;baz 200\nmain;qux 50\n",
        )
        .unwrap();

        let analysis = analyze_profile(&temp_path).unwrap();
        assert_eq!(analysis.total_duration_us, 350);
        assert!(!analysis.function_times.is_empty());
        assert!(!analysis.top_functions.is_empty());

        let _ = fs::remove_file(&temp_path);
    }
}
