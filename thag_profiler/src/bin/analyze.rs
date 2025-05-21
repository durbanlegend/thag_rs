use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use inferno::flamegraph::{
    self,
    color::{BasicPalette, MultiPalette},
    Options, Palette,
};
use inline_colorization::{color_cyan, color_reset};
use inquire::{InquireError, MultiSelect, Select};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    string::ToString,
    time::Duration,
};
use strum::Display;
use thag_profiler::{profiling::ProfileStats, regex, thousands, ProfileError, ProfileResult};

#[derive(Debug, Default, Clone)]
pub struct ProcessedProfile {
    pub path: PathBuf,
    pub stacks: Vec<String>,
    pub title: String,
    pub subtitle: String,
    pub duration: u32,
    pub timestamp: DateTime<Local>,
    pub profile_type: ProfileType,
    pub memory_data: Option<MemoryData>,
    pub memory_events: Vec<MemoryEvent>,
}

#[derive(Debug, Default, Clone, Display)]
pub enum ProfileType {
    #[default]
    Time,
    Memory,
}

// Keep existing MemoryData struct
#[derive(Debug, Default, Clone)]
pub struct MemoryData {
    pub dealloc: bool, // true for a deallocation file

    // Allocation counts
    pub allocation_count: usize,   // Number of allocation operations
    pub deallocation_count: usize, // Number of deallocation operations

    // Bytes tracked
    pub bytes_allocated: u64,   // Total bytes from allocations
    pub bytes_deallocated: u64, // Total bytes from deallocations

    // Memory state
    pub peak_memory: u64,    // Maximum memory in use at any point
    pub current_memory: u64, // Current memory in use

    // Size distribution
    pub allocation_sizes: HashMap<usize, usize>, // Size -> Count of allocations
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemoryEvent {
    timestamp: u128,
    delta: usize,
    operation: char,
    stack: Vec<String>,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err("Usage: analyze <directory>".to_string().into());
    }

    // Directory for .folded files
    let dir_path = PathBuf::from(args[1].clone());

    // Ensure profiling is disabled for the analyzer
    // Only takes effect if this tool is compiled (`thag tools/thag_profile.rs -x`).
    // profiling::disable_profiling();
    loop {
        let analysis_types = vec![
            "Time Profile - Single",
            "Time Profile - Differential",
            "Memory Profile - Single",
            "Memory Profile - Differential",
            "Exit",
        ];

        let analysis_type =
            Select::new("Select analysis type:", analysis_types).prompt_skippable()?;

        match analysis_type {
            Some("Exit") => break,
            Some("Time Profile - Single") => analyze_type(AnalysisType::TimeSingle, &dir_path)?,
            Some("Time Profile - Differential") => {
                analyze_type(AnalysisType::TimeDifferential, &dir_path)?;
            }
            Some("Memory Profile - Single") => analyze_type(AnalysisType::MemorySingle, &dir_path)?,
            Some("Memory Profile - Differential") => {
                analyze_type(AnalysisType::MemoryDifferential, &dir_path)?;
            }
            Some(_) => unreachable!(),
            None => {
                println!("Exiting");
                return Ok(());
            }
        }
    }

    Ok(())
}

// Define analysis types for menu hierarchy
#[derive(Debug, Clone, Copy)]
enum AnalysisType {
    TimeSingle,
    TimeDifferential,
    MemorySingle,
    MemoryDifferential,
    #[allow(dead_code)]
    Exit,
}

fn analyze_type(analysis_type: AnalysisType, dir_path: &PathBuf) -> ProfileResult<()> {
    let filter = match analysis_type {
        AnalysisType::TimeSingle | AnalysisType::TimeDifferential => {
            |f: &str| -> bool { !f.contains("-memory") }
        }
        AnalysisType::MemorySingle | AnalysisType::MemoryDifferential => {
            |f: &str| -> bool { f.contains("-memory") }
        }
        AnalysisType::Exit => return Ok(()),
    };

    loop {
        let maybe_profile_group = group_profile_files(dir_path, filter)?;

        let Some(profile_group) = maybe_profile_group else {
            return Ok(());
        };

        if profile_group.is_empty() {
            println!("No {analysis_type:?} profile files found.");
            return Ok(());
        }

        match analysis_type {
            AnalysisType::TimeSingle => analyze_single_time_profile(&profile_group)?,
            AnalysisType::TimeDifferential => {
                analyze_differential_profiles(&profile_group, &ProfileType::Time)?
            }
            AnalysisType::MemorySingle => analyze_single_memory_profile(&profile_group)?,
            AnalysisType::MemoryDifferential => {
                analyze_differential_profiles(&profile_group, &ProfileType::Memory)?
            }
            AnalysisType::Exit => return Ok(()),
        }
    }
}

