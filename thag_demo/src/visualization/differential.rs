//! Differential comparison utilities for thag_demo profiling analysis

use crate::visualization::VisualizationConfig;
use inferno::flamegraph::{self, Options};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use thag_profiler::enhance_svg_accessibility;

/// Generate a differential flamegraph comparing before and after profiles
pub fn generate_differential_visualization(
    before_file: &PathBuf,
    after_file: &PathBuf,
    output_path: &str,
    config: VisualizationConfig,
    before_name: &str,
    after_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // First try using inferno's built-in differential mode
    if let Ok(()) = generate_inferno_differential(before_file, after_file, output_path, &config) {
        return Ok(());
    }

    // Fallback to manual differential generation
    generate_manual_differential(
        before_file,
        after_file,
        output_path,
        config,
        before_name,
        after_name,
    )
}

/// Generate differential using inferno's built-in differential mode
fn generate_inferno_differential(
    before_file: &PathBuf,
    after_file: &PathBuf,
    output_path: &str,
    config: &VisualizationConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to use inferno command line tool for differential analysis
    let output = Command::new("inferno-diff-folded")
        .arg(before_file)
        .arg(after_file)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let diff_data = String::from_utf8(output.stdout)?;
            let stacks: Vec<String> = diff_data.lines().map(|line| line.to_string()).collect();

            if !stacks.is_empty() {
                let mut opts = Options::default();
                opts.title = config.title.clone();
                opts.subtitle = config.subtitle.clone();
                opts.colors = config.palette.clone();
                opts.count_name = config.count_name.clone();
                opts.min_width = config.min_width;
                opts.flame_chart = config.flame_chart;

                let output_file = File::create(output_path)?;
                flamegraph::from_lines(&mut opts, stacks.iter().map(String::as_str), output_file)?;

                enhance_svg_accessibility(output_path)?;
                return Ok(());
            }
        }
    }

    Err("inferno-diff-folded not available or failed".into())
}

/// Generate differential manually by computing the difference between profiles
fn generate_manual_differential(
    before_file: &PathBuf,
    after_file: &PathBuf,
    output_path: &str,
    config: VisualizationConfig,
    before_name: &str,
    after_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
    flamegraph::from_lines(&mut opts, stacks.iter().map(String::as_str), output_file)?;

    enhance_svg_accessibility(output_path)?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(())
}

/// Parse a folded file into stack -> count mapping
fn parse_folded_file(
    file_path: &PathBuf,
) -> Result<std::collections::HashMap<String, i64>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut stacks = std::collections::HashMap::new();

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

/// Compute the difference between two stack profiles
fn compute_stack_diff(
    before: &std::collections::HashMap<String, i64>,
    after: &std::collections::HashMap<String, i64>,
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

/// Generate side-by-side comparison of before and after profiles
pub fn generate_side_by_side_comparison(
    before_file: &PathBuf,
    after_file: &PathBuf,
    output_dir: &str,
    config: VisualizationConfig,
    before_name: &str,
    after_name: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Generate before flamegraph
    let before_config = VisualizationConfig {
        title: format!("{} - Before ({})", config.title, before_name),
        ..config.clone()
    };

    let before_output = format!("{}/{}_before.svg", output_dir, before_name);
    crate::visualization::flamegraph_gen::generate_flamegraph_from_file(
        before_file,
        &before_output,
        before_config,
    )?;

    // Generate after flamegraph
    let after_config = VisualizationConfig {
        title: format!("{} - After ({})", config.title, after_name),
        ..config
    };

    let after_output = format!("{}/{}_after.svg", output_dir, after_name);
    crate::visualization::flamegraph_gen::generate_flamegraph_from_file(
        after_file,
        &after_output,
        after_config,
    )?;

    Ok((before_output, after_output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_folded_data() {
        let mut stacks = HashMap::new();
        stacks.insert("main;foo;bar".to_string(), 100);
        stacks.insert("main;foo;baz".to_string(), 200);

        let temp_path = std::env::temp_dir().join("test_folded.folded");
        std::fs::write(&temp_path, "main;foo;bar 100\nmain;foo;baz 200\n").unwrap();

        let parsed = parse_folded_file(&temp_path).unwrap();
        assert_eq!(parsed.get("main;foo;bar"), Some(&100));
        assert_eq!(parsed.get("main;foo;baz"), Some(&200));

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_compute_stack_diff() {
        let mut before = HashMap::new();
        before.insert("main;foo".to_string(), 100);
        before.insert("main;bar".to_string(), 50);

        let mut after = HashMap::new();
        after.insert("main;foo".to_string(), 150);
        after.insert("main;bar".to_string(), 30);
        after.insert("main;baz".to_string(), 20);

        let diff = compute_stack_diff(&before, &after).unwrap();

        // Should have improvements and regressions
        assert!(diff
            .iter()
            .any(|(stack, count)| stack == "main;foo" && *count == 50));
        assert!(diff
            .iter()
            .any(|(stack, count)| stack == "main;bar" && *count == -20));
        assert!(diff
            .iter()
            .any(|(stack, count)| stack == "main;baz" && *count == 20));
    }
}
