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
    LocalResult::{Ambiguous, None as Nada, Single},
    TimeZone,
};
use inferno::flamegraph::{
    self,
    color::{BasicPalette, MultiPalette},
    Options, Palette,
};
use inquire::{MultiSelect, Select};
// use inquire::Text;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
// use std::io::Write;
// use std::borrow::Cow::Borrowed;
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
                                    // pub notes: String,
}

fn process_profile_data(lines: &[String]) -> ProcessedProfile {
    let mut processed = ProcessedProfile::default();
    let mut stacks = Vec::new();

    for line in lines {
        if line.starts_with('#') {
            match line.split_once(": ") {
                Some(("# Script", script_path)) => {
                    // Get just the file name for the title
                    let script_name = std::path::Path::new(script_path.trim())
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_else(|| script_path.trim());

                    processed.title = format!("Flamechart for profile: {script_name}");
                    // Store full path for subtitle
                    processed.subtitle = format!("Path: {}\n\nStarted: ", script_path.trim());
                }
                Some(("# Started", timestamp)) => {
                    // Convert microseconds since epoch to DateTime
                    if let Ok(ts) = timestamp.trim().parse::<i64>() {
                        match Local.timestamp_micros(ts) {
                            Single(dt) | Ambiguous(dt, _) => {
                                processed.timestamp = dt;
                            }
                            Nada => (),
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
    processed
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

    let processed = process_profile_data(&lines);

    // Build stats if needed
    let mut stats = ProfileStats::default();
    for line in &processed.stacks {
        if let Some((stack, time)) = line.rsplit_once(' ') {
            if let Ok(duration) = time.parse::<u128>() {
                stats.record(stack, Duration::from_micros(u64::try_from(duration)?));
            }
        }
    }

    let options = vec![
        "Generate & Show Flamechart",
        "Show Statistics",
        "Filter Functions",
        // "Show Async Boundaries",
    ];

    let selection = Select::new("Select action:", options).prompt()?;

    match selection {
        "Generate & Show Flamechart" => generate_flamechart(&processed)?,
        "Show Statistics" => show_statistics(&stats, &processed),
        "Filter Functions" => {
            let filtered = filter_functions(&processed)?;
            generate_flamechart(&filtered)?;
        }
        // "Show Async Boundaries" => show_async_boundaries(&stats),
        _ => println!("Unknown option"),
    }

    Ok(())
}

fn generate_flamechart(profile: &ProcessedProfile) -> ThagResult<()> {
    if profile.stacks.is_empty() {
        return Err(ThagError::Profiling("No profile data available"));
    }

    let color_scheme = select_color_scheme()?;

    let output = File::create("flamechart.svg")?;
    let mut opts = Options::default();
    // opts.title = profile.title.clone();
    opts.subtitle = Some(format!(
        "Started: {}",
        profile.timestamp.format("%Y-%m-%d %H:%M:%S%.3f")
    ));
    // opts.notes = profile.subtitle.clone();
    opts.colors = color_scheme;
    "μs".clone_into(&mut opts.count_name);
    opts.min_width = 0.01;
    // opts.color_diffusion = true;
    opts.flame_chart = true;

    flamegraph::from_lines(
        &mut opts,
        profile.stacks.iter().rev().map(String::as_str),
        output,
    )?;

    println!("Flame chart generated: flamechart.svg");
    if let Err(e) = open_in_browser("flamechart.svg") {
        eprintln!("Failed to open browser: {e}");
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
    println!(
        "Started: {}",
        profile.timestamp.format("%Y-%m-%d %H:%M:%S%.3f")
    );
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
        println!("{func}: {calls} calls, {total_time} μs total, {avg_time} μs avg");
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
    function_list.sort_unstable();

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

// fn show_async_boundaries(stats: &ProfileStats) {
//     println!("\nAsync Boundaries:");
//     println!("================");

//     for func_name in stats.async_boundaries.iter() {
//         if let Some(&calls) = stats.calls.get(func_name) {
//             let total_time = stats.total_time.get(func_name).unwrap_or(&0);
//             println!(
//                 "{}: {} calls, {} μs total (includes subprocess time)",
//                 func_name, calls, total_time
//             );
//         }
//     }
// }

fn select_color_scheme() -> ThagResult<Palette> {
    let options = vec![
        "Aqua (Default)",
        "Blue",
        "Green",
        "Hot",
        "Mem",
        "Orange",
        "Purple",
        "Red",
        "Yellow",
        "Rust", // Special case - this is a MultiPalette
    ];

    let selection = Select::new("Select color scheme:", options)
        // .with_default(0)
        .prompt()
        .map_err(|e| <String as Into<ThagError>>::into(e.to_string()))?;

    Ok(match selection {
        "Aqua (Default)" => Palette::Basic(BasicPalette::Aqua),
        "Blue" => Palette::Basic(BasicPalette::Blue),
        "Green" => Palette::Basic(BasicPalette::Green),
        "Hot" => Palette::Basic(BasicPalette::Hot),
        "Mem" => Palette::Basic(BasicPalette::Mem),
        "Orange" => Palette::Basic(BasicPalette::Orange),
        "Purple" => Palette::Basic(BasicPalette::Purple),
        "Red" => Palette::Basic(BasicPalette::Red),
        "Yellow" => Palette::Basic(BasicPalette::Yellow),
        "Rust" => Palette::Multi(MultiPalette::Rust),
        _ => unreachable!(),
    })
}