fn analyze_single_time_profile(profile_group: &FileGroup) -> ProfileResult<()> {
    match select_profile_file(&profile_group)? {
        None => Ok(()), // User selected "Back"
        Some(file_path) => {
            let processed = read_and_process_profile(&file_path)?;
            let stats = build_time_stats(&processed)?;

            let inclusive = file_path.display().to_string().contains("inclusive");
            let options = if inclusive {
                vec![
                    "Show Statistics By Total Time",
                    "Show Statistics By Calls",
                    "Back to Profile Selection",
                ]
            } else {
                vec![
                    "Show Aggregated Execution Timeline (Flamegraph)",
                    "...Filter Aggregated Functions (Recursive or Exact Match)",
                    "Show Individual Sequential Execution Timeline (Flamechart)",
                    "...Filter Individual Sequential Functions (Recursive or Exact Match)",
                    "Show Statistics By Total Time",
                    "Show Statistics By Calls",
                    "Back to Profile Selection",
                ]
            };
            loop {
                let maybe_action =
                    Select::new("Select action:", options.clone()).prompt_skippable()?;

                let action = match maybe_action {
                    Some(action) => action,
                    None => {
                        return Ok(());
                    }
                };

                match action {
                    "Back to Profile Selection" => break,
                    "Show Aggregated Execution Timeline (Flamegraph)" => {
                        generate_time_flamegraph(&processed, false)?;
                    }
                    "...Filter Aggregated Functions (Recursive or Exact Match)" => {
                        filter_functions(&processed)?.map_or_else(
                            || Ok(()),
                            |filtered| generate_time_flamegraph(&filtered, false),
                        )?;
                    }
                    "Show Individual Sequential Execution Timeline (Flamechart)" => {
                        generate_time_flamegraph(&processed, true)?;
                    }
                    "...Filter Individual Sequential Functions (Recursive or Exact Match)" => {
                        filter_functions(&processed)?.map_or_else(
                            || Ok(()),
                            |filtered| generate_time_flamegraph(&filtered, true),
                        )?;
                    }
                    "Show Statistics By Total Time" => {
                        let inclusive = file_path.display().to_string().contains("inclusive");
                        show_statistics(&stats, &processed, inclusive, true);
                    }
                    "Show Statistics By Calls" => {
                        let inclusive = file_path.display().to_string().contains("inclusive");
                        show_statistics(&stats, &processed, inclusive, false);
                    }
                    // _ => return Ok(()),
                    _ => println!("Unknown option"),
                }
            }
            Ok(())
        }
    }
}

fn analyze_differential_profiles(
    profile_group: &FileGroup,
    profile_type: &ProfileType,
) -> ProfileResult<()> {
    match select_profile_files(profile_group) {
        Ok(Some((before, after))) => {
            generate_differential_flamegraph(profile_type, &before, &after)?
        }
        Ok(None) => {
            eprintln!("No selection made");
            return Ok(());
        }
        Err(e) => eprintln!("{e}"),
    }
    Ok(())
}

fn analyze_single_memory_profile(profile_group: &FileGroup) -> ProfileResult<()> {
    match select_profile_file(&profile_group)? {
        None => Ok(()), // User selected "Back"
        Some(selected_file) => {
            let processed = read_and_process_profile(&selected_file)?;

            // eprintln!("processed.memory_data={:#?}", processed.memory_data);
            let alloc_type = if let Some(memory_data) = processed.memory_data.as_ref() {
                if memory_data.dealloc {
                    "Deallocation"
                } else {
                    "Allocation"
                }
            } else {
                eprintln!("\nNo memory data found in file.\nPlease make another selection");
                println!("\nPress Enter to continue...");
                let _ = std::io::stdin().read_line(&mut String::new());
                return Ok(());
            };

            let size_distribution_option = format!("Show {alloc_type} Size Distribution");

            loop {
                let options = vec![
                    "Show Aggregated Memory Profile (Flamegraph)",
                    "...Filter Aggregated Functions (Recursive or Exact Match)",
                    "Show Individual Sequential Memory Profile (Flamechart)",
                    "...Filter Individual Sequential Functions (Recursive or Exact Match)",
                    "Show Memory Statistics",
                    &size_distribution_option,
                    "Back to Profile Selection",
                ];

                // Show memory-specific menu and handle selection...
                let maybe_selection = Select::new("Select action:", options).prompt_skippable()?;

                let selection = match maybe_selection {
                    Some(selection) => selection,
                    None => return Ok(()),
                };

                match selection {
                    "Back to Profile Selection" => break,
                    "Show Aggregated Memory Profile (Flamegraph)" => {
                        generate_memory_flamegraph(&processed, false)
                            .map_or_else(|e| println!("{e}"), |()| {});
                    }
                    "...Filter Aggregated Functions (Recursive or Exact Match)" => {
                        filter_memory_patterns(&processed).map_or_else(
                            |e| println!("{e}"),
                            |maybe_filtered| {
                                maybe_filtered.map_or_else(
                                    || println!("Could not find matching profile"),
                                    |filtered| {
                                        generate_memory_flamegraph(&filtered, false)
                                            .map_or_else(|e| println!("{e}"), |()| {});
                                    },
                                );
                            },
                        );
                    }
                    "Show Individual Sequential Memory Profile (Flamechart)" => {
                        generate_memory_flamegraph(&processed, true)
                            .map_or_else(|e| println!("{e}"), |()| {});
                    }
                    "...Filter Individual Sequential Functions (Recursive or Exact Match)" => {
                        filter_memory_patterns(&processed).map_or_else(
                            |e| println!("{e}"),
                            |maybe_filtered| {
                                maybe_filtered.map_or_else(
                                    || println!("Could not find matching profile"),
                                    |filtered| {
                                        generate_memory_flamegraph(&filtered, true)
                                            .map_or_else(|e| println!("{e}"), |()| {});
                                    },
                                );
                            },
                        );
                    }
                    "Show Memory Statistics" => show_memory_statistics(&processed),
                    #[allow(unused_variables)]
                    size_distribution_option => show_allocation_distribution(&processed)
                        .map_or_else(|e| println!("{e}"), |()| {}),
                }
            }
            Ok(())
        }
    }
}

fn generate_time_flamegraph(profile: &ProcessedProfile, as_chart: bool) -> ProfileResult<()> {
    if profile.stacks.is_empty() {
        return Err(ProfileError::General(
            "No profile data available".to_string(),
        ));
    }

    let Some(color_scheme) = select_color_scheme(&ProfileType::Time)? else {
        return Ok(());
    };

    let chart_type = ChartType::TimeSequence;
    let svg = if as_chart {
        "flamechart.svg"
    } else {
        "flamegraph.svg"
    };
    let output = File::create(svg)?;
    let mut opts = Options::default();
    chart_type.configure_options(&mut opts);
    opts.title = if as_chart {
        "Execution Timeline Flamechart (Individual)".to_string()
    } else {
        "Execution Timeline Flamegraph (Aggregated)".to_string()
    };
    opts.subtitle = Some(format!(
        "{}  Started:  {}  Total sec: {:.3}",
        profile.subtitle,
        profile.timestamp.format("%Y-%m-%d %H:%M:%S"),
        f64::from(profile.duration) / 1_000_000_f64
    ));
    // opts.notes = profile.subtitle.clone();
    opts.colors = color_scheme;
    "μs".clone_into(&mut opts.count_name);
    opts.min_width = 0.0;
    // opts.color_diffusion = true;
    opts.flame_chart = as_chart;

    flamegraph::from_lines(
        &mut opts,
        profile.stacks.iter().rev().map(String::as_str),
        output,
    )?;

    enhance_svg_accessibility(svg)?;

    println!(
        "Flame {} generated: {svg}",
        if as_chart { "chart" } else { "graph" }
    );
    open_in_browser(svg).map_err(|e| ProfileError::General(e.to_string()))?;
    Ok(())
}

