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
use std::cmp::Ordering;
// use std::borrow::Cow::Borrowed;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
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

            loop {
                let options = vec![
                    "Show Memory Flamechart",
                    "Show Memory Statistics",
                    "Show Allocation Size Distribution",
                    "Show Memory Timeline",
                    "Filter Memory Patterns",
                    "Back to Profile Selection",
                ];

                // Show memory-specific menu and handle selection...
                let selection = Select::new("Select action:", options)
                    .prompt()
                    .map_err(|e| ThagError::Profiling(e.to_string()))?;

                match selection {
                    "Back to Profile Selection" => break,
                    "Show Memory Flamechart" => generate_memory_flamechart(&processed)
                        .map_or_else(|e| println!("{e}"), |()| {}),
                    "Show Memory Statistics" => show_memory_statistics(&processed, &selected_file),
                    "Show Allocation Size Distribution" => show_allocation_distribution(&processed)
                        .map_or_else(|e| println!("{e}"), |()| {}),
                    "Show Memory Timeline" => generate_memory_timeline(&processed, &selected_file)
                        .map_or_else(|e| println!("{e}"), |()| {}),
                    "Filter Memory Patterns" => {
                        filter_memory_patterns(&processed).map_or_else(
                            |e| println!("{e}"),
                            |filtered| {
                                generate_memory_flamechart(&filtered)
                                    .map_or_else(|e| println!("{e}"), |()| {});
                            },
                        );
                    }
                    _ => {}
                }

                println!("\nPress Enter to continue...");
                let _ = std::io::stdin().read_line(&mut String::new());
            }
            Ok(())
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

