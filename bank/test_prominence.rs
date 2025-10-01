/*[toml]
[dependencies]
# thag_styling = { version = "0.2", thag-auto = true, features = ["inquire_theming"] }
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
inquire = "0.7"
rand = "0.8"
termbg = "0.6"
*/

/// Subjective prominence testing tool for color ranking validation
///
/// This tool displays sets of colors and asks users to rank them by prominence
/// to validate and improve the prominence calculation algorithm.
///
/// ## Usage
///
/// ```
/// # Test with detected background
/// thag test_prominence.rs
///
/// # Test with dark background
/// thag test_prominence.rs -- --dark
///
/// # Test with light background
/// thag test_prominence.rs -- --light
///
/// # Test with both backgrounds (if you have a way to switch mid-execution)
/// thag test_prominence.rs -- --both
/// ```
//# Purpose: Nice app demo
//# Categories: color, crates, demo, technique
use clap::{Arg, Command};
use inquire::Select;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::error::Error;
use thag_styling::{auto_help, Style};

#[derive(Debug, Clone)]
struct ColorTest {
    hex: String,
    name: String,
    calculated_prominence: f32,
}

#[derive(Debug, Clone)]
struct TestSet {
    name: String,
    background_hex: String,
    background_name: String,
    is_light_theme: bool,
    colors: Vec<ColorTest>,
}

fn main() -> Result<(), Box<dyn Error>> {
    auto_help!();

    let matches = Command::new("test_prominence")
        .about("Test subjective prominence vs calculated prominence")
        .arg(
            Arg::new("dark")
                .long("dark")
                .action(clap::ArgAction::SetTrue)
                .help("Test with dark backgrounds only"),
        )
        .arg(
            Arg::new("light")
                .long("light")
                .action(clap::ArgAction::SetTrue)
                .help("Test with light backgrounds only"),
        )
        .arg(
            Arg::new("auto")
                .long("auto")
                .action(clap::ArgAction::SetTrue)
                .help("Auto-detect background and run appropriate test"),
        )
        .arg(
            Arg::new("colors")
                .long("colors")
                .value_name("N")
                .default_value("5")
                .help("Number of colors to test per set (3-10)"),
        )
        .arg(
            Arg::new("sets")
                .long("sets")
                .value_name("N")
                .default_value("3")
                .help("Number of test sets to run"),
        )
        .get_matches();

    let test_dark = matches.get_flag("dark");
    let test_light = matches.get_flag("light");
    let test_auto = matches.get_flag("auto");
    let colors_per_set: usize = matches
        .get_one::<String>("colors")
        .unwrap()
        .parse()
        .unwrap_or(5)
        .clamp(3, 10);
    let num_sets: usize = matches
        .get_one::<String>("sets")
        .unwrap()
        .parse()
        .unwrap_or(3)
        .clamp(1, 10);

    if test_auto || (!test_dark && !test_light) {
        // Auto-detect or default behavior
        let detected_dark = detect_terminal_background();
        println!(
            "🔍 Auto-detected terminal background: {}",
            if detected_dark { "Dark" } else { "Light" }
        );
        run_prominence_tests(detected_dark, !detected_dark, colors_per_set, num_sets)?;
    } else {
        run_prominence_tests(test_dark, test_light, colors_per_set, num_sets)?;
    }

    Ok(())
}

fn run_prominence_tests(
    test_dark: bool,
    test_light: bool,
    colors_per_set: usize,
    num_sets: usize,
) -> Result<(), Box<dyn Error>> {
    println!("🎨 Color Prominence Testing Tool");
    println!("═══════════════════════════════════════════════════════════════");
    println!("This tool will show you sets of colors and ask you to rank them");
    println!("by subjective prominence (most eye-catching = 1, least = last).");
    println!("Your responses help validate the prominence calculation algorithm.");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut all_results = Vec::new();

    if test_dark {
        println!("🌙 Testing with DARK backgrounds...\n");
        for i in 1..=num_sets {
            let test_set = generate_dark_test_set(i, colors_per_set)?;
            let result = run_single_test(&test_set)?;
            all_results.push(result);
            println!();
        }
    }

    if test_light {
        println!("☀️ Testing with LIGHT backgrounds...\n");
        for i in 1..=num_sets {
            let test_set = generate_light_test_set(i, colors_per_set)?;
            let result = run_single_test(&test_set)?;
            all_results.push(result);
            println!();
        }
    }

    analyze_results(&all_results)?;

    Ok(())
}

