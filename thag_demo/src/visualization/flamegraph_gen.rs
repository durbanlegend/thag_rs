//! Flamegraph generation utilities for thag_demo profiling analysis

use crate::visualization::VisualizationConfig;
use inferno::flamegraph::{self, Options};
use std::fs::File;
use thag_profiler::enhance_svg_accessibility;

/// Generate a flamegraph SVG from folded stack data
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
    let output = File::create(output_path)?;

    flamegraph::from_lines(&mut opts, stacks.iter().rev().map(String::as_str), output)?;

    // Enhance accessibility
    enhance_svg_accessibility(output_path)?;

    Ok(())
}

/// Generate a flamegraph from a single folded file
pub fn generate_flamegraph_from_file(
    folded_file: &std::path::Path,
    output_path: &str,
    config: VisualizationConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(folded_file)?;
    let stacks: Vec<String> = content.lines().map(|line| line.to_string()).collect();

    generate_flamegraph_svg(&stacks, output_path, config)
}

/// Generate both flamegraph and flamechart versions
pub fn generate_both_visualizations(
    stacks: &[String],
    base_output_path: &str,
    mut config: VisualizationConfig,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Generate flamegraph (timeline view)
    config.flame_chart = false;
    let flamegraph_path = format!("{}_flamegraph.svg", base_output_path);
    generate_flamegraph_svg(stacks, &flamegraph_path, config.clone())?;

    // Generate flamechart (aggregated view)
    config.flame_chart = true;
    let flamechart_path = format!("{}_flamechart.svg", base_output_path);
    generate_flamegraph_svg(stacks, &flamechart_path, config)?;

    Ok((flamegraph_path, flamechart_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_generate_flamegraph_svg_empty() {
        let stacks = vec![];
        let config = VisualizationConfig::default();
        let result = generate_flamegraph_svg(&stacks, "test.svg", config);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_flamegraph_svg_valid() {
        let stacks = vec![
            "main;foo;bar 100".to_string(),
            "main;foo;baz 200".to_string(),
        ];
        let config = VisualizationConfig::default();
        let temp_path = std::env::temp_dir().join("test_flamegraph.svg");
        let result = generate_flamegraph_svg(&stacks, temp_path.to_str().unwrap(), config);

        if result.is_ok() {
            assert!(temp_path.exists());
            // Clean up
            let _ = std::fs::remove_file(&temp_path);
        }
    }
}
