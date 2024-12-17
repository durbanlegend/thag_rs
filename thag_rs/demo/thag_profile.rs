/*[toml]
[dependencies]
inferno = "0.12.0"
inquire = "0.7.5"
serde = { version = "1.0.216", features = ["derive"] }
thag_core = { path = "/Users/donf/projects/thag_rs/thag_core" }
*/

use inferno::flamegraph::{self, color::BasicPalette, Options, Palette};
use inquire::{MultiSelect, Select};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use thag_core::profiling::ProfileStats;

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
                // The stack is already in correct leaf->root order
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
        "Generate Flamegraph",
        "Show Statistics",
        "Filter Functions",
        "Show Async Boundaries",
    ];

    let selection = Select::new("Select action:", options).prompt()?;

    match selection {
        "Generate Flamegraph" => generate_flamegraph(&processed.filtered_stacks)?,
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

    // Create formatted stacks with their original indices to maintain order
    let mut formatted_with_index: Vec<(usize, String)> = stacks
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            let (stack_part, time_str) = line.rsplit_once(' ')?;
            let parts: Vec<&str> = stack_part.split(';').collect();
            let reversed: Vec<&str> = parts.iter().rev().copied().collect();
            Some((idx, format!("{} {}", reversed.join(";"), time_str)))
        })
        .collect();

    // Sort by original index to maintain chronological order
    formatted_with_index.sort_by_key(|(idx, _)| *idx);

    // Extract just the formatted strings
    let formatted_stacks: Vec<String> = formatted_with_index
        .into_iter()
        .map(|(_, stack)| stack)
        .collect();

    println!("\nChronologically ordered stacks:");
    for stack in &formatted_stacks {
        println!("{}", stack);
    }

    flamegraph::from_lines(
        &mut opts,
        formatted_stacks.iter().map(String::as_str),
        output,
    )?;

    println!("Flamegraph generated: flamegraph.svg");
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

    // Convert to vec for sorting
    let mut entries: Vec<_> = stats.calls.iter().collect();
    entries.sort_by_key(|(_, &calls)| std::cmp::Reverse(calls));

    for (func, &calls) in entries {
        let total_time = stats.total_time.get(func).unwrap_or(&0);
        let avg_time = if calls > 0 { total_time / calls } else { 0 };
        println!(
            "{}{}: {} calls, {} μs total, {} μs avg",
            func,
            if stats.is_async_boundary(func) {
                " (async)"
            } else {
                ""
            },
            calls,
            total_time,
            avg_time
        );
    }

    // Show aggregate statistics
    println!("\nAggregate Statistics:");
    println!("Total calls: {}", stats.count());
    println!("Total duration: {:?}", stats.total_duration());
    if let Some(min) = stats.min_time() {
        println!("Minimum duration: {:?}", min);
    }
    if let Some(max) = stats.max_time() {
        println!("Maximum duration: {:?}", max);
    }
    if let Some(avg) = stats.average() {
        println!("Average duration: {:?}", avg);
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