fn generate_differential_flamegraph(
    profile_type: &ProfileType,
    before: &PathBuf,
    after: &PathBuf,
) -> ProfileResult<()> {
    // First, generate the differential data
    let mut diff_data = Vec::new();
    inferno::differential::from_files(
        inferno::differential::Options::default(), // Options for differential processing
        before,
        after,
        &mut diff_data,
    )
    .map_err(|e| ProfileError::General(e.to_string()))?;

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
    opts.title = format!("Differential {profile_type} Profile: {script_name}");
    opts.subtitle = format!("Comparing {before_name} → {after_name}").into();
    opts.colors = match profile_type {
        ProfileType::Time => select_color_scheme(&ProfileType::Time)?.unwrap_or_default(),
        ProfileType::Memory => select_color_scheme(&ProfileType::Memory)?.unwrap_or_default(),
    };
    match profile_type {
        ProfileType::Time => "μs",
        ProfileType::Memory => "bytes",
    }
    .clone_into(&mut opts.count_name);
    opts.flame_chart = false;

    // Convert diff_data to lines
    let diff_lines =
        String::from_utf8(diff_data).map_err(|e| ProfileError::General(e.to_string()))?;
    let lines: Vec<&str> = diff_lines.lines().collect();

    flamegraph::from_lines(&mut opts, lines.iter().copied(), output)
        .map_err(|e| ProfileError::General(e.to_string()))?;

    enhance_svg_accessibility(svg)?;

    println!("\nDifferential flame graph generated: flamegraph-diff.svg");
    println!("Red indicates increased time, blue indicates decreased time");
    println!("The width of the boxes represents the absolute time difference");

    open_in_browser("flamegraph-diff.svg").map_err(|e| ProfileError::General(e.to_string()))?;
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

fn show_statistics(
    stats: &ProfileStats,
    profile: &ProcessedProfile,
    inclusive: bool,
    by_total_time: bool,
) {
    println!("\n{}", profile.title);
    println!("{}", profile.subtitle);
    println!(
        "\nStarted: {}",
        profile.timestamp.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "\nFunction Statistics ({color_cyan}{}CLUSIVE{color_reset} of Children) Ranked by {}:",
        if inclusive { "IN" } else { "EX" },
        if by_total_time { "Total Time" } else { "Calls" },
    );
    println!(
        "{color_cyan}{}{color_reset}",
        "═".repeat(if by_total_time { 65 } else { 60 })
    );

    if by_total_time {
        let mut entries: Vec<_> = stats.total_time.iter().collect();
        entries.sort_by_key(|(_, &total_time)| std::cmp::Reverse(total_time));

        for (func, &total_time) in entries {
            let calls = *stats.calls.get(func).unwrap_or(&0);
            show_stats(func, total_time, calls);
        }
    } else {
        let mut entries: Vec<_> = stats.calls.iter().collect();
        entries.sort_by_key(|(_, &calls)| std::cmp::Reverse(calls));

        for (func, &calls) in entries {
            let total_time = *stats.total_time.get(func).unwrap_or(&0);
            show_stats(func, total_time, calls);
        }
    }
}

fn show_stats(func: &str, total_time: u128, calls: u64) {
    let avg_time = if calls > 0 {
        total_time / u128::from(calls)
    } else {
        0
    };
    println!(
        "{:>10} calls {:>14} μs total {:>14} μs avg     {func}",
        thousands(calls),
        thousands(total_time),
        thousands(avg_time)
    );
}

fn filter_functions(processed: &ProcessedProfile) -> ProfileResult<Option<ProcessedProfile>> {
    // Get unique top-level functions from the stacks, not counting `main`.
    let functions: HashSet<_> = processed
        .stacks
        .iter()
        .filter_map(|line| {
            line.split(';')
                .find(|path| !path.ends_with("::main"))
                .map(|s| s.split_whitespace().next().unwrap_or(""))
                .filter(|s| !s.is_empty())
        })
        .collect();

    let mut function_list: Vec<_> = functions.into_iter().collect();
    function_list.sort_unstable();

    // Display information about filtering modes
    println!("\nThere are two filtering modes available:");
    println!("  1. Recursive: Removes functions and all their child calls (original behavior)");
    println!(
        "     For example, filtering out 'main' will remove main and everything called by main"
    );
    println!("  2. Exact Match: Removes functions ONLY as standalone entries but preserves them when they have children");
    println!("     For example, filtering out 'process_data' will remove standalone 'process_data' entries");
    println!("     but will keep 'process_data;parse_json' and 'process_data;validate' entries\n");

    // Ask user to select filtering mode
    let maybe_filter_mode = inquire::Select::new(
        "Select filtering mode:",
        vec![
            "Recursive (filter out function and ALL its children)",
            "Exact Match (filter out function only when it has NO children)",
        ],
    )
    .prompt_skippable()?;

    let filter_mode = match maybe_filter_mode {
        Some(filter_mode) => filter_mode,
        None => return Ok(None),
    };

    let exact_match = filter_mode.starts_with("Exact Match");

    // Select functions to filter
    // Show which mode is selected
    println!(
        "\nUsing {} filtering mode",
        if exact_match {
            "Exact Match"
        } else {
            "Recursive"
        }
    );

    let maybe_to_filter =
        MultiSelect::new("Select functions to filter out:", function_list).prompt_skippable()?;

    let to_filter = match maybe_to_filter {
        Some(to_filter) => to_filter,
        None => return Ok(None),
    };

    eprintln!("to_filter={to_filter:#?}");
    // Apply appropriate filtering based on the chosen mode
    let filtered_stacks: Vec<String> = if exact_match {
        // In exact match mode, we need to keep any stack where the filtered function
        // has children (i.e., is part of a call chain)
        processed
            .stacks
            .iter()
            .filter(|line| {
                let count = line
                    .split_once(" ")
                    .unwrap_or(("", ""))
                    .0
                    .split(';')
                    .skip_while(|s| !to_filter.contains(s))
                    .filter(|s| !to_filter.contains(s))
                    // .inspect(|s| println!("s={s}"))
                    .count();

                count > 0
            })
            .cloned()
            .collect()
    } else {
        // Recursive mode (original behavior): filter out function and all its children
        processed
            .stacks
            .iter()
            .filter(|line| {
                let found = line
                    .split_once(" ")
                    .unwrap_or(("", ""))
                    .0
                    .split(';')
                    .any(|s| !to_filter.contains(&s));

                !found
            })
            .cloned()
            .collect()
    };

    let mut duration: u32 = 0;
    for line in &filtered_stacks {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let delta = parts[parts.len() - 1].parse::<u32>().unwrap_or(0_u32);
            duration += delta;
        }
    }

    Ok(Some(ProcessedProfile {
        stacks: filtered_stacks,
        duration,
        ..processed.clone() // Keep the metadata
    }))
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

fn load_last_used_time_scheme() -> ProfileResult<String> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| ProfileError::General("Could not find config directory".to_string()))?
        .join("thag")
        .join("time_flamegraph_colors.json");

    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let config: ColorSchemeConfig =
            serde_json::from_str(&content).map_err(|e| ProfileError::General(e.to_string()))?;
        Ok(config.last_used)
    } else {
        Ok(ColorSchemeConfig::default().last_used)
    }
}

