/*[toml]
[dependencies]
chrono = "0.4.39"
dirs = "6.0.0"
inferno = "0.12.0"
inquire = "0.7.5"
serde = { version = "1.0.216", features = ["derive"] }
serde_json = "1.0.138"
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
use serde::{Deserialize, Serialize};
// use std::borrow::Cow::Borrowed;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
// use std::path::Path;
use std::process::Command;
use std::time::Duration;
use thag_rs::profiling::ProfileStats;
use thag_rs::{ThagError, ThagResult};

#[derive(Default, Clone)]
pub struct ProcessedProfile {
    pub stacks: Vec<String>,
    pub title: String,
    pub subtitle: String,
    pub timestamp: DateTime<Local>,
    pub profile_type: ProfileType,
    pub memory_data: Option<MemoryData>,
}

#[derive(Default, Clone)]
pub enum ProfileType {
    #[default]
    Time,
    Memory,
}

#[derive(Default, Clone)]
pub struct MemoryData {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub peak_memory: u64,
    pub current_memory: u64,
    pub allocation_sizes: HashMap<usize, u64>, // Size -> Count
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum ChartType {
    TimeSequence, // flame_chart = true
    Aggregated,   // flame_chart = false
}

impl ChartType {
    fn configure_options(self, opts: &mut Options) {
        opts.flame_chart = matches!(self, Self::TimeSequence);
        opts.title = match self {
            Self::TimeSequence => "Execution Timeline",
            Self::Aggregated => "Aggregated Profile",
        }
        .to_string();
    }
}

fn process_profile_data(lines: &[String]) -> ProcessedProfile {
    let mut processed = ProcessedProfile::default();
    let mut stacks = Vec::new();

    // Determine profile type from first non-empty line
    for line in lines {
        if line.starts_with("# Time Profile") {
            processed.profile_type = ProfileType::Time;
            break;
        } else if line.starts_with("# Memory Profile") {
            processed.profile_type = ProfileType::Memory;
            break;
        }
    }

    for line in lines {
        if line.starts_with('#') {
            match line.split_once(": ") {
                Some(("# Script", script_path)) => {
                    let script_name = std::path::Path::new(script_path.trim())
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_else(|| script_path.trim());

                    processed.title = format!("Profile for: {script_name}");
                    processed.subtitle = format!("Path: {}", script_path.trim());
                }
                Some(("# Started", timestamp)) => {
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

    // Calculate memory stats if it's a memory profile
    if matches!(processed.profile_type, ProfileType::Memory) {
        processed.memory_data = Some(calculate_memory_stats(&processed.stacks));
    }

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
    loop {
        let analysis_types = vec![
            "Time Profile - Single",
            "Time Profile - Differential",
            "Memory Profile",
            "Exit",
        ];

        let analysis_type = Select::new("Select analysis type:", analysis_types).prompt()?;

        match analysis_type {
            "Exit" => break,
            "Time Profile - Single" => analyze_single_time_profile()?,
            "Time Profile - Differential" => analyze_differential_time_profiles()?,
            "Memory Profile" => analyze_memory_profiles()?,
            _ => println!("Invalid selection"),
        }

        println!("\nPress Enter to continue...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }

    Ok(())
}

// fn handle_time_profiles() -> ThagResult<()> {
//     loop {
//         // Get time profile files (exclude memory profiles)
//         let profile_groups = group_profile_files(|f| !f.contains("-memory"))?;

//         if profile_groups.is_empty() {
//             println!("No time profile files found.");
//             break;
//         }

//         // First ask if user wants single or differential analysis
//         let options = vec![
//             "Single Profile",
//             "Differential Analysis",
//             "Back to Main Menu",
//         ];
//         let choice = Select::new("Select analysis type:", options)
//             .prompt()
//             .map_err(|e| ThagError::Profiling(e.to_string()))?;

//         match choice {
//             "Back to Main Menu" => break,
//             "Differential Analysis" => {
//                 if let Err(e) = analyze_differential_time_profiles() {
//                     eprintln!("Error in differential analysis: {}", e);
//                 }
//                 println!("\nPress Enter to continue...");
//                 let _ = std::io::stdin().read_line(&mut String::new());
//             }
//             "Single Profile" => {
//                 match select_profile_file(&profile_groups)? {
//                     None => continue, // Return to time profile type selection
//                     Some(file_path) => {
//                         let processed = read_and_process_profile(&file_path)?;
//                         let stats = build_time_stats(&processed)?;

//                         loop {
//                             let options = vec![
//                                 "Show Flamechart",
//                                 "Show Statistics",
//                                 "Filter Functions",
//                                 "Back to Profile Selection",
//                             ];

//                             let action = Select::new("Select action:", options)
//                                 .prompt()
//                                 .map_err(|e| ThagError::Profiling(e.to_string()))?;

//                             match action {
//                                 "Back to Profile Selection" => break,
//                                 "Show Flamechart" => {
//                                     generate_flamechart(&processed)?;
//                                 }
//                                 "Show Statistics" => {
//                                     show_statistics(&stats, &processed);
//                                 }
//                                 "Filter Functions" => {
//                                     let filtered = filter_functions(&processed)?;
//                                     generate_flamechart(&filtered)?;
//                                 }
//                                 _ => println!("Unknown option"),
//                             }

//                             println!("\nPress Enter to continue...");
//                             let _ = std::io::stdin().read_line(&mut String::new());
//                         }
//                     }
//                 }
//             }
//             _ => println!("Unknown option"),
//         }
//     }
//     Ok(())
// }

// fn handle_memory_profiles() -> ThagResult<()> {
//     loop {
//         let profile_groups = group_profile_files(|f| f.contains("-memory"))?;

//         match select_profile_file(&profile_groups)? {
//             None => break, // Return to main menu
//             Some(file_path) => {
//                 let processed = read_and_process_profile(&file_path)?;

//                 loop {
//                     match show_memory_analysis_menu()? {
//                         None => break, // Return to file selection
//                         Some(action) => {
//                             process_memory_action(&action, &processed)?;
//                             println!("\nPress Enter to continue...");
//                             let _ = std::io::stdin().read_line(&mut String::new());
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     Ok(())
// }

fn analyze_single_time_profile() -> ThagResult<()> {
    // Get time profile files (exclude memory profiles)
    let profile_groups = group_profile_files(|f| !f.contains("-memory"))?;

    if profile_groups.is_empty() {
        println!("No time profile files found.");
        return Ok(());
    }

    match select_profile_file(&profile_groups)? {
        None => Ok(()), // User selected "Back"
        Some(file_path) => {
            let processed = read_and_process_profile(&file_path)?;
            let stats = build_time_stats(&processed)?;

            loop {
                let options = vec![
                    "Show Flamechart",
                    "Show Statistics",
                    "Filter Functions",
                    "Back to Profile Selection",
                ];

                let action = Select::new("Select action:", options)
                    .prompt()
                    .map_err(|e| ThagError::Profiling(e.to_string()))?;

                match action {
                    "Back to Profile Selection" => break,
                    "Show Flamechart" => generate_flamechart(&processed)?,
                    "Show Statistics" => {
                        show_statistics(&stats, &processed);
                    }
                    "Filter Functions" => {
                        let filtered = filter_functions(&processed)?;
                        generate_flamechart(&filtered)?;
                    }
                    _ => println!("Unknown option"),
                }

                println!("\nPress Enter to continue...");
                let _ = std::io::stdin().read_line(&mut String::new());
            }
            Ok(())
        }
    }
}

fn analyze_differential_time_profiles() -> ThagResult<()> {
    let filter = |filename: &str| !filename.contains("-memory");
    // let profile_groups = group_profile_files(filter)?;
    let (before, after) = select_profile_files(filter)?;
    generate_differential_flamegraph(&before, &after)
}

fn analyze_memory_profiles() -> ThagResult<()> {
    let profile_groups = group_profile_files(|f| f.contains("-memory"))?;

    if profile_groups.is_empty() {
        println!("No memory profile files found.");
        return Ok(());
    }

    match select_profile_file(&profile_groups)? {
        None => Ok(()), // User selected "Back"
        Some(selected_file) => {
            let processed = read_and_process_profile(&selected_file)?;

            let options = vec![
                "Show Memory Flamechart",
                "Show Memory Statistics",
                "Show Allocation Size Distribution",
                "Show Memory Timeline",
                "Filter Memory Patterns",
                "Back",
            ];

            // Show memory-specific menu and handle selection...
            let selection = Select::new("Select action:", options)
                .prompt()
                .map_err(|e| ThagError::Profiling(e.to_string()))?;

            match selection {
                // "Back" => Ok(()),
                "Show Memory Flamechart" => generate_memory_flamechart(&processed),
                "Show Memory Statistics" => {
                    show_memory_statistics(&processed);
                    Ok(())
                }
                "Show Allocation Size Distribution" => {
                    show_allocation_distribution(&processed);
                    Ok(())
                }
                "Show Memory Timeline" => generate_memory_timeline(&processed),
                "Filter Memory Patterns" => {
                    let filtered = filter_memory_patterns(&processed)?;
                    generate_memory_flamechart(&filtered)
                }
                _ => Ok(()),
            }
        }
    }
}

fn generate_flamechart(profile: &ProcessedProfile) -> ThagResult<()> {
    if profile.stacks.is_empty() {
        return Err(ThagError::Profiling(
            "No profile data available".to_string(),
        ));
    }

    let color_scheme = select_color_scheme()?;

    let chart_type = ChartType::TimeSequence;
    let svg = "flamechart.svg";
    let output = File::create(svg)?;
    let mut opts = Options::default();
    chart_type.configure_options(&mut opts);
    opts.title = if opts.flame_chart {
        "Execution Timeline Chart".to_string()
    } else {
        "Flame Graph".to_string()
    };
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

    enhance_svg_accessibility(svg)?;

    println!("Flame chart generated: flamechart.svg");
    open_in_browser("flamechart.svg").map_err(|e| ThagError::Profiling(e.to_string()))?;
    Ok(())
}

fn generate_differential_flamegraph(before: &PathBuf, after: &PathBuf) -> ThagResult<()> {
    // First, generate the differential data
    let mut diff_data = Vec::new();
    inferno::differential::from_files(
        inferno::differential::Options::default(), // Options for differential processing
        before,
        after,
        &mut diff_data,
    )
    .map_err(|e| ThagError::Profiling(e.to_string()))?;

    // Extract timestamps from filenames for the subtitle
    let before_name = before.file_name().unwrap_or_default().to_string_lossy();
    let after_name = after.file_name().unwrap_or_default().to_string_lossy();

    // Get script name from the first file
    let script_name = before
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.split('-').next())
        .unwrap_or("unknown");

    // Now generate the flamegraph from the differential data
    let svg = "flamegraph-diff.svg";
    let output = File::create(svg)?;
    let mut opts = Options::default();
    opts.title = format!("Differential Profile: {script_name}");
    opts.subtitle = format!("Comparing {before_name} → {after_name}").into();
    opts.colors = select_color_scheme()?;
    "μs".clone_into(&mut opts.count_name);
    opts.flame_chart = false;

    // Convert diff_data to lines
    let diff_lines =
        String::from_utf8(diff_data).map_err(|e| ThagError::Profiling(e.to_string()))?;
    let lines: Vec<&str> = diff_lines.lines().collect();

    flamegraph::from_lines(&mut opts, lines.iter().copied(), output)
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

    enhance_svg_accessibility(svg)?;

    println!("\nDifferential flame graph generated: flamegraph-diff.svg");
    println!("Red indicates increased time, blue indicates decreased time");
    println!("The width of the boxes represents the absolute time difference");

    open_in_browser("flamegraph-diff.svg").map_err(|e| ThagError::Profiling(e.to_string()))?;
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

fn filter_functions(processed: &ProcessedProfile) -> ThagResult<ProcessedProfile> {
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

    let to_filter = MultiSelect::new("Select functions to filter out:", function_list)
        .prompt()
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

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

#[derive(Debug, Serialize, Deserialize)]
struct ColorSchemeConfig {
    last_used: String,
}

impl Default for ColorSchemeConfig {
    fn default() -> Self {
        Self {
            last_used: "Aqua (Default)".to_string(),
        }
    }
}

#[derive(Debug)]
struct ColorSchemeOption {
    name: &'static str,
    description: &'static str,
    palette: Palette,
}

fn get_color_schemes() -> Vec<ColorSchemeOption> {
    vec![
        // Basic Palettes
        ColorSchemeOption {
            name: "Aqua (Default)",
            description: "Cool blue-green gradient, easy on the eyes",
            palette: Palette::Basic(BasicPalette::Aqua),
        },
        ColorSchemeOption {
            name: "Blue",
            description: "Calming blue tones, good for general use",
            palette: Palette::Basic(BasicPalette::Blue),
        },
        ColorSchemeOption {
            name: "Green",
            description: "Forest-like green palette, natural feel",
            palette: Palette::Basic(BasicPalette::Green),
        },
        ColorSchemeOption {
            name: "Hot",
            description: "Heat map style, red-yellow gradient",
            palette: Palette::Basic(BasicPalette::Hot),
        },
        ColorSchemeOption {
            name: "Mem",
            description: "Memory profiling focused, purple tones",
            palette: Palette::Basic(BasicPalette::Mem),
        },
        ColorSchemeOption {
            name: "Orange",
            description: "Warm orange tones, high visibility",
            palette: Palette::Basic(BasicPalette::Orange),
        },
        ColorSchemeOption {
            name: "Purple",
            description: "Rich purple gradient, distinctive look",
            palette: Palette::Basic(BasicPalette::Purple),
        },
        ColorSchemeOption {
            name: "Red",
            description: "Bold red tones, high contrast",
            palette: Palette::Basic(BasicPalette::Red),
        },
        ColorSchemeOption {
            name: "Yellow",
            description: "Bright yellow scheme, high visibility",
            palette: Palette::Basic(BasicPalette::Yellow),
        },
        // Special Palettes
        ColorSchemeOption {
            name: "Rust",
            description: "Official Rust-themed palette, orange and grey tones",
            palette: Palette::Multi(MultiPalette::Rust),
        },
    ]
}

fn load_last_used_scheme() -> ThagResult<String> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| ThagError::Profiling("Could not find config directory".to_string()))?
        .join("thag")
        .join("flamechart_colors.json");

    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let config: ColorSchemeConfig =
            serde_json::from_str(&content).map_err(|e| ThagError::Profiling(e.to_string()))?;
        Ok(config.last_used)
    } else {
        Ok(ColorSchemeConfig::default().last_used)
    }
}

fn save_color_scheme(name: &str) -> ThagResult<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ThagError::Profiling("Could not find config directory".to_string()))?
        .join("thag");

    fs::create_dir_all(&config_dir)?;

    let config = ColorSchemeConfig {
        last_used: name.to_string(),
    };

    let config_path = config_dir.join("flamechart_colors.json");
    fs::write(
        config_path,
        serde_json::to_string_pretty(&config).map_err(|e| ThagError::Profiling(e.to_string()))?,
    )?;

    Ok(())
}

fn select_color_scheme() -> ThagResult<Palette> {
    let schemes = get_color_schemes();
    let last_used = load_last_used_scheme()?;

    // First ask if user wants to use the last scheme or select a new one
    let use_last = inquire::Confirm::new(&format!(
        "Use last color scheme ({last_used})? (Press 'n' to select a different scheme)"
    ))
    .with_default(true)
    .prompt()
    .map_err(|e| ThagError::Profiling(e.to_string()))?;

    if use_last {
        return Ok(schemes
            .iter()
            .find(|s| s.name == last_used)
            .unwrap_or(&schemes[0])
            .palette);
    }

    // Group schemes for display
    println!("\nBasic Color Schemes:");
    println!("-------------------");
    schemes.iter().take(9).for_each(|s| {
        println!("{}: {}", s.name, s.description);
    });

    println!("\nSpecial Color Schemes:");
    println!("--------------------");
    schemes.iter().skip(9).for_each(|s| {
        println!("{}: {}", s.name, s.description);
    });

    println!(); // Add space before selection prompt

    let selection = Select::new(
        "Select color scheme:",
        schemes.iter().map(|s| s.name).collect::<Vec<_>>(),
    )
    .prompt()
    .map_err(|e| ThagError::Profiling(e.to_string()))?;

    // Save the selection
    save_color_scheme(selection)?;

    Ok(schemes
        .iter()
        .find(|s| s.name == selection)
        .unwrap()
        .palette)
}

// fn group_profile_files(filter: F) -> ThagResult<Vec<(String, Vec<PathBuf>)>> {
fn group_profile_files<T: Fn(&str) -> bool>(filter: T) -> ThagResult<Vec<(String, Vec<PathBuf>)>> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // Use file_navigator to get the directory and list .folded files
    let dir = std::env::current_dir()?;
    for entry in (dir.read_dir()?).flatten() {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if path.extension().and_then(|e| e.to_str()) == Some("folded") && filter(filename) {
                if let Some(script_stem) = filename.split('-').next() {
                    groups
                        .entry(script_stem.to_string())
                        .or_default()
                        .push(path);
                }
            }
        }
    }

    // Sort files within each group in reverse chronological order
    for files in groups.values_mut() {
        files.sort_by(|a, b| b.cmp(a)); // Reverse sort
    }

    // Convert to sorted vec for display
    let mut result: Vec<_> = groups.into_iter().collect();
    // Sort groups alphabetically but keep files within groups in reverse chronological order
    result.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(result)
}

fn select_profile_files<T: Fn(&str) -> bool>(filter: T) -> ThagResult<(PathBuf, PathBuf)> {
    let groups = group_profile_files(filter)?;

    if groups.is_empty() {
        return Err(ThagError::Profiling("No profile files found".to_string()));
    }

    // First select the script group
    let script_options: Vec<_> = groups
        .iter()
        .map(|(name, files)| format!("{} ({} profiles)", name, files.len()))
        .collect();

    let script_selection = Select::new("Select script to compare:", script_options.clone())
        .prompt()
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

    let script_idx = script_options
        .iter()
        .position(|s| s == &script_selection)
        .ok_or_else(|| ThagError::Profiling("Invalid selection".to_string()))?;

    let files = &groups[script_idx].1;
    if files.len() < 2 {
        return Err(ThagError::Profiling(
            "Need at least 2 profiles to compare".to_string(),
        ));
    }

    // Select 'before' profile
    let file_options: Vec<_> = files
        .iter()
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let before = Select::new("Select 'before' profile:", file_options.clone())
        .prompt()
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

    // Create new options list excluding the 'before' selection
    let after_options: Vec<_> = file_options
        .into_iter()
        .filter(|name| name != &before)
        .collect();

    let after = Select::new("Select 'after' profile:", after_options)
        .prompt()
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

    Ok((
        files
            .iter()
            .find(|p| p.file_name().unwrap_or_default().to_string_lossy() == before)
            .cloned()
            .unwrap(),
        files
            .iter()
            .find(|p| p.file_name().unwrap_or_default().to_string_lossy() == after)
            .cloned()
            .unwrap(),
    ))
}

fn select_profile_file(profile_groups: &[(String, Vec<PathBuf>)]) -> ThagResult<Option<PathBuf>> {
    if profile_groups.is_empty() {
        return Ok(None);
    }

    let mut file_options: Vec<_> = profile_groups
        .iter()
        .flat_map(|(_, files)| files)
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    file_options.push("Back".to_string());

    let selected = Select::new("Select profile to analyze:", file_options)
        .prompt()
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

    if selected == "Back" {
        return Ok(None);
    }

    // Find the actual PathBuf for the selected file
    for (_, files) in profile_groups {
        if let Some(file) = files.iter().find(|f| f.to_string_lossy() == selected) {
            return Ok(Some(file.clone()));
        }
    }

    Ok(None)
}

fn read_and_process_profile(path: &PathBuf) -> ThagResult<ProcessedProfile> {
    let input = File::open(path)?;
    let reader = BufReader::new(input);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    Ok(process_profile_data(&lines))
}

fn build_time_stats(processed: &ProcessedProfile) -> ThagResult<ProfileStats> {
    let mut stats = ProfileStats::default();
    for line in &processed.stacks {
        if let Some((stack, time)) = line.rsplit_once(' ') {
            if let Ok(duration) = time.parse::<u128>() {
                stats.record(
                    stack,
                    Duration::from_micros(
                        u64::try_from(duration).map_err(|e| ThagError::Profiling(e.to_string()))?,
                    ),
                );
            }
        }
    }
    Ok(stats)
}

fn generate_memory_flamechart(profile: &ProcessedProfile) -> ThagResult<()> {
    if profile.stacks.is_empty() {
        return Err(ThagError::Profiling(
            "No memory profile data available".to_string(),
        ));
    }

    let memory_data = profile
        .memory_data
        .as_ref()
        .ok_or_else(|| ThagError::Profiling("No memory statistics available".to_string()))?;

    let output = File::create("memory-flamechart.svg")?;
    let mut opts = Options::default();
    opts.title = format!("{} (Memory Profile)", profile.title);
    opts.subtitle = Some(format!(
        "{}\nStarted: {}\nTotal Allocations: {}, Peak Memory: {} bytes",
        profile.subtitle,
        profile.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
        memory_data.total_allocations,
        memory_data.peak_memory
    ));
    opts.colors = Palette::Basic(BasicPalette::Mem);
    "bytes".clone_into(&mut opts.count_name);
    opts.min_width = 0.1;
    opts.flame_chart = true;

    flamegraph::from_lines(
        &mut opts,
        profile.stacks.iter().rev().map(String::as_str),
        output,
    )?;

    enhance_svg_accessibility("memory-flamechart.svg")?;
    println!("Memory flame chart generated: memory-flamechart.svg");
    open_in_browser("memory-flamechart.svg").map_err(|e| ThagError::Profiling(e.to_string()))?;
    Ok(())
}

fn show_memory_statistics(profile: &ProcessedProfile) {
    if let Some(memory_data) = &profile.memory_data {
        println!("\nMemory Profile Statistics");
        println!("========================");
        println!("Total Allocations:    {}", memory_data.total_allocations);
        println!("Total Deallocations:  {}", memory_data.total_deallocations);
        println!("Peak Memory Usage:    {} bytes", memory_data.peak_memory);
        println!("Current Memory Usage: {} bytes", memory_data.current_memory);

        if memory_data.total_deallocations < memory_data.total_allocations {
            println!(
                "\nWarning: Possible memory leak - {} allocations weren't freed",
                memory_data.total_allocations - memory_data.total_deallocations
            );
        }

        // Show top allocation sites
        println!("\nTop Allocation Sites:");
        let mut allocation_sites: Vec<_> = profile
            .stacks
            .iter()
            .filter_map(|stack| {
                stack
                    .split_once(' ')
                    .map(|(site, size)| (site, size.parse::<usize>().unwrap_or(0)))
            })
            .collect();

        allocation_sites.sort_by_key(|(_site, size)| std::cmp::Reverse(*size));

        for (site, size) in allocation_sites.iter().take(10) {
            println!("{size:>10} bytes: {site}");
        }
    }
}

fn show_allocation_distribution(profile: &ProcessedProfile) -> ThagResult<()> {
    let memory_data = profile
        .memory_data
        .as_ref()
        .ok_or_else(|| ThagError::Profiling("No memory statistics available".to_string()))?;

    if memory_data.allocation_sizes.is_empty() {
        println!("No allocation data available.");
        return Ok(());
    }

    println!("\nAllocation Size Distribution");
    println!("===========================");

    // Define size buckets (in bytes)
    let buckets = vec![
        (0, 64, "0-64"),
        (65, 256, "65-256"),
        (257, 1024, "257-1K"),
        (1025, 4096, "1K-4K"),
        (4097, 16384, "4K-16K"),
        (16385, 65536, "16K-64K"),
        (65537, usize::MAX, ">64K"),
    ];

    let mut bucket_counts: HashMap<&str, u64> = HashMap::new();
    let mut total_bytes = 0u64;

    for (&size, &count) in &memory_data.allocation_sizes {
        total_bytes += size as u64 * count;
        for &(min, max, label) in &buckets {
            if size >= min && size <= max {
                *bucket_counts.entry(label).or_default() += count;
                break;
            }
        }
    }

    // Calculate max count for bar scaling
    let max_count = bucket_counts.values().max().copied().unwrap_or(1);

    // Display distribution with bars
    for &(_, _, label) in &buckets {
        let count = bucket_counts.get(label).copied().unwrap_or(0);
        let bar_length = ((count as f64 / max_count as f64) * 50.0) as usize;
        println!("{:>8}: {:>6} |{}", label, count, "█".repeat(bar_length));
    }

    println!("\nTotal memory allocated: {} bytes", total_bytes);
    println!(
        "Average allocation size: {} bytes",
        total_bytes / memory_data.total_allocations
    );

    Ok(())
}

// fn generate_memory_timeline(_profile: &ProcessedProfile) -> ThagResult<()> {
//     // Create a timeline visualization showing memory usage over time
//     let svg = "memory-timeline.svg";
//     let _output = File::create(svg)?;

//     // TODO: Implement SVG generation for memory timeline
//     // This would show:
//     // - X axis: Time
//     // - Y axis: Memory usage
//     // - Color-coded regions for different allocation patterns
//     // - Markers for peak memory events

//     println!("Memory timeline generated: memory-timeline.svg");
//     enhance_svg_accessibility(svg)?;

//     open_in_browser(svg).map_err(|e| ThagError::Profiling(e.to_string()))?;
//     Ok(())
// }

fn generate_memory_timeline(profile: &ProcessedProfile) -> ThagResult<()> {
    if let Some(_memory_data) = &profile.memory_data {
        let mut timeline_data: Vec<(usize, u64)> = profile
            .stacks
            .iter()
            .filter_map(|line| {
                line.split_whitespace()
                    .last()
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|size| (size, 1))
            })
            .collect();

        // Sort by size for visualization
        timeline_data.sort_by_key(|(size, _)| *size);

        // Generate SVG (basic implementation)
        let svg = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="1200" height="600" xmlns="http://www.w3.org/2000/svg">
    <style>
        .label {{ font-family: Arial; font-size: 12px; }}
        .title {{ font-family: Arial; font-size: 16px; font-weight: bold; }}
    </style>
    <text x="600" y="30" class="title" text-anchor="middle">Memory Timeline</text>
    <!-- Add timeline visualization here -->
</svg>"#;

        fs::write("memory-timeline.svg", svg)?;
        open_in_browser("memory-timeline.svg").map_err(|e| ThagError::Profiling(e.to_string()))?;
    }
    Ok(())
}

fn filter_memory_patterns(profile: &ProcessedProfile) -> ThagResult<ProcessedProfile> {
    let patterns = vec![
        "Large allocations (>1MB)",
        "Temporary allocations",
        "Leaked memory",
        "Frequent allocations",
        "Custom pattern...",
    ];

    let selected = MultiSelect::new("Select memory patterns to highlight:", patterns)
        .prompt()
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

    // Create a new filtered profile based on selected patterns
    let mut filtered = profile.clone();
    filtered.stacks = profile
        .stacks
        .iter()
        .filter(|stack| {
            // Apply selected filters
            selected
                .iter()
                .any(|pattern| matches_memory_pattern(stack, pattern))
        })
        .cloned()
        .collect();

    Ok(filtered)
}

fn matches_memory_pattern(stack: &str, pattern: &str) -> bool {
    match pattern {
        "Large allocations (>1MB)" => {
            if let Some((_stack, size)) = stack.rsplit_once(' ') {
                size.parse::<usize>()
                    .map(|s| s > 1_000_000)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        "Temporary allocations" => stack.contains("temp") || stack.contains("temporary"),
        "Leaked memory" => stack.contains("leak") || !stack.contains("free"),
        "Frequent allocations" => stack.contains("loop") || stack.contains("iter"),
        _ => false,
    }
}

fn enhance_svg_accessibility(svg_path: &str) -> ThagResult<()> {
    let content = fs::read_to_string(svg_path)?;

    // Make the inactive search link more visible
    let enhanced = content.replace(
        "opacity:0.1",
        "opacity:0.5", // Darker grey for better visibility
    );

    fs::write(svg_path, enhanced)?;
    Ok(())
}

fn calculate_memory_stats(stacks: &[String]) -> MemoryData {
    let mut memory_data = MemoryData::default();
    let mut current_memory = 0u64;
    let mut peak_memory = 0u64;

    for line in stacks {
        if let Some((stack, size_str)) = line.rsplit_once(' ') {
            if let Ok(size) = size_str.parse::<u64>() {
                memory_data.total_allocations += 1;
                *memory_data
                    .allocation_sizes
                    .entry(size as usize)
                    .or_default() += 1;

                // Assume deallocations if the stack contains certain keywords
                if stack.contains("free") || stack.contains("drop") || stack.contains("deallocate")
                {
                    memory_data.total_deallocations += 1;
                    current_memory = current_memory.saturating_sub(size);
                } else {
                    current_memory += size;
                    peak_memory = peak_memory.max(current_memory);
                }
            }
        }
    }

    memory_data.peak_memory = peak_memory;
    memory_data.current_memory = current_memory;
    memory_data
}

// #[derive(Debug)]
// enum MainMenuChoice {
//     TimeProfile,
//     MemoryProfile,
//     Exit,
// }

// fn show_main_menu() -> ThagResult<MainMenuChoice> {
//     let options = vec!["Time Profile Analysis", "Memory Profile Analysis", "Exit"];

//     let selection = Select::new("Select profile type:", options)
//         .prompt()
//         .map_err(|e| ThagError::Profiling(e.to_string()))?;

//     Ok(match selection {
//         "Time Profile Analysis" => MainMenuChoice::TimeProfile,
//         "Memory Profile Analysis" => MainMenuChoice::MemoryProfile,
//         "Exit" => MainMenuChoice::Exit,
//         _ => MainMenuChoice::Exit,
//     })
// }

// fn show_memory_analysis_menu() -> ThagResult<Option<&'static str>> {
//     let options = vec![
//         "Show Memory Flamechart",
//         "Show Memory Statistics",
//         "Show Allocation Size Distribution",
//         "Show Memory Timeline",
//         "Filter Memory Patterns",
//         "Back to File Selection",
//     ];

//     let selection = Select::new("Select action:", options)
//         .prompt()
//         .map_err(|e| ThagError::Profiling(e.to_string()))?;

//     Ok(match selection {
//         "Back to File Selection" => None,
//         action => Some(action),
//     })
// }

// fn process_memory_action(action: &str, profile: &ProcessedProfile) -> ThagResult<()> {
//     match action {
//         "Show Memory Flamechart" => generate_memory_flamechart(profile)?,
//         "Show Memory Statistics" => show_memory_statistics(profile),
//         "Show Allocation Size Distribution" => show_allocation_distribution(profile),
//         "Show Memory Timeline" => generate_memory_timeline(profile)?,
//         "Filter Memory Patterns" => {
//             let filtered = filter_memory_patterns(profile)?;
//             generate_memory_flamechart(&filtered)?;
//         }
//         _ => println!("Unknown action"),
//     }
//     Ok(())
// }

fn parse_allocation_log(lines: &[String]) -> MemoryData {
    let mut memory_data = MemoryData::default();
    let mut current_memory = 0u64;

    for line in lines {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let (stack, op) = if parts.len() > 2 {
                (parts[0..parts.len() - 1].join(" "), *parts.last().unwrap())
            } else {
                (parts[0].to_string(), parts[1])
            };

            if let Some(size_str) = op.strip_prefix('+') {
                if let Ok(size) = size_str.parse::<u64>() {
                    memory_data.total_allocations += 1;
                    current_memory += size;
                    memory_data.peak_memory = memory_data.peak_memory.max(current_memory);
                }
            } else if let Some(size_str) = op.strip_prefix('-') {
                if let Ok(size) = size_str.parse::<u64>() {
                    memory_data.total_deallocations += 1;
                    current_memory = current_memory.saturating_sub(size);
                }
            }
        }
    }

    memory_data.current_memory = current_memory;
    memory_data
}