fn generate_dark_test_set(
    set_num: usize,
    colors_per_set: usize,
) -> Result<TestSet, Box<dyn Error>> {
    let backgrounds = vec![
        ("#1e1e2e", "Catppuccin Mocha"),
        ("#131513", "Atelier Seaside Dark"),
        ("#282a36", "Dracula"),
        ("#2E3440", "Nord"),
        ("#000000", "True Black"),
    ];

    let bg = backgrounds[set_num % backgrounds.len()];

    // Diverse color palette for testing
    let mut color_pool = vec![
        ("#ff79c6", "Bright Magenta"),
        ("#8be9fd", "Cyan"),
        ("#bd93f9", "Purple"),
        ("#f1fa8c", "Yellow"),
        ("#50fa7b", "Green"),
        ("#ff5555", "Red"),
        ("#ffb86c", "Orange"),
        ("#80bfff", "Light Blue"),
        ("#ff6ac1", "Hot Pink"),
        ("#9aedfe", "Light Cyan"),
        ("#c9ff61", "Lime"),
        ("#ff9580", "Salmon"),
        ("#b4a7d1", "Lavender"),
        ("#ffd93d", "Gold"),
        ("#6bcf7f", "Mint Green"),
    ];

    color_pool.shuffle(&mut thread_rng());
    let selected_colors: Vec<_> = color_pool.into_iter().take(colors_per_set).collect();

    let mut colors = Vec::new();
    for (hex, name) in selected_colors {
        let prominence = calculate_prominence(hex, true)?;
        colors.push(ColorTest {
            hex: hex.to_string(),
            name: name.to_string(),
            calculated_prominence: prominence,
        });
    }

    Ok(TestSet {
        name: format!("Dark Set {}", set_num),
        background_hex: bg.0.to_string(),
        background_name: bg.1.to_string(),
        is_light_theme: false,
        colors,
    })
}

fn generate_light_test_set(
    set_num: usize,
    colors_per_set: usize,
) -> Result<TestSet, Box<dyn Error>> {
    let backgrounds = vec![
        ("#f4fbf4", "Atelier Seaside Light"),
        ("#faf4ed", "Rosé Pine Dawn"),
        ("#fbf1c7", "Gruvbox Light"),
        ("#ffffff", "True White"),
        ("#fdf6e3", "Solarized Light"),
    ];

    let bg = backgrounds[set_num % backgrounds.len()];

    // Darker colors for light backgrounds
    let mut color_pool = vec![
        ("#ad2bee", "Dark Purple"),
        ("#1999b3", "Dark Teal"),
        ("#e619c3", "Dark Pink"),
        ("#98981b", "Dark Yellow"),
        ("#29a329", "Dark Green"),
        ("#e6193c", "Dark Red"),
        ("#87711d", "Dark Orange"),
        ("#3d62f5", "Dark Blue"),
        ("#b02a37", "Crimson"),
        ("#0d7377", "Dark Cyan"),
        ("#6b5b95", "Royal Purple"),
        ("#d4a574", "Dark Gold"),
        ("#2e8b57", "Sea Green"),
        ("#8b4513", "Saddle Brown"),
        ("#4b0082", "Indigo"),
    ];

    color_pool.shuffle(&mut thread_rng());
    let selected_colors: Vec<_> = color_pool.into_iter().take(colors_per_set).collect();

    let mut colors = Vec::new();
    for (hex, name) in selected_colors {
        let prominence = calculate_prominence(hex, false)?;
        colors.push(ColorTest {
            hex: hex.to_string(),
            name: name.to_string(),
            calculated_prominence: prominence,
        });
    }

    Ok(TestSet {
        name: format!("Light Set {}", set_num),
        background_hex: bg.0.to_string(),
        background_name: bg.1.to_string(),
        is_light_theme: true,
        colors,
    })
}