fn load_last_used_memory_scheme() -> ProfileResult<String> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| ProfileError::General("Could not find config directory".to_string()))?
        .join("thag")
        .join("memory_flamegraph_colors.json");

    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let config: ColorSchemeConfig =
            serde_json::from_str(&content).map_err(|e| ProfileError::General(e.to_string()))?;
        Ok(config.last_used)
    } else {
        Ok(ColorSchemeConfig::default().last_used)
    }
}

fn save_time_color_scheme(name: &str) -> ProfileResult<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ProfileError::General("Could not find config directory".to_string()))?
        .join("thag");

    fs::create_dir_all(&config_dir)?;

    let config = ColorSchemeConfig {
        last_used: name.to_string(),
    };

    let config_path = config_dir.join("time_flamegraph_colors.json");
    fs::write(
        config_path,
        serde_json::to_string_pretty(&config).map_err(|e| ProfileError::General(e.to_string()))?,
    )?;

    Ok(())
}

fn save_memory_color_scheme(name: &str) -> ProfileResult<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ProfileError::General("Could not find config directory".to_string()))?
        .join("thag");

    fs::create_dir_all(&config_dir)?;

    let config = ColorSchemeConfig {
        last_used: name.to_string(),
    };

    let config_path = config_dir.join("memory_flamegraph_colors.json");
    fs::write(
        config_path,
        serde_json::to_string_pretty(&config).map_err(|e| ProfileError::General(e.to_string()))?,
    )?;

    Ok(())
}

