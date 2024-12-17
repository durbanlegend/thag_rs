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
use thag_core::profiling::ProfileStats;

#[derive(Default)]
struct ProcessedProfile {
    stats: ProfileStats,
    filtered_stacks: Vec<String>,
}

fn is_async_boundary(func_name: &str) -> bool {
    // Known async boundaries
    matches!(func_name,
        "handle_build_or_check" |
        "build" |
        // Add other known async boundaries
        _ if func_name.contains("spawn") ||
            func_name.contains("wait") ||
            func_name.contains("async")
    )
}

fn process_profile_data(lines: &[String]) -> ProcessedProfile {
    let mut result = ProcessedProfile::default();

    for line in lines {
        if let Some((stack_part, time_str)) = line.rsplit_once(' ') {
            if let Ok(time) = time_str.parse::<u64>() {
                let parts: Vec<&str> = stack_part.split(';').collect();
                let func_name = parts[0];

                // Mark async boundaries in statistics
                if is_async_boundary(func_name) {
                    result.stats.async_boundaries.insert(func_name.to_string());
                }

                *result.stats.calls.entry(func_name.to_string()).or_default() += 1;
                *result
                    .stats
                    .total_time
                    .entry(func_name.to_string())
                    .or_default() += time;

                result.filtered_stacks.push(line.to_string());
            }
        }
    }

    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = File::open("thag-profile.folded")?;
    let reader = BufReader::new(input);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let processed = process_profile_data(&lines);

    let options = vec!["Generate Flamegraph", "Show Statistics", "Filter Functions"];

    let selection = Select::new("Select action:", options).prompt()?;

    match selection {
        "Generate Flamegraph" => generate_flamegraph(&processed.filtered_stacks)?,
        "Show Statistics" => show_statistics(&processed.stats),
        "Filter Functions" => {
            let filtered = filter_functions(&processed.filtered_stacks)?;
            generate_flamegraph(&filtered)?;
        }
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

    // Make sure each line is properly formatted as "stack time"
    let formatted_stacks: Vec<String> = stacks
        .iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.rsplitn(2, ' ').collect();
            if parts.len() == 2 {
                Some(format!("{} {}", parts[1], parts[0]))
            } else {
                None
            }
        })
        .collect();

    if formatted_stacks.is_empty() {
        return Err("No valid stack traces found".into());
    }

    flamegraph::from_lines(
        &mut opts,
        formatted_stacks.iter().map(String::as_str),
        output,
    )?;

    println!("Flamegraph generated: flamegraph.svg");
    Ok(())
}

fn show_statistics(stats: &ProfileStats) {
    println!("\nFunction Statistics:");
    println!("===================");

    let mut entries: Vec<_> = stats.calls.iter().collect();
    entries.sort_by_key(|(_, &calls)| std::cmp::Reverse(calls));

    for (func, &calls) in entries {
        let total_time = stats.total_time.get(func).unwrap_or(&0);
        let avg_time = if calls > 0 { total_time / calls } else { 0 };
        println!(
            "{}: {} calls, {} μs total, {} μs avg",
            func, calls, total_time, avg_time
        );
    }

    // Show async boundaries separately
    if !stats.async_boundaries.is_empty() {
        println!("\nAsync Boundaries:");
        println!("================");
        for func in &stats.async_boundaries {
            if let Some(&calls) = stats.calls.get(func) {
                let total_time = stats.total_time.get(func).unwrap_or(&0);
                println!(
                    "{}: {} calls, {} μs total (includes subprocess time)",
                    func, calls, total_time
                );
            }
        }
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