fn run_single_test(test_set: &TestSet) -> Result<TestResult, Box<dyn Error>> {
    println!("📋 Test: {}", test_set.name);
    println!(
        "🎨 Background: {} ({})",
        test_set.background_name, test_set.background_hex
    );
    println!("───────────────────────────────────────────────────────────────");

    // Display colors
    println!("Colors to rank:");
    for (i, color) in test_set.colors.iter().enumerate() {
        let style = Style::from_fg_hex(&color.hex)?;

        println!(
            "  {}. {} {} ({}) [calc prominence: {:.3}]",
            i + 1,
            style.paint("██████"),
            style.paint(&color.name),
            color.hex,
            color.calculated_prominence
        );
    }

    println!("\n🤔 Please rank these colors by SUBJECTIVE PROMINENCE");
    println!(
        "   (1 = most eye-catching/prominent, {} = least prominent)",
        test_set.colors.len()
    );

    let mut subjective_ranking = Vec::new();
    let mut available_colors = test_set.colors.clone();

    for rank in 1..=test_set.colors.len() {
        let options: Vec<String> = available_colors
            .iter()
            .map(|c| format!("{} ({})", c.name, c.hex))
            .collect();

        let prompt = if rank == 1 {
            "🥇 Which color is MOST prominent/eye-catching?".to_string()
        } else if rank == test_set.colors.len() {
            "🥉 Which color is LEAST prominent/eye-catching?".to_string()
        } else {
            format!("#{} Which color ranks #{} in prominence?", rank, rank)
        };

        let selection = Select::new(&prompt, options).prompt()?;

        // Find and remove the selected color
        let selected_index = available_colors
            .iter()
            .position(|c| format!("{} ({})", c.name, c.hex) == selection)
            .ok_or("Color not found")?;

        let selected_color = available_colors.remove(selected_index);
        subjective_ranking.push((rank, selected_color));
    }

    let result = TestResult {
        test_set: test_set.clone(),
        subjective_ranking,
    };

    display_comparison(&result);

    Ok(result)
}

fn display_comparison(result: &TestResult) {
    println!("\n📊 Results Comparison:");
    println!("┌─────┬─────────────────────┬──────────┬─────────────────────┬──────────┐");
    println!("│Subj │ Color               │ Calc     │ Algorithmic Order   │ Calc     │");
    println!("│Rank │                     │ Prom     │                     │ Prom     │");
    println!("├─────┼─────────────────────┼──────────┼─────────────────────┼──────────┤");

    // Sort by calculated prominence for right side
    let mut calc_sorted = result.test_set.colors.clone();
    calc_sorted.sort_by(|a, b| {
        b.calculated_prominence
            .partial_cmp(&a.calculated_prominence)
            .unwrap()
    });

    for i in 0..result.subjective_ranking.len() {
        let (subj_rank, subj_color) = &result.subjective_ranking[i];
        let calc_color = &calc_sorted[i];

        println!(
            "│  {}  │ {:19} │  {:.3}   │ {:19} │  {:.3}   │",
            subj_rank,
            format!("{} {}", subj_color.name, subj_color.hex),
            subj_color.calculated_prominence,
            format!("{} {}", calc_color.name, calc_color.hex),
            calc_color.calculated_prominence
        );
    }
    println!("└─────┴─────────────────────┴──────────┴─────────────────────┴──────────┘");

    // Calculate agreement
    let agreement = calculate_ranking_agreement(&result);
    println!("🎯 Ranking Agreement: {:.1}%", agreement * 100.0);
}