fn select_color_scheme(profile_type: &ProfileType) -> ProfileResult<Option<Palette>> {
    let schemes = get_color_schemes();
    let last_used = match profile_type {
        ProfileType::Time => load_last_used_time_scheme()?,
        ProfileType::Memory => load_last_used_memory_scheme()?,
    };

    // First ask if user wants to use the last scheme or select a new one
    let maybe_use_last = inquire::Confirm::new(&format!(
        "Use last color scheme ({last_used})? (Press 'n' to select a different scheme)"
    ))
    .with_default(true)
    .prompt_skippable()?;

    let use_last = match maybe_use_last {
        Some(use_last) => use_last,
        None => return Ok(None),
    };

    if use_last {
        return Ok(Some(
            schemes
                .iter()
                .find(|s| s.name == last_used)
                .unwrap_or(&schemes[0])
                .palette,
        ));
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

    let maybe_selection = Select::new(
        "Select color scheme:",
        schemes.iter().map(|s| s.name).collect::<Vec<_>>(),
    )
    .prompt_skippable()?;

    let selection = match maybe_selection {
        Some(selection) => selection,
        None => return Ok(None),
    };

    // Save the selection
    match profile_type {
        ProfileType::Time => save_time_color_scheme(selection)?,
        ProfileType::Memory => save_memory_color_scheme(selection)?,
    };

    Ok(Some(
        schemes
            .iter()
            .find(|s| s.name == selection)
            .unwrap()
            .palette,
    ))
}

type FileGroup = Vec<(String, Vec<PathBuf>)>;

fn group_profile_files<T: Fn(&str) -> bool>(
    dir_path: &Path,
    filter: T,
) -> ProfileResult<Option<FileGroup>> {
    let all_groups = collect_profile_files(dir_path, &filter)?;
    let mut current_filter = String::new();

    loop {
        // Apply current filter
        let filtered_groups = if current_filter.is_empty() {
            all_groups.clone()
        } else {
            all_groups
                .clone()
                .into_iter()
                .filter_map(|(group_name, paths)| {
                    let filtered_paths: Vec<PathBuf> = paths
                        .into_iter()
                        .filter(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|name| name.contains(&current_filter))
                        })
                        .collect();

                    if filtered_paths.is_empty() {
                        None
                    } else {
                        Some((group_name, filtered_paths))
                    }
                })
                .collect()
        };

        // Display filter info if active
        if !current_filter.is_empty() {
            println!(
                "Current filter: '{}' (showing {} of {} groups)",
                current_filter,
                filtered_groups.len(),
                all_groups.len()
            );
        }

        // eprintln!("{:#?}", backtrace::Backtrace::new());
        // Display the current list of files
        println!("\nAvailable profile files:");
        for (i, (group_name, paths)) in filtered_groups.iter().enumerate() {
            println!("{}. {} ({} files)", i + 1, group_name, paths.len());
            // Optionally show the first few files in each group
            for (j, path) in paths.iter().take(3).enumerate() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    println!("   {}.{}: {}", i + 1, j + 1, filename);
                }
            }
            if paths.len() > 3 {
                println!("   ... and {} more", paths.len() - 3);
            }
        }
        println!();

        // Create selection options with Filter as the first option
        let mut options = vec!["Filter/modify selection".to_string()];

        // Add numbered options for each group
        for (i, (group_name, paths)) in filtered_groups.iter().enumerate() {
            options.push(format!("{}. {} ({} files)", i + 1, group_name, paths.len()));
        }

        let maybe_selection =
            inquire::Select::new("Select an option:", options).prompt_skippable()?;

        let selection = match maybe_selection {
            Some(selection) => selection,
            None => return Ok(None),
        };

        if selection == "Filter/modify selection" {
            // Show filter options
            let maybe_filter_action = inquire::Select::new(
                "Filter options:",
                vec!["Apply/modify filter", "Clear filter", "Back to selection"],
            )
            .prompt_skippable()?;

            let filter_action = match maybe_filter_action {
                Some(filter_action) => filter_action,
                None => return Ok(None),
            };

            match filter_action {
                "Apply/modify filter" => {
                    let maybe_current_filter = inquire::Text::new("Enter filter string:")
                        .with_initial_value(&current_filter)
                        .prompt_skippable()?;

                    current_filter = match maybe_current_filter {
                        Some(current_filter) => current_filter,
                        None => return Ok(None),
                    };
                }
                "Clear filter" => current_filter.clear(),
                _ => return Ok(None), // Back to selection
            }
        } else {
            // Extract the index from the selection string
            match selection.split('.').next() {
                Some(index_str) => {
                    match index_str.trim().parse::<usize>() {
                        Ok(index) => {
                            if index > 0 && index <= filtered_groups.len() {
                                // Return the selected group
                                return Ok(Some(vec![filtered_groups[index - 1].clone()]));
                            }
                        }
                        _ => return Ok(None), // Back to selection
                    }
                }
                _ => return Ok(None), // Back to selection
            }
        }
    }
}

// Helper function to collect all profile files matching the initial filter
fn collect_profile_files<T: Fn(&str) -> bool>(
    dir_path: &Path,
    filter: &T,
) -> ProfileResult<Vec<(String, Vec<PathBuf>)>> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // Use file_navigator to get the directory and list .folded files
    for entry in (dir_path.read_dir()?).flatten() {
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
    // Show in reverse chronological order
    result.sort_by(|(_a, ta), (_b, tb)| ta.cmp(tb));

    Ok(result)
}

