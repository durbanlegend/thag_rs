/*[toml]
[dependencies]
inferno = "0.12.0"
inquire = "0.7.5"
serde = { version = "1.0.216", features = ["derive"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["minimal", "simplelog"] }
*/

/// Profile graph/chart generator for the internal profiler of `thag_core`.
///
/// E.g.:
///
///```
/// thag demo/thag_profile.rs -x    # Compile this script as a command
///
/// cargo run --features thag/profile <path>/demo/time_cookbook.rs -f   # Profile a demo script
///
/// thag_profile    # Generate a flamechart or show stats for the new profile
///```
//# Purpose: Low-footprint profiling.
//# Categories: tools
use inferno::flamegraph::{self, color::BasicPalette, Options, Palette};
use inquire::{MultiSelect, Select};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use thag_rs::profiling::ProfileStats;

#[derive(Default)]
struct ProcessedProfile {
    stats: ProfileStats,
    filtered_stacks: Vec<String>,
}

fn process_profile_data(lines: &[String]) -> ProcessedProfile {
    let mut result = ProcessedProfile::default();

    for line in lines {
        if let Some((stack_part, time_str)) = line.rsplit_once(' ') {
            if let Ok(time) = time_str.parse::<u64>() {
                // The stack is already in correct order, root->leaf.
                // We don't need to reverse it
                let parts: Vec<&str> = stack_part.split(';').map(str::trim).collect();

                if !parts.is_empty() {
                    // Store original line if time > 0
                    if time > 0 {
                        result
                            .filtered_stacks
                            .push(format!("{} {}", stack_part, time));
                    }

                    // Update statistics
                    let func_name = parts[0]; // Leaf function
                    if is_async_boundary(func_name) {
                        result.stats.mark_async(func_name);
                    }
                    let duration = std::time::Duration::from_micros(time);
                    result.stats.record(func_name, duration);
                }
            }
        }
    }

    result
}

fn is_async_boundary(func_name: &str) -> bool {
    // Expanded list of async indicators
    matches!(func_name.trim(),
        "handle_build_or_check" |
        "build" |
        _ if func_name.contains("spawn") ||
            func_name.contains("wait") ||
            func_name.contains("async") ||
            func_name.contains("process") ||
            func_name.contains("command") ||
            func_name.contains("exec")
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = File::open("thag-profile.folded")?;
    let reader = BufReader::new(input);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let processed = process_profile_data(&lines);

    let options = vec![
        "Generate & Show Flamegraph",
        "Show Statistics",
        "Filter Functions",
        "Show Async Boundaries",
    ];

    let selection = Select::new("Select action:", options).prompt()?;

    match selection {
        "Generate & Show Flamegraph" => generate_flamegraph(&processed.filtered_stacks)?,
        "Show Statistics" => show_statistics(&processed.stats),
        "Filter Functions" => {
            let filtered = filter_functions(&processed.filtered_stacks)?;
            generate_flamegraph(&filtered)?;
        }
        "Show Async Boundaries" => show_async_boundaries(&processed.stats),
        _ => println!("Unknown option"),
    }

    Ok(())
}

fn generate_flamegraph(stacks: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if stacks.is_empty() {
        return Err("No profile data available".into());
    }

    let output = File::create("flamegraph.svg")?;
    let mut opts = Options::default();
    opts.title = "Thag Profile".to_string();
    opts.colors = Palette::Basic(BasicPalette::Aqua);
    opts.count_name = "μs".to_owned();
    opts.min_width = 0.1;
    opts.flame_chart = true; // Enable flame chart mode for temporal ordering

    // Just pass the stacks directly, no need for sequence numbers
    flamegraph::from_lines(&mut opts, stacks.iter().rev().map(String::as_str), output)?;

    println!("Flame chart generated: flamegraph.svg");
    open_in_browser("flamegraph.svg")?;
    Ok(())
}

fn open_in_browser(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", path]).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

fn show_statistics(stats: &ProfileStats) {
    println!("\nFunction Statistics:");
    println!("===================");

    let mut entries: Vec<_> = stats.calls.iter().collect();
    entries.sort_by_key(|(_, &calls)| std::cmp::Reverse(calls));

    for (func, &calls) in entries {
        let total_time = stats.total_time.get(func).unwrap_or(&0);
        let avg_time = if calls > 0 {
            total_time / u128::from(calls)
        } else {
            0
        };
        println!(
            "{}: {} calls, {} μs total, {} μs avg",
            func, calls, total_time, avg_time
        );
    }
}

fn filter_functions(stacks: &[String]) -> Result<Vec<String>, inquire::InquireError> {
    let functions: HashSet<_> = stacks
        .iter()
        .filter_map(|line| {
            line.split(';')
                .next()
                .map(|s| s.split_whitespace().next().unwrap_or(""))
                .filter(|s| !s.is_empty())
        })
        .collect();

    let mut function_list: Vec<_> = functions.into_iter().collect();
    function_list.sort();

    let to_filter = MultiSelect::new("Select functions to filter out:", function_list).prompt()?;

    Ok(stacks
        .iter()
        .filter(|line| {
            let func = line
                .split(';')
                .next()
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("");
            !to_filter.contains(&func)
        })
        .cloned()
        .collect())
}

fn show_async_boundaries(stats: &ProfileStats) {
    println!("\nAsync Boundaries:");
    println!("================");

    for func_name in stats.async_boundaries.iter() {
        if let Some(&calls) = stats.calls.get(func_name) {
            let total_time = stats.total_time.get(func_name).unwrap_or(&0);
            println!(
                "{}: {} calls, {} μs total (includes subprocess time)",
                func_name, calls, total_time
            );
        }
    }
}