#[derive(Debug, Clone)]
struct TestResult {
    test_set: TestSet,
    subjective_ranking: Vec<(usize, ColorTest)>, // (rank, color)
}

fn calculate_ranking_agreement(result: &TestResult) -> f32 {
    // Sort by calculated prominence
    let mut calc_sorted = result.test_set.colors.clone();
    calc_sorted.sort_by(|a, b| {
        b.calculated_prominence
            .partial_cmp(&a.calculated_prominence)
            .unwrap()
    });

    // Create ranking maps
    let subjective_map: HashMap<String, usize> = result
        .subjective_ranking
        .iter()
        .map(|(rank, color)| (color.hex.clone(), *rank))
        .collect();

    let calculated_map: HashMap<String, usize> = calc_sorted
        .iter()
        .enumerate()
        .map(|(i, color)| (color.hex.clone(), i + 1))
        .collect();

    // Calculate Spearman rank correlation coefficient
    let n = result.test_set.colors.len() as f32;
    let mut d_squared_sum = 0.0;

    for color in &result.test_set.colors {
        let subj_rank = subjective_map[&color.hex] as f32;
        let calc_rank = calculated_map[&color.hex] as f32;
        let d = subj_rank - calc_rank;
        d_squared_sum += d * d;
    }

    let correlation = 1.0 - (6.0 * d_squared_sum) / (n * (n * n - 1.0));

    // Convert correlation (-1 to 1) to agreement percentage (0 to 1)
    (correlation + 1.0) / 2.0
}

fn analyze_results(results: &[TestResult]) -> Result<(), Box<dyn Error>> {
    println!("🔬 ANALYSIS SUMMARY");
    println!("═══════════════════════════════════════════════════════════════");

    let avg_agreement: f32 =
        results.iter().map(calculate_ranking_agreement).sum::<f32>() / results.len() as f32;

    println!(
        "📈 Overall Algorithm Agreement: {:.1}%",
        avg_agreement * 100.0
    );

    // Analyze by theme type
    let dark_results: Vec<_> = results
        .iter()
        .filter(|r| !r.test_set.is_light_theme)
        .collect();
    let light_results: Vec<_> = results
        .iter()
        .filter(|r| r.test_set.is_light_theme)
        .collect();

    if !dark_results.is_empty() {
        let dark_avg: f32 = dark_results
            .iter()
            .map(|r| calculate_ranking_agreement(r))
            .sum::<f32>()
            / dark_results.len() as f32;
        println!("🌙 Dark Theme Agreement: {:.1}%", dark_avg * 100.0);
    }

    if !light_results.is_empty() {
        let light_avg: f32 = light_results
            .iter()
            .map(|r| calculate_ranking_agreement(r))
            .sum::<f32>()
            / light_results.len() as f32;
        println!("☀️ Light Theme Agreement: {:.1}%", light_avg * 100.0);
    }

    // Identify problematic colors
    analyze_color_patterns(results);

    println!("\n💡 RECOMMENDATIONS:");
    if avg_agreement < 0.7 {
        println!("• Algorithm needs significant improvement (< 70% agreement)");
        println!("• Consider adjusting saturation/lightness weights");
        println!("• May need hue-specific prominence factors");
    } else if avg_agreement < 0.85 {
        println!("• Algorithm is reasonable but can be improved");
        println!("• Focus on specific color/background combinations that disagree most");
    } else {
        println!("• Algorithm performs well! (> 85% agreement)");
        println!("• Minor tweaks may still improve edge cases");
    }

    Ok(())
}