fn select_profile_files(profile_group: &FileGroup) -> ProfileResult<Option<(PathBuf, PathBuf)>> {
    let files = &profile_group[0].1;

    if files.len() < 2 {
        return Err(ProfileError::General(
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

    let maybe_before =
        Select::new("Select 'before' profile:", file_options.clone()).prompt_skippable()?;

    let before = match maybe_before {
        Some(before) => before,
        None => return Ok(None),
    };

    // Create new options list excluding the 'before' selection
    let after_options: Vec<_> = file_options
        .into_iter()
        .filter(|name| name != &before)
        .collect();

    let maybe_after = Select::new("Select 'after' profile:", after_options).prompt_skippable()?;

    let after = match maybe_after {
        Some(after) => after,
        None => return Ok(None),
    };

    Ok(Some((
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
    )))
}

fn select_profile_file(profile_group: &[(String, Vec<PathBuf>)]) -> ProfileResult<Option<PathBuf>> {
    if profile_group.is_empty() {
        return Ok(None);
    }

    let mut file_options: Vec<_> = profile_group
        .iter()
        .flat_map(|(_, files)| files)
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    file_options.push("Back".to_string());

    let maybe_selected =
        Select::new("Select profile to analyze:", file_options).prompt_skippable()?;

    let selected = match maybe_selected {
        Some(selected) => selected,
        None => return Ok(None),
    };

    if selected == "Back" {
        return Ok(None);
    }

    // Find the actual PathBuf for the selected file
    for (_, files) in profile_group {
        if let Some(file) = files.iter().find(|f| f.to_string_lossy() == selected) {
            return Ok(Some(file.clone()));
        }
    }

    Ok(None)
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
fn read_and_process_profile(path: &PathBuf) -> ProfileResult<ProcessedProfile> {
    let input = File::open(path)?;
    let reader = BufReader::new(input);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let filename = path
        .file_name()
        .ok_or_else(|| ProfileError::General("Failed to get file name".to_string()))?
        .to_string_lossy()
        .to_string();

    let re = regex!(r#"^(\w+)\-(\d{8}\-\d{6})"#);
    let timestamp = if let Some(captures) = re.captures(&filename) {
        let _script_stem = captures.get(1).unwrap().as_str();
        let datetime_str = captures.get(2).unwrap().as_str();
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y%m%d-%H%M%S") {
            let local_dt = Local.from_local_datetime(&naive_dt).single().unwrap();
            // println!("Parsed datetime: {local_dt}");
            // println!("ISO format: {}", local_dt.to_rfc3339());
            // processed.timestamp = local_dt;
            local_dt
        } else {
            println!("Failed to parse datetime");
            DateTime::default()
        }
    } else {
        DateTime::default()
    };

    let subtitle = filename.clone();
    let mut processed = ProcessedProfile {
        path: path.clone(),
        profile_type: if filename.contains("memory.folded")
            || filename.contains("memory_detail.folded")
            || filename.contains("memory_detail_dealloc.folded")
        {
            ProfileType::Memory
        } else {
            ProfileType::Time
        },
        subtitle,
        timestamp,
        ..Default::default()
    };

    if matches!(processed.profile_type, ProfileType::Time) {
        processed.stacks = lines
            .iter()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .cloned()
            .collect();
    }

    for stack in &processed.stacks {
        let parts: Vec<&str> = stack.split_whitespace().collect();
        if parts.len() >= 2 {
            let delta = parts[parts.len() - 1].parse::<u32>().unwrap_or(0_u32);
            processed.duration += delta;
        }
    }

    // Process memory entries from folded file
    if matches!(processed.profile_type, ProfileType::Memory) {
        let entries: Vec<MemoryEvent> = lines
            .iter()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let stack = parts[..parts.len() - 1].join(" ");
                    if let Some(op_size) = parts.last() {
                        // Parse operation and size
                        let (operation, size) = if let Some(size) = op_size.parse::<i64>() {
                            if size >= 0 {
                                ('+', size)
                            } else {
                                ('-', size)
                            }
                        } else {
                            return None;
                        };

                        return Some(MemoryEvent {
                            timestamp: 0, // We might want to parse this from the log
                            delta: size.unsigned_abs() as usize,
                            operation,
                            stack: stack.split(';').map(str::to_string).collect(),
                        });
                    }
                }
                None
            })
            .collect();

        processed.memory_events = entries;

        if !processed.memory_events.is_empty() {
            let mut memory_data = MemoryData::default();
            let mut current_memory = 0u64;

            for memory_event in &processed.memory_events {
                current_memory += memory_event.delta as u64;
                memory_data.bytes_allocated += memory_event.delta as u64;
                memory_data.allocation_count += 1;
                // Track allocation size distribution
                *memory_data
                    .allocation_sizes
                    .entry(memory_event.delta)
                    .or_default() += 1;
                // } else if memory_event.operation == '-' {
                // }
                memory_data.peak_memory = memory_data.peak_memory.max(current_memory);
                // Since each allocation comes from Profile::drop, as a first-order approximation
                // we can assume they were deallocated, unless we're misattributing due to duplicate
                // instances of the same function running in parallel with both leaky and non-leaky
                // code paths .
                current_memory -= memory_event.delta as u64;
            }

            // eprintln!(
            //     "memory_data.allocation_sizes={:#?}",
            //     memory_data.allocation_sizes
            // );
            memory_data.current_memory = current_memory;
            memory_data.dealloc = path.display().to_string().contains("dealloc");

            processed.memory_data = Some(memory_data);
        }

        // Store original entries for later comparison
        processed.stacks = processed
            .memory_events
            .clone()
            .into_iter()
            .map(|memory_event| format!("{} {}", memory_event.stack.join(";"), memory_event.delta))
            .collect();
    }

    Ok(processed)
}

fn build_time_stats(processed: &ProcessedProfile) -> ProfileResult<ProfileStats> {
    let mut stats = ProfileStats::default();
    for line in &processed.stacks {
        if let Some((stack, time)) = line.rsplit_once(' ') {
            if let Ok(duration) = time.parse::<u128>() {
                stats.record(
                    stack,
                    Duration::from_micros(
                        u64::try_from(duration)
                            .map_err(|e| ProfileError::General(e.to_string()))?,
                    ),
                );
            }
        }
    }
    Ok(stats)
}

fn generate_memory_flamegraph(profile: &ProcessedProfile, as_chart: bool) -> ProfileResult<()> {
    if profile.stacks.is_empty() {
        return Err(ProfileError::General(
            "No memory profile data available".to_string(),
        ));
    }

    let Some(color_scheme) = select_color_scheme(&ProfileType::Memory)? else {
        return Ok(());
    };

    let memory_data = profile
        .memory_data
        .as_ref()
        .ok_or_else(|| ProfileError::General("No memory statistics available".to_string()))?;

    let svg = if as_chart {
        "memory_flamechart.svg"
    } else {
        "memory_flamegraph.svg"
    };

    let output = File::create(svg)?;

    let mut opts = Options::default();
    opts.title = if as_chart {
        "Memory Profile Flamechart (Individual)".to_string()
    } else {
        "Memory Profile Flamegraph (Aggregated)".to_string()
    };

    let alloc_type = if memory_data.dealloc {
        "Deallocated"
    } else {
        "Allocated"
    };

    opts.subtitle = Some(format!(
        "{}  Started: {}  Total Bytes {alloc_type}: {} Peak: {}",
        profile.subtitle,
        profile.timestamp.format("%Y-%m-%d %H:%M:%S"),
        thousands(memory_data.bytes_allocated),
        thousands(memory_data.peak_memory),
    ));
    opts.colors = color_scheme;
    "bytes".clone_into(&mut opts.count_name);
    opts.min_width = 0.001;
    opts.flame_chart = as_chart;

    flamegraph::from_lines(
        &mut opts,
        profile.stacks.iter().rev().map(String::as_str),
        output,
    )?;

    enhance_svg_accessibility(svg)?;
    println!("Memory flame chart generated: memory_flamechart.svg");
    open_in_browser(svg).map_err(|e| ProfileError::General(e.to_string()))?;
    Ok(())
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::type_complexity
)]
fn analyze_allocation_sites(profile: &ProcessedProfile) -> Vec<(String, usize)> {
    let mut total_allocs: HashMap<String, usize> = HashMap::new();

    // Process lines directly without creating intermediate MemoryEvents
    for event in &profile.memory_events {
        // Explicitly specify Result type
        let parse_result: Option<(char, i64)> = match (event.operation, event.delta as i64) {
            ('+', s) => Some(('+', s)),
            ('-', s) => Some(('-', s)),
            _ => None,
        };
        // eprintln!("parse_result: {:?}", parse_result);
        if let Some((operation, size)) = parse_result {
            let delta = size.unsigned_abs() as usize;

            if operation == '+' {
                *total_allocs.entry(event.stack.join("\n\t-> ")).or_default() += delta;
            }
        }
    }

    let mut total_sites: Vec<_> = total_allocs.into_iter().collect();
    total_sites.sort_by(|a, b| b.1.cmp(&a.1));

    total_sites
}