// Add pattern analysis to memory statistics display
#[allow(clippy::cast_sign_loss)]
fn show_memory_statistics(profile: &ProcessedProfile, file_path: &Path) {
    if let Some(memory_data) = &profile.memory_data {
        let mut memory_data = memory_data.clone();

        // Try to find and parse allocation log
        if let Some(log_path) = find_allocation_log(file_path) {
            match parse_allocation_log(&log_path) {
                Ok(alloc_entries) => {
                    // Add allocation size analysis before enhancing stats
                    println!("\nAllocation Analysis:");
                    println!("===================");
                    let mut total_allocated = 0u64;
                    let mut total_deallocated = 0u64;

                    for entry in &alloc_entries {
                        if entry.size > 0 {
                            total_allocated += entry.size as u64;
                        } else {
                            total_deallocated += (-entry.size) as u64;
                        }
                    }

                    println!("Total Bytes Allocated:   {total_allocated}");
                    println!("Total Bytes Deallocated: {total_deallocated}");
                    println!(
                        "Average Allocation Size: {} bytes",
                        if memory_data.total_allocations > 0 {
                            total_allocated / memory_data.total_allocations
                        } else {
                            0
                        }
                    );

                    // Now enhance the stats
                    enhance_memory_stats(&mut memory_data, &alloc_entries);
                    println!("\nMemory Profile Statistics (including allocation log)");

                    // Show allocation patterns
                    let patterns = analyze_allocation_patterns(&alloc_entries);
                    if !patterns.is_empty() {
                        println!("\nAllocation Patterns:");
                        println!("===================");
                        let mut pattern_vec: Vec<_> = patterns.iter().collect();
                        pattern_vec.sort_by_key(|(_, p)| std::cmp::Reverse(p.total_allocated));

                        for (stack, pattern) in pattern_vec.iter().take(5) {
                            if !stack.is_empty() {
                                // Skip empty stack frames
                                println!("\nStack: {stack}");
                                println!("  Allocations:   {}", pattern.allocation_count);
                                println!("  Deallocations: {}", pattern.deallocation_count);
                                println!("  Total Bytes:   {}", pattern.total_allocated);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("\nMemory Profile Statistics (allocation log parsing failed: {e})");
                }
            }
        } else {
            println!("\nMemory Profile Statistics (no allocation log found)");
        }

        println!("========================");
        println!("Total Allocations:    {}", memory_data.total_allocations);
        if memory_data.total_deallocations > 0 {
            println!("Total Deallocations:  {}", memory_data.total_deallocations);
            if memory_data.total_deallocations > memory_data.total_allocations {
                println!("Note: Deallocation count exceeds allocation count.");
                println!("      This may indicate:");
                println!("      - Deallocations of memory allocated before profiling started");
                println!("      - Multiple deallocation events for complex data structures");
                println!("      - Partial deallocations of larger allocations");
            } else {
                println!(
                    "Net Allocations:      {}",
                    memory_data.total_allocations - memory_data.total_deallocations
                );
            }
        }
        println!("Peak Memory Usage:    {} bytes", memory_data.peak_memory);
        println!("Current Memory Usage: {} bytes", memory_data.current_memory);

        // Show top allocation sites from profile data
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

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
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
        println!("{label:>8}: {count:>6} |{}", "█".repeat(bar_length));
    }

    println!("\nTotal memory allocated: {total_bytes} bytes",);
    println!(
        "Average allocation size: {} bytes",
        total_bytes / memory_data.total_allocations
    );

    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn filter_memory_patterns(profile: &ProcessedProfile) -> ThagResult<ProcessedProfile> {
    let patterns = vec![
        "Large allocations (>1MB)",
        "Temporary allocations",
        "Leaked memory",
        "Frequent allocations",
        "Custom pattern...",
    ];

    let selected = MultiSelect::new("Select memory patterns to filter out:", patterns)
        .prompt()
        .map_err(|e| ThagError::Profiling(e.to_string()))?;

    // If nothing selected, return unfiltered profile
    if selected.is_empty() {
        println!("No patterns selected - showing all entries");
        return Ok(profile.clone());
    }

    // Handle custom pattern if selected
    #[allow(clippy::cast_precision_loss)]
    let custom_pattern = if selected.contains(&"Custom pattern...") {
        inquire::Text::new("Enter custom pattern to filter (e.g., 'vec' or 'string'):")
            .prompt()
            .map_err(|e| ThagError::Profiling(e.to_string()))?
    } else {
        String::new()
    };

    // Track filtering statistics
    let mut filter_stats: HashMap<&str, usize> = HashMap::new();
    let total_entries = profile.stacks.len();

    // Create a new filtered profile
    let mut filtered = profile.clone();
    filtered.stacks = profile
        .stacks
        .iter()
        .filter(|stack| {
            selected.iter().any(|&pattern| {
                let matches = if pattern == "Custom pattern..." {
                    stack
                        .to_lowercase()
                        .contains(&custom_pattern.to_lowercase())
                } else {
                    matches_memory_pattern(stack, pattern)
                };
                if matches {
                    *filter_stats.entry(pattern).or_insert(0) += 1;
                }
                matches
            })
        })
        .cloned()
        .collect();

    // Display filtering statistics
    println!("\nFiltering Statistics:");
    println!("====================");
    println!("Total entries: {total_entries}");

    let mut total_filtered = 0;
    for pattern in &selected {
        let count = filter_stats.get(pattern).copied().unwrap_or(0);
        total_filtered = total_filtered.max(count); // Use max to avoid double-counting
        let percentage = (count as f64 / total_entries as f64 * 100.0).round();
        if pattern == &"Custom pattern..." {
            println!("Pattern '{custom_pattern}': {count} entries ({percentage:.1}%)");
        } else {
            println!("Pattern '{pattern}': {count} entries ({percentage:.1}%)");
        }
    }

    let remaining = filtered.stacks.len();
    let remaining_percentage = (remaining as f64 / total_entries as f64 * 100.0).round();
    let filtered_percentage = (total_filtered as f64 / total_entries as f64 * 100.0).round();

    println!("\nSummary:");
    println!("Entries remaining: {remaining} ({remaining_percentage:.1}%)");
    println!("Entries filtered:  {total_filtered} ({filtered_percentage:.1}%)");

    if filtered.stacks.is_empty() {
        println!(
            "\nWarning: All entries were filtered out. Displaying unfiltered profile instead."
        );
        println!("Consider adjusting your filter criteria.");
        println!("\nPress Enter to continue with unfiltered display...");
        let _ = std::io::stdin().read_line(&mut String::new());
        Ok(profile.clone())
    } else {
        Ok(filtered)
    }
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
        "Temporary allocations" => {
            let stack_lower = stack.to_lowercase();
            stack_lower.contains("temp")
                || stack_lower.contains("tmp")
                || stack_lower.contains("temporary")
                || stack_lower.contains("buffer")
        }
        "Leaked memory" => {
            // Only consider it leaked if it's an allocation that doesn't have associated deallocation terms
            let stack_lower = stack.to_lowercase();
            stack_lower.contains("alloc")
                && !stack_lower.contains("free")
                && !stack_lower.contains("drop")
                && !stack_lower.contains("deallocate")
        }
        "Frequent allocations" => {
            let stack_lower = stack.to_lowercase();
            stack_lower.contains("loop")
                || stack_lower.contains("iter")
                || stack_lower.contains("each")
                || stack_lower.contains("map")
        }
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

#[allow(clippy::cast_possible_truncation)]
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

#[derive(Debug)]
struct AllocationLogEntry {
    stack: String,
    size: i64, // Positive for allocations, negative for deallocations
}

#[derive(Debug, Default)]
struct AllocationPattern {
    allocation_count: u64,
    deallocation_count: u64,
    total_allocated: u64,
    total_deallocated: u64,
}

#[allow(clippy::cast_sign_loss)]
fn analyze_allocation_patterns(
    entries: &[AllocationLogEntry],
) -> HashMap<String, AllocationPattern> {
    let mut patterns: HashMap<String, AllocationPattern> = HashMap::new();

    for entry in entries {
        let pattern = patterns.entry(entry.stack.clone()).or_default();
        if entry.size > 0 {
            pattern.allocation_count += 1;
            pattern.total_allocated += entry.size as u64;
        } else {
            pattern.deallocation_count += 1;
            pattern.total_deallocated += (-entry.size) as u64;
        }
    }

    patterns
}

fn find_allocation_log(memory_profile_path: &Path) -> Option<PathBuf> {
    let file_stem = memory_profile_path.file_stem()?.to_str()?;
    // Remove "-memory" suffix and add "-alloc.log"
    let log_name = file_stem.replace("-memory", "-alloc.log");
    let log_path = memory_profile_path.with_file_name(log_name);
    if log_path.exists() {
        Some(log_path)
    } else {
        None
    }
}

fn parse_allocation_log(path: &Path) -> ThagResult<Vec<AllocationLogEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Split into stack and operation
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            let (stack, op) = if parts.len() > 1 {
                (parts[0..parts.len() - 1].join(" "), *parts.last().unwrap())
            } else {
                (String::new(), parts[0])
            };

            // Parse operation (+size, -size, or =size)
            let size = if let Some(size_str) = op.strip_prefix('+') {
                size_str.parse::<i64>().unwrap_or(0)
            } else if let Some(size_str) = op.strip_prefix('-') {
                -size_str.parse::<i64>().unwrap_or(0)
            } else if let Some(_size_str) = op.strip_prefix('=') {
                0 // Current size, not a change
            } else {
                continue;
            };

            entries.push(AllocationLogEntry { stack, size });
        }
    }

    Ok(entries)
}

#[allow(clippy::cast_sign_loss)]
fn enhance_memory_stats(memory_data: &mut MemoryData, alloc_entries: &[AllocationLogEntry]) {
    let mut current_memory = 0i64;
    let mut peak_memory = 0i64;
    let mut log_allocations = 0u64;
    let mut deallocations = 0u64;
    let mut total_allocated = 0u64;
    let mut total_deallocated = 0u64;

    for entry in alloc_entries {
        match entry.size.cmp(&0) {
            Ordering::Greater => {
                log_allocations += 1;
                current_memory += entry.size;
                total_allocated += entry.size as u64;
            }
            Ordering::Less => {
                deallocations += 1;
                current_memory += entry.size; // Adding negative number
                total_deallocated += (-entry.size) as u64;
            }
            Ordering::Equal => (),
        }
        peak_memory = peak_memory.max(current_memory);
    }

    memory_data.total_deallocations = deallocations;
    memory_data.peak_memory = peak_memory.max(0) as u64;
    memory_data.current_memory = current_memory.max(0) as u64;

    if log_allocations != memory_data.total_allocations || deallocations > log_allocations {
        println!("\nAllocation Tracking Analysis:");
        println!(
            "  Profile shows: {} allocations",
            memory_data.total_allocations
        );
        println!("  Log shows:     {log_allocations} allocations, {deallocations} deallocations");
        println!("  Total bytes:   {total_allocated} allocated, {total_deallocated} deallocated");
    }
}

fn generate_memory_timeline(profile: &ProcessedProfile, file_path: &Path) -> ThagResult<()> {
    // Find and parse allocation log
    if let Some(log_path) = find_allocation_log(file_path) {
        let alloc_entries = parse_allocation_log(&log_path)?;

        // Debug print
        println!("Found {} allocation entries", alloc_entries.len());

        // Calculate timeline data
        let mut cumulative_memory = 0i64;
        let mut timeline_points = Vec::new();
        let mut peak_memory = 0i64;
        let mut min_memory = 0i64;

        for (index, entry) in alloc_entries.iter().enumerate() {
            cumulative_memory += entry.size;
            peak_memory = peak_memory.max(cumulative_memory);
            min_memory = min_memory.min(cumulative_memory);
            timeline_points.push((index as i32, cumulative_memory));

            // Debug first few points and any negative values
            if index < 5 || cumulative_memory < 0 {
                println!("Point {}: {} bytes", index, cumulative_memory);
            }
        }

        // Adjust all points to be non-negative
        if min_memory < 0 {
            let offset = min_memory.abs();
            for (_x, y) in &mut timeline_points {
                *y += offset;
            }
            peak_memory += offset;
            println!(
                "Adjusted for negative values by adding offset of {} bytes",
                offset
            );
        }

        if timeline_points.is_empty() {
            return Err(ThagError::Profiling(
                "No memory allocation data available".to_string(),
            ));
        }

        // SVG dimensions and layout
        let width = 1200i32;
        let height = 600i32;
        let padding = 60i32;
        let plot_width = width - 2 * padding;
        let plot_height = height - 2 * padding;

        // Generate y-axis labels (memory scale)
        let memory_labels = generate_memory_scale_labels(peak_memory);
        let memory_points = generate_scale_points(plot_height, memory_labels.len());

        // Generate x-axis labels (time points)
        let time_labels = generate_time_scale_labels(timeline_points.len());
        let time_points = generate_scale_points(plot_width, time_labels.len());

        let path_data = timeline_points
            .iter()
            .map(|(x, y)| {
                // Fix x-coordinate scaling
                let x_pos = padding + ((x * plot_width) / timeline_points.len().max(1) as i32);
                println!("Point {}: x={}, x_pos={}", x, x, x_pos); // Debug x scaling
                let y_pos =
                    height - padding - ((y * plot_height as i64) / peak_memory.max(1)) as i32;
                if *x == 0 {
                    format!("M {x_pos},{y_pos}")
                } else {
                    format!("L {x_pos},{y_pos}")
                }
            })
            .collect::<String>();

        println!(
            "First 100 chars of path data: {}",
            &path_data[..path_data.len().min(100)]
        );

        println!(
            "Last 100 chars of path data: {}",
            &path_data[path_data.len() - 100..path_data.len()]
        );

        // Add debug visualization for first few points
        let debug_points = timeline_points
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, (x, y))| {
                let x_pos = padding + (x * plot_width / timeline_points.len() as i32);
                let y_pos =
                    height - padding - ((y * plot_height as i64) / peak_memory.max(1)) as i32;
                format!(
                    r#"<circle cx="{}" cy="{}" r="4" fill="{}"/>
                           <text x="{}" y="{}" class="label">{} bytes</text>"#,
                    x_pos,
                    y_pos,
                    if i == 0 { "red" } else { "blue" },
                    x_pos + 5,
                    y_pos,
                    y
                )
            })
            .collect::<String>();

        // Create SVG content
        let svg = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
        <svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">
            <style>
                .axis {{ stroke: #333; stroke-width: 1 }}
                .grid {{ stroke: #ccc; stroke-width: 0.5; opacity: 0.5 }}
                .line {{ stroke: #2196F3; stroke-width: 2; fill: none }}
                .peak {{ stroke: #F44336; stroke-width: 1; stroke-dasharray: 5,5 }}
                .label {{ font-family: Arial; font-size: 12px; }}
                .title {{ font-family: Arial; font-size: 16px; font-weight: bold; }}
            </style>

            <!-- Title -->
            <text x="{}" y="30" class="title" text-anchor="middle">Memory Usage Timeline</text>

            <!-- Grid lines -->
            {}

            <!-- Memory usage line -->
            <path class="line" d="{}"/>

            <!-- Debug visualization -->
            {}

            <!-- Peak memory line -->
            <line class="peak"
                      x1="{padding}"
                      y1="{}"
                      x2="{}"
                      y2="{}" />

            <!-- Axes -->
            <g class="axis">
                <line x1="{padding}" y1="{}" x2="{}" y2="{}" />
                <line x1="{padding}" y1="{}" x2="{padding}" y2="{padding}" />
            </g>

            <!-- Axis labels -->
            {}
        </svg>"#,
            // Format arguments in order:
            width / 2, // Title x position
            generate_grid_lines(
                padding,
                plot_width,
                plot_height,
                &memory_points,
                &time_points
            ), // Grid lines
            // Memory usage line path data
            // Generate path data
            timeline_points
                .iter()
                .map(|(x, y)| {
                    let x_pos = padding + ((x * plot_width) / timeline_points.len().max(1) as i32);
                    let y_pos = calculate_y_position(*y, peak_memory, height, padding, plot_height);
                    if *x == 0 {
                        format!("M {x_pos},{y_pos}")
                    } else {
                        format!("L {x_pos},{y_pos}")
                    }
                })
                .collect::<String>(),
            // Debug points
            timeline_points
                .iter()
                .take(5)
                .enumerate()
                .map(|(i, (x, y))| {
                    let x_pos = padding + ((x * plot_width) / timeline_points.len().max(1) as i32);
                    let y_pos = calculate_y_position(*y, peak_memory, height, padding, plot_height);
                    format!(
                        r#"<circle cx="{}" cy="{}" r="4" fill="{}"/>
                               <text x="{}" y="{}" class="label">{} bytes</text>"#,
                        x_pos,
                        y_pos,
                        if i == 0 { "red" } else { "blue" },
                        x_pos + 5,
                        y_pos,
                        y
                    )
                })
                .collect::<String>(),
            // Peak line y-coordinate
            calculate_y_position(peak_memory, peak_memory, height, padding, plot_height),
            width - padding,
            calculate_y_position(peak_memory, peak_memory, height, padding, plot_height),
            // Axis coordinates
            height - padding, // x-axis y1
            width - padding,  // x-axis x2
            height - padding, // x-axis y2
            height - padding, // y-axis y1
            // Axis labels
            generate_axis_labels(
                padding,
                height,
                plot_width,
                &memory_labels,
                &memory_points,
                &time_labels,
                &time_points
            )
        );

        fs::write("memory-timeline.svg", svg)?;
        println!("Memory timeline generated: memory-timeline.svg");

        // Fix the error handling for open_in_browser
        open_in_browser("memory-timeline.svg").map_err(|e| ThagError::Profiling(e.to_string()))?;
    } else {
        return Err(ThagError::Profiling("No allocation log found".to_string()));
    }
    Ok(())
}

fn generate_memory_scale_labels(peak_memory: i64) -> Vec<String> {
    let step = match peak_memory {
        0..=1_000 => 100,
        1_001..=10_000 => 1_000,
        10_001..=100_000 => 10_000,
        100_001..=1_000_000 => 100_000,
        _ => 1_000_000,
    };

    let mut labels = Vec::new();
    let mut current = 0;
    while current <= peak_memory {
        labels.push(format_memory_size(current));
        current += step;
    }
    labels
}

fn format_memory_size(size: i64) -> String {
    if size >= 1_000_000 {
        format!("{}MB", size / 1_000_000)
    } else if size >= 1_000 {
        format!("{}KB", size / 1_000)
    } else {
        format!("{}B", size)
    }
}

fn generate_time_scale_labels(total_points: usize) -> Vec<String> {
    let step = match total_points {
        0..=10 => 1,
        11..=100 => 10,
        101..=1000 => 100,
        _ => 1000,
    };

    let mut labels = Vec::new();
    let mut current = 0;
    while current < total_points {
        labels.push(current.to_string());
        current += step;
    }
    labels
}

fn generate_scale_points(length: i32, num_divisions: usize) -> Vec<i32> {
    (0..num_divisions)
        .map(|i| (i * length as usize) / (num_divisions - 1))
        .map(|p| p as i32)
        .collect()
}

fn generate_grid_lines(
    padding: i32,
    plot_width: i32,
    plot_height: i32,
    memory_points: &[i32],
    time_points: &[i32],
) -> String {
    let mut grid = String::new();

    // Horizontal grid lines
    for &y in memory_points {
        grid.push_str(&format!(
            r#"<line class="grid" x1="{}" y1="{}" x2="{}" y2="{}" />"#,
            padding,
            y + padding,
            padding + plot_width,
            y + padding
        ));
    }

    // Vertical grid lines
    for &x in time_points {
        grid.push_str(&format!(
            r#"<line class="grid" x1="{}" y1="{}" x2="{}" y2="{}" />"#,
            x + padding,
            padding,
            x + padding,
            padding + plot_height
        ));
    }

    grid
}

fn generate_axis_labels(
    padding: i32,
    height: i32,
    plot_width: i32,
    memory_labels: &[String],
    memory_points: &[i32],
    time_labels: &[String],
    time_points: &[i32],
) -> String {
    let mut labels = String::new();

    // Y-axis (memory) labels - reverse the order to have 0 at bottom
    for (i, (label, &y)) in memory_labels.iter().zip(memory_points).enumerate() {
        let y_pos = height - padding - y; // Flip the y-coordinate
        labels.push_str(&format!(
                r#"<text x="{}" y="{}" class="label" text-anchor="end" alignment-baseline="middle">{}</text>"#,
                padding - 5,
                y_pos,
                label
            ));
    }

    // X-axis (time) labels
    for (label, &x) in time_labels.iter().zip(time_points) {
        labels.push_str(&format!(
            r#"<text x="{}" y="{}" class="label" text-anchor="middle">{}</text>"#,
            x + padding,
            height - padding + 20,
            label
        ));
    }

    // Axis titles
    labels.push_str(&format!(
        r#"<text x="{}" y="{}" class="label" text-anchor="middle">Time (events)</text>"#,
        padding + plot_width / 2,
        height - 10
    ));
    labels.push_str(&format!(
        r#"<text x="{}" y="{}" class="label" text-anchor="middle" transform="rotate(-90,{},{})">{}</text>"#,
        15,
        height / 2,
        15,
        height / 2,
        "Memory Usage"
    ));

    labels
}

fn calculate_y_position(value: i64, peak: i64, height: i32, padding: i32, plot_height: i32) -> i32 {
    // Convert value to y-coordinate (0 at bottom)
    height - padding - ((value * plot_height as i64) / peak.max(1)) as i32
}