fn analyze_color_patterns(results: &[TestResult]) {
    println!("\n🎨 COLOR PATTERN ANALYSIS:");

    let mut color_disagreements: HashMap<String, Vec<f32>> = HashMap::new();

    for result in results {
        // For each color, see how far off the algorithm was
        for (subj_rank, color) in &result.subjective_ranking {
            let mut calc_sorted = result.test_set.colors.clone();
            calc_sorted.sort_by(|a, b| {
                b.calculated_prominence
                    .partial_cmp(&a.calculated_prominence)
                    .unwrap()
            });

            let calc_rank = calc_sorted
                .iter()
                .position(|c| c.hex == color.hex)
                .map(|i| i + 1)
                .unwrap_or(1) as f32;

            let disagreement = (*subj_rank as f32 - calc_rank).abs();

            // Group by color family/hue
            let color_family = categorize_color(&color.hex);
            color_disagreements
                .entry(color_family)
                .or_insert_with(Vec::new)
                .push(disagreement);
        }
    }

    println!("Average disagreement by color family:");
    let mut families: Vec<_> = color_disagreements.iter().collect();
    families.sort_by(|a, b| {
        let avg_a = a.1.iter().sum::<f32>() / a.1.len() as f32;
        let avg_b = b.1.iter().sum::<f32>() / b.1.len() as f32;
        avg_b.partial_cmp(&avg_a).unwrap()
    });

    for (family, disagreements) in families {
        let avg = disagreements.iter().sum::<f32>() / disagreements.len() as f32;
        let samples = disagreements.len();
        println!("• {:12}: {:.2} ranks off (n={})", family, avg, samples);
    }
}

fn categorize_color(hex: &str) -> String {
    // Simple hue-based categorization
    let (r, g, b) = hex_to_rgb(hex).unwrap_or((0, 0, 0));
    let (h, s, _) = rgb_to_hsl([r, g, b]);

    if s < 0.2 {
        return "Gray".to_string();
    }

    match h {
        h if h < 15.0 || h >= 345.0 => "Red",
        h if h < 45.0 => "Orange",
        h if h < 75.0 => "Yellow",
        h if h < 105.0 => "Yellow-Green",
        h if h < 135.0 => "Green",
        h if h < 165.0 => "Green-Cyan",
        h if h < 195.0 => "Cyan",
        h if h < 225.0 => "Blue-Cyan",
        h if h < 255.0 => "Blue",
        h if h < 285.0 => "Blue-Magenta",
        h if h < 315.0 => "Magenta",
        _ => "Red-Magenta",
    }
    .to_string()
}

// Utility functions
fn calculate_prominence(hex: &str, is_light_theme: bool) -> Result<f32, Box<dyn Error>> {
    const SATURATION_WEIGHT: f32 = 0.6;
    const LIGHTNESS_WEIGHT: f32 = 0.4;

    let (r, g, b) = hex_to_rgb(hex)?;
    let (_h, s, l) = rgb_to_hsl([r, g, b]);
    let saturation_score = s;

    let lightness_score = if is_light_theme {
        1.0 - l // Darker = higher score
    } else {
        l // Lighter = higher score
    };

    Ok(SATURATION_WEIGHT * saturation_score + LIGHTNESS_WEIGHT * lightness_score)
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), Box<dyn Error>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Invalid hex color".into());
    }

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok((r, g, b))
}

fn rgb_to_hsl(rgb: [u8; 3]) -> (f32, f32, f32) {
    let r = f32::from(rgb[0]) / 255.0;
    let g = f32::from(rgb[1]) / 255.0;
    let b = f32::from(rgb[2]) / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let lightness = (max + min) / 2.0;

    let saturation = if delta == 0.0 {
        0.0
    } else if lightness < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let hue = if delta == 0.0 {
        0.0
    } else if (max - r).abs() < f32::EPSILON {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < f32::EPSILON {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    (hue, saturation, lightness)
}

fn detect_terminal_background() -> bool {
    // Try to detect if we're on a dark background
    // This is a simple heuristic - assume dark background unless we can detect otherwise

    // Check if we can get terminal background info
    if let Ok(termbg::Theme::Dark) = termbg::theme(std::time::Duration::from_millis(100)) {
        true
    } else if let Ok(termbg::Theme::Light) = termbg::theme(std::time::Duration::from_millis(100)) {
        false
    } else {
        // Default to dark if we can't detect
        true
    }
}
