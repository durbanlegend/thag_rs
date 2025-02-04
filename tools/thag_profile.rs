/*[toml]
[dependencies]
chrono = "0.4.39"
inferno = "0.12.0"
inquire = "0.7.5"
serde = { version = "1.0.216", features = ["derive"] }
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["config", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["config", "simplelog"] }
*/

/// Profile graph/chart generator for the `thag` internal profiler.
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
use chrono::{
    DateTime, Local,
    LocalResult::{Ambiguous, Single},
    TimeZone,
};
use inferno::flamegraph::{self, color::BasicPalette, Options, Palette};
use inquire::{MultiSelect, Select};
// use inquire::Text;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
// use std::io::Write;
use std::process::Command;
use std::time::Duration;
use thag_rs::profiling::ProfileStats;
use thag_rs::{ThagError, ThagResult};

#[derive(Default, Clone)]
pub struct ProcessedProfile {
    pub stacks: Vec<String>,
    pub title: String,
    pub subtitle: String,
    pub timestamp: DateTime<Local>, // From chrono crate
}

fn process_profile_data(lines: &[String]) -> ThagResult<ProcessedProfile> {
    let mut processed = ProcessedProfile::default();
    let mut stacks = Vec::new();

    for line in lines {
        if line.starts_with('#') {
            match line.split_once(": ") {
                Some(("# Script", script)) => {
                    processed.title = format!("Profile: {}", script.trim());
                }
                Some(("# Started", timestamp)) => {
                    // Convert microseconds since epoch to DateTime
                    if let Ok(ts) = timestamp.trim().parse::<i64>() {
                        match Local.timestamp_micros(ts) {
                            Single(dt) | Ambiguous(dt, _) => {
                                processed.timestamp = dt;
                                processed.subtitle =
                                    format!("Started: {}", dt.format("%Y-%m-%d %H:%M:%S%.3f"));
                            }
                            _ => (),
                        }
                    }
                }
                _ => {}
            }
        } else if !line.is_empty() {
            stacks.push(line.to_string());
        }
    }

    processed.stacks = stacks;
    Ok(processed)
}

// fn is_async_boundary(func_name: &str) -> bool {
//     // Expanded list of async indicators
//     matches!(func_name.trim(),
//         "handle_build_or_check" |
//         "build" |
//         _ if func_name.contains("spawn") ||
//             func_name.contains("wait") ||
//             func_name.contains("async") ||
//             func_name.contains("process") ||
//             func_name.contains("command") ||
//             func_name.contains("exec")
//     )
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = File::open("thag-profile.folded")?;
    let reader = BufReader::new(input);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let processed = process_profile_data(&lines)?;

    // Build stats if needed
    let mut stats = ProfileStats::default();
    for line in &processed.stacks {
        if let Some((stack, time)) = line.rsplit_once(' ') {
            if let Ok(duration) = time.parse::<u128>() {
                stats.record(stack, Duration::from_micros(duration as u64));
            }
        }
    }

    let options = vec![
        "Generate & Show Flamechart",
        "Show Statistics",
        "Filter Functions",
        "Show Async Boundaries",
    ];

    let selection = Select::new("Select action:", options).prompt()?;

    match selection {
        "Generate & Show Flamechart" => generate_flamechart(&processed)?,
        "Show Statistics" => show_statistics(&stats, &processed),
        "Filter Functions" => {
            let filtered = filter_functions(&processed)?;
            generate_flamechart(&filtered)?;
        }
        "Show Async Boundaries" => show_async_boundaries(&stats),
        _ => println!("Unknown option"),
    }

    Ok(())
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let args = parse_args()?;

//     // Read the profile data
//     let content = std::fs::read_to_string("thag-profile.folded")?;
//     let processed = process_profile_data(&content)?;

//     // Build stats if needed
//     let mut stats = ProfileStats::default();
//     for line in &processed.stacks {
//         if let Some((stack, time)) = line.rsplit_once(' ') {
//             if let Ok(duration) = time.parse::<u128>() {
//                 stats.record(stack, Duration::from_micros(duration as u64));
//             }
//         }
//     }

//     match args.command {
//         Command::ShowStats => {
//             show_statistics(&stats, &processed);
//         }
//         Command::ShowFlameChart { filter } => {
//             let filtered_profile = if let Some(filter) = filter {
//                 ProcessedProfile {
//                     stacks: processed
//                         .stacks
//                         .into_iter()
//                         .filter(|line| line.contains(&filter))
//                         .collect(),
//                     ..processed // Keep the metadata
//                 }
//             } else {
//                 processed
//             };
//             generate_flamechart(&filtered_profile)?;
//         }
//         Command::ShowAsync => {
//             // Similar adaptation for async view if needed
//         }
//     }

//     Ok(())
// }

// fn extract_metadata(stacks: &[String]) -> (String, String) {
//     let mut title = String::from("Thag Profile");
//     let mut subtitle = String::new();

//     for line in stacks.into_iter().take(10) {
//         eprintln!("line={line}");
//         if line.starts_with("# ") {
//             let split_once = line.split_once(": ");
//             eprintln!("split_once={split_once:?}");
//             match split_once {
//                 Some(("# Script", script)) => {
//                     eprintln!("Found {script}");
//                     title = format!("Profile: {}", script.trim());
//                 }
//                 Some(("# Started", timestamp)) => {
//                     eprintln!("Found {timestamp}");
//                     subtitle = format!("Started: {}", timestamp.trim());
//                 }
//                 Some(other) => {
//                     eprintln!("Found {other:?}");
//                 }
//                 _ => {}
//             }
//         } else {
//             eprintln!("Does not start with '# '={line}");
//         }
//     }

//     (title, subtitle)
// }

fn generate_flamechart(profile: &ProcessedProfile) -> ThagResult<()> {
    if profile.stacks.is_empty() {
        return Err(ThagError::Profiling("No profile data available"));
    }

    let output = File::create("flamechart.svg")?;
    let mut opts = Options::default();
    opts.title = profile.title.clone();
    opts.subtitle = Some(profile.subtitle.clone());
    opts.colors = Palette::Basic(BasicPalette::Aqua);
    opts.count_name = "μs".to_owned();
    opts.min_width = 0.1;
    opts.flame_chart = true;

    flamegraph::from_lines(
        &mut opts,
        profile.stacks.iter().rev().map(String::as_str),
        output,
    )?;
    // .map_err(|e| ThagError::FromStr(&e.to_string()))?;

    println!("Flame chart generated: flamechart.svg");
    if let Err(e) = open_in_browser("flamechart.svg") {
        eprintln!("Failed to open browser: {}", e);
    }
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

fn show_statistics(stats: &ProfileStats, profile: &ProcessedProfile) {
    println!("\n{}", profile.title);
    println!("{}", profile.subtitle);
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

fn filter_functions(
    processed: &ProcessedProfile,
) -> Result<ProcessedProfile, inquire::InquireError> {
    let functions: HashSet<_> = processed
        .stacks
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

    Ok(ProcessedProfile {
        stacks: processed
            .stacks
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
            .collect(),
        ..processed.clone() // Keep the metadata
    })
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