// Add pattern analysis to memory statistics display
#[allow(clippy::cast_sign_loss)]
fn show_memory_statistics(profile: &ProcessedProfile) {
    if let Some(memory_data) = &profile.memory_data {
        let memory_data = memory_data.clone();
        let alloc_type = if memory_data.dealloc {
            "Deallocation"
        } else {
            "Allocation"
        };

        println!("Peak Memory Usage:    {} bytes", memory_data.peak_memory);
        println!("Current Memory Usage: {} bytes", memory_data.current_memory);

        // Show top allocation sites from profile data
        let total_sites = analyze_allocation_sites(profile);

        let heading = format!("Top {alloc_type} Sites (Total {alloc_type}s)");
        println!("\n{heading}");
        println!("{}", "━".repeat(heading.len()));
        for (stack, size) in total_sites.iter().take(15) {
            if *size == 0 {
                break;
            }
            println!("{:>12} bytes: {stack}\n", thousands(size));
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn show_allocation_distribution(profile: &ProcessedProfile) -> ProfileResult<()> {
    let memory_data = profile
        .memory_data
        .as_ref()
        .ok_or_else(|| ProfileError::General("No memory statistics available".to_string()))?;

    let alloc_type = if memory_data.dealloc {
        "Deallocation"
    } else {
        "Allocation"
    };

    if memory_data.allocation_sizes.is_empty() {
        println!("No {alloc_type} data available.");
        return Ok(());
    }

    let heading = format!("{alloc_type} Size Distribution");
    println!("\n{heading}");
    println!("{}", "━".repeat(heading.len()));

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
    let mut bucket_totals: HashMap<&str, u64> = HashMap::new();

    let mut total_bytes = 0u64;

    for (&size, &count) in &memory_data.allocation_sizes {
        let count = count as u64;

        let bytes = size as u64 * count;
        total_bytes += bytes;

        for &(min, max, label) in &buckets {
            if size >= min && size <= max {
                *bucket_counts.entry(label).or_default() += count;
                *bucket_totals.entry(label).or_default() += bytes;
                break;
            }
        }
    }

    // eprintln!("bucket_counts={bucket_counts:#?}\nbucket_totals={bucket_totals:#?}");

    // Calculate max count for bar scaling
    let max_count = bucket_counts.values().max().copied().unwrap_or(1);

    // Calculate max total for bar scaling
    let max_total = bucket_totals.values().max().copied().unwrap_or(1);

    // Display distribution with bars
    println!(
        "{:>8} │ {:>6} │ {:<50}   {:>14} │ {:<50}",
        "Bucket", "Count", "Count Graph", "Size (bytes)", "Size Graph"
    );
    println!(
        "{:>8} │ {:>6} │ {:<50}   {:>14} │ {:<50}",
        "─".repeat(8),
        "─".repeat(6),
        "─".repeat(50),
        "─".repeat(14),
        "─".repeat(50),
    );
    for &(_, _, label) in &buckets {
        let count = bucket_counts.get(label).copied().unwrap_or(0);
        let count_bar_length = ((count as f64 / max_count as f64) * 50.0) as usize;
        let total = bucket_totals.get(label).copied().unwrap_or(0);
        let total_bar_length = ((total as f64 / max_total as f64) * 50.0) as usize;
        println!(
            "{label:>8} │ {count:>6} │ {:<50}   {:>14} │ {:<50}",
            "█".repeat(count_bar_length),
            thousands(total),
            "█".repeat(total_bar_length)
        );
    }

    assert_eq!(total_bytes, memory_data.bytes_allocated);

    let alloc_type = if memory_data.dealloc {
        "deallocated"
    } else {
        "allocated"
    };

    println!(
        "\nTotal memory {alloc_type}: {} bytes",
        memory_data.bytes_allocated
    );
    println!("Total stack frames: {}", memory_data.allocation_count);
    if memory_data.allocation_count > 0 {
        println!(
            "Average {alloc_type} per stack frame: {} bytes",
            memory_data.bytes_allocated / memory_data.allocation_count as u64
        );
    }

    Ok(())
}

#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn filter_memory_patterns(profile: &ProcessedProfile) -> ProfileResult<Option<ProcessedProfile>> {
    // First provide memory pattern selection
    let patterns = vec![
        "Large allocations (>1MB)",
        "Temporary allocations",
        "Leaked memory",
        "Frequent allocations",
        "Custom pattern...",
    ];

    let maybe_selected_patterns =
        MultiSelect::new("Select memory patterns to filter out:", patterns).prompt_skippable()?;

    let selected_patterns = match maybe_selected_patterns {
        Some(selected_patterns) => selected_patterns,
        None => return Ok(None),
    };

    // Handle custom pattern if selected
    let maybe_custom_pattern = if selected_patterns.contains(&"Custom pattern...") {
        inquire::Text::new("Enter custom pattern to filter (e.g., 'vec' or 'string'):")
            .prompt_skippable()
    } else {
        Ok(Some(String::new()))
    };

    let custom_pattern = if matches!(maybe_custom_pattern, Err(InquireError::OperationCanceled)) {
        return Ok(None);
    } else if let Some(custom_pattern) = maybe_custom_pattern? {
        custom_pattern
    } else {
        return Ok(None);
    };

    // Create a filtered profile based on memory patterns
    let pattern_filtered = if selected_patterns.is_empty() {
        profile.clone()
    } else {
        // Track filtering statistics
        let mut filter_stats: HashMap<&str, usize> = HashMap::new();
        let total_entries = profile.stacks.len();

        // Create a new filtered profile
        let mut filtered = profile.clone();
        filtered.stacks = profile
            .stacks
            .iter()
            .filter(|stack| {
                selected_patterns.iter().any(|&pattern| {
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
                    !matches
                })
            })
            .cloned()
            .collect();

        // Display filtering statistics
        let heading = "Pattern Filtering Statistics";
        println!("\n{heading}");
        println!("{}", "━".repeat(heading.len()));

        let mut total_filtered = 0;
        for pattern in &selected_patterns {
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

        println!("\nPattern Filtering Summary:");
        println!("Entries remaining: {remaining} ({remaining_percentage:.1}%)");
        println!("Entries filtered:  {total_filtered} ({filtered_percentage:.1}%)");

        if filtered.stacks.is_empty() {
            println!("\nWarning: All entries were filtered out by patterns.");
            println!("Continuing with unfiltered profile for function filtering.");
            profile.clone()
        } else {
            filtered
        }
    };

    // Now add function-based filtering similar to filter_functions

    // Get unique top-level functions from the stacks, not counting `main`.
    let functions: HashSet<String> = pattern_filtered
        .stacks
        .iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // Extract stack trace part (without the size at the end)
                let stack_str = parts[..parts.len() - 1].join(" ");
                // Get the root function from the stack
                stack_str
                    .split(';')
                    // .inspect(|path| eprintln!("path={path}"))
                    .find(|path| !path.ends_with("::main"))
                    .map(ToString::to_string)
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
        .chain(pattern_filtered.stacks.iter().filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // Extract stack trace part (without the size at the end)
                let stack_str = parts[..parts.len() - 1].join(" ");
                // Get the root function from the stack
                stack_str
                    .split(';')
                    // .inspect(|path| eprintln!("path={path}"))
                    .find(|path| path.ends_with("::main"))
                    .map(ToString::to_string)
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        }))
        .collect();

    let mut function_list: Vec<String> = functions.into_iter().collect();
    function_list.sort_unstable();

    if function_list.is_empty() {
        println!("No unique functions found to filter.");
        return Ok(Some(pattern_filtered));
    }

    // Display information about filtering modes
    println!("\nThere are two filtering modes available:");
    println!("  1. Recursive: Removes functions and all their child calls");
    println!(
        "     For example, filtering out 'main' will remove main and everything called by main"
    );
    println!("  2. Exact Match: Removes functions ONLY as standalone entries but preserves them when they have children");
    println!("     For example, filtering out 'allocate_buffer' will remove standalone 'allocate_buffer' entries");
    println!("     but will keep 'allocate_buffer;copy_data' entries\n");

    // Ask user to select filtering mode
    let maybe_filter_mode = inquire::Select::new(
        "Select filtering mode:",
        vec![
            "Recursive (filter out function and ALL its children)",
            "Exact Match (filter out function only when it has NO children)",
        ],
    )
    .prompt_skippable()?;

    let filter_mode = match maybe_filter_mode {
        Some(filter_mode) => filter_mode,
        None => return Ok(None),
    };

    let exact_match = filter_mode.starts_with("Exact Match");

    // Select functions to filter
    println!(
        "\nUsing {} filtering mode",
        if exact_match {
            "Exact Match"
        } else {
            "Recursive"
        }
    );

    let maybe_to_filter =
        MultiSelect::new("Select functions to filter out:", function_list).prompt_skippable()?;

    let to_filter = match maybe_to_filter {
        Some(to_filter) => to_filter,
        None => return Ok(None),
    };

    if to_filter.is_empty() {
        println!("No functions selected for filtering.");
        return Ok(Some(pattern_filtered));
    }

    // Apply appropriate filtering based on the chosen mode
    let filtered_stacks: Vec<String> = if exact_match {
        // In exact match mode, we need to keep any stack where the filtered function
        // has children (i.e., is part of a call chain)
        pattern_filtered
            .stacks
            .iter()
            .filter(|line| {
                let count = line
                    .split_once(" ")
                    .unwrap_or(("", ""))
                    .0
                    .split(';')
                    .skip_while(|s| !to_filter.contains(&s.to_string()))
                    .filter(|s| !to_filter.contains(&s.to_string()))
                    // .inspect(|s| println!("s={s}"))
                    .count();

                count > 0
            })
            .cloned()
            .collect()
    } else {
        // Recursive mode: filter out function and all its children
        pattern_filtered
            .stacks
            .iter()
            .inspect(|stack| eprintln!("Recursive stack: {stack}"))
            .filter(|line| {
                let found = line
                    .split_once(" ")
                    .unwrap_or(("", ""))
                    .0
                    .split(';')
                    .any(|s| !to_filter.contains(&s.to_string()));

                !found
            })
            .cloned()
            .collect()
    };

    // Track function filtering statistics
    let function_filtered_count = pattern_filtered.stacks.len() - filtered_stacks.len();
    let function_filtered_percent = if pattern_filtered.stacks.is_empty() {
        0.0
    } else {
        (function_filtered_count as f64 / pattern_filtered.stacks.len() as f64) * 100.0
    };

    let heading = "Function Filtering Statistics";
    println!("\n{heading}");
    println!("{}", "━".repeat(heading.len()));
    println!(
        "Functions filtered: {} of {} ({:.1}%)",
        function_filtered_count,
        pattern_filtered.stacks.len(),
        function_filtered_percent
    );

    let mut memory_data = MemoryData::default();
    for line in &filtered_stacks {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let delta = parts[parts.len() - 1].parse::<u64>().unwrap_or(0_u64);
            memory_data.bytes_allocated += delta;
        }
    }

    Ok(Some(ProcessedProfile {
        stacks: filtered_stacks,
        memory_data: Some(memory_data),
        ..pattern_filtered // Keep the metadata
    }))
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

fn enhance_svg_accessibility(svg_path: &str) -> ProfileResult<()> {
    let content = fs::read_to_string(svg_path)?;

    // Make the inactive search link more visible
    let enhanced = content.replace(
        "opacity:0.1",
        "opacity:0.5", // Darker grey for better visibility
    );

    fs::write(svg_path, enhanced)?;
    Ok(())
}
