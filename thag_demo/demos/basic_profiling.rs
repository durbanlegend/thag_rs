/*[toml]
[dependencies]
thag_demo_proc_macros = { version = "0.1, thag-auto" }
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }
inferno = "0.11"
chrono = { version = "0.4", features = ["serde"] }

[profile.release]
debug = true
strip = false
*/

/// Basic profiling demo - shows how to use thag_profiler for function timing
/// This demo demonstrates the core profiling features of thag_profiler
//# Purpose: Demonstrate basic time profiling with thag_profiler
//# Categories: profiling, demo, timing
use chrono::Local;
use ibig::{ubig, UBig};
use inferno::flamegraph::{self, color::MultiPalette, Options, Palette};
use num_traits::identities::One;
use std::io::Write;
use std::iter::successors;
use std::thread;
use std::time::Duration;
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::{cached, timing};
use thag_profiler::{enable_profiling, enhance_svg_accessibility, profiled};

const FIB_N: usize = 45;
const HUNDREDFOLD: usize = FIB_N * 100;
// const MULTIPLIER: usize = 200;
// const MULTIPLIED: usize = FIB_N * MULTIPLIER;

#[profiled]
#[timing]
fn fibonacci_recursions(n: usize) {
    let result = fibonacci(n);
    println!("fibonacci({n}) = {result}");
}

// For recursive functions, only time-profile the caller, to avoid
// unfixable multiple counting of elapsed time.
fn fibonacci(n: usize) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[profiled]
#[timing]
fn fibonacci_recursions_cached(n: usize) {
    let result = fibonacci_cached(UBig::from(n));
    println!("fibonacci({n}) (cached) = {result}");
}

// For recursive functions, only time-profile the caller, to avoid
// unfixable multiple counting of elapsed time.
#[cached]
fn fibonacci_cached(n: UBig) -> UBig {
    if n <= UBig::one() {
        n
    } else {
        fibonacci_cached(n.clone() - 1) + fibonacci_cached(n - 2)
    }
}

#[profiled]
#[timing]
fn fibonacci_iter(n: usize) {
    let result = successors(Some((ubig!(0), ubig!(1))), |(a, b)| {
        Some((b.clone(), (a + b).into()))
    })
    .map(|(a, _b)| a)
    .nth(n)
    .unwrap();
    println!("fibonacci({n}) (iter) = {result}");
}

#[profiled]
#[timing]
fn cpu_intensive_work() {
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum += i * i;
    }
    println!("CPU work result: {}", sum);
}

#[profiled]
#[timing]
fn simulated_io_work() {
    println!("Starting simulated I/O work...");
    thread::sleep(Duration::from_millis(100));
    println!("I/O work completed");
}

#[profiled]
fn nested_function_calls() {
    cpu_intensive_work();
    simulated_io_work();

    // Calculate some fibonacci numbers
    println!("\nHey, it's-a me, Fibonacci!\n");
    println!(
        "Let's calculate my {FIB_N}th Fibonacci number recursively, because {FIB_N} makes for a chunky computation, but not insanely so."
    );
    println!("Elapsed time for recursion increases exponentially with the Fibonacci number, so we don't want to overdo it.\n");

    // First recursively - bad idea as O(2^n)
    fibonacci_recursions(FIB_N);

    println!("\nOof, bad idea. And it will quickly get a lot worse for bigger numbers.");
    // let _ = std::io::stdout().flush();
    pause_awhile();
}

// Pause to display output and help drill down to the tiny flamegraph bars for fast functions
#[profiled]
fn pause_awhile() {
    let _ = std::io::stdout().flush();
    thread::sleep(Duration::from_secs(2));
}

#[profiled]
fn alt_fibonacci_cached() {
    println!("\nHow about we use thag's demo #[cached] attribute on the fibonacci function?\n");

    pause_awhile();

    // Then with cached functions
    fibonacci_recursions_cached(FIB_N);

    println!("\nThat's insane!");
    println!(
        "\nA little bird told me I can go up two orders of magnitude and calculate my {}th number and still come out way ahead!\n",
        HUNDREDFOLD
    );

    pause_awhile();

    // Then with cached functions
    fibonacci_recursions_cached(HUNDREDFOLD);

    println!("\nHoly smokes! What a difference! Recursion is not always your friend, but #[cached] is your friend - at least up until the stack overflows from too much recursion.");
}

#[profiled]
fn alt_fibonacci_iter() {
    println!("\nWhat if we try Rust iterators instead, still for F({HUNDREDFOLD})?\n");

    pause_awhile();

    // Non-nested with Rust iterator. Even then, will it show up in profiling?
    fibonacci_iter(HUNDREDFOLD);

    println!(
        "\n🤯 Not too shabby! But we can go a lot bigger and faster still with no overflows - you can go down the fibonacci rabbit hole in the demo collection of the `thag_rs` crate. Ciao!\n"
    );
}

#[enable_profiling(time)]
fn demo() {
    println!("🔥 Basic Profiling Demo");
    println!("════════════════════════");
    println!();

    println!("Running nested function calls with profiling...");
    nested_function_calls();

    // Separate function to help in drilling down
    alt_fibonacci_cached();

    // Separate function to help in drilling down
    alt_fibonacci_iter();

    pause_awhile();

    println!("✅ Demo completed!");
    println!("📊 Check the generated flamechart files for visual analysis.");
    println!("🔍 Use 'thag_profile' command to analyze the profiling data.");
}

fn main() {
    // Ensure no stack overflow at hundredfold on all platforms
    let child = thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            demo();
        })
        .unwrap();

    let _ = child.join().unwrap();

    // Add interactive visualization
    show_interactive_visualization();
}

fn show_interactive_visualization() {
    println!();
    println!("🎯 Would you like to view an interactive flamechart?");
    println!("This will generate a visual flamechart and open it in your browser.");
    print!("Enter 'y' for yes, or any other key to skip: ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        if input.trim().to_lowercase() == "y" {
            println!();
            println!("🔥 Generating interactive flamechart...");

            // Try to load and display the profile data
            match load_and_show_profile() {
                Ok(()) => {
                    println!("✅ Flamechart generation completed!");
                }
                Err(e) => {
                    println!("⚠️  Could not generate flamechart: {}", e);
                    println!(
                        "💡 Make sure the demo completed successfully and generated profile files."
                    );
                }
            }
        }
    }
}

fn load_and_show_profile() -> Result<(), Box<dyn std::error::Error>> {
    // Wait a moment for profile files to be written
    std::thread::sleep(std::time::Duration::from_millis(500));

    let current_dir = std::env::current_dir()?;
    let mut exclusive_files = Vec::new();
    let mut inclusive_files = Vec::new();

    for entry in std::fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("thag_demo_basic_profiling") && name.ends_with(".folded") {
                if name.contains("inclusive") {
                    inclusive_files.push(path);
                } else {
                    exclusive_files.push(path);
                }
            }
        }
    }

    if exclusive_files.is_empty() && inclusive_files.is_empty() {
        return Err("No profile files found".into());
    }

    // Sort by modification time, most recent first
    exclusive_files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    inclusive_files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    // Use exclusive file for both text analysis and flamechart generation
    if !exclusive_files.is_empty() {
        let exclusive_file = &exclusive_files[0];
        show_simple_profile_analysis(exclusive_file)?;
        generate_flamechart(exclusive_file)?;
    }

    Ok(())
}

fn show_simple_profile_analysis(
    file_path: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut function_times: std::collections::HashMap<String, u128> =
        std::collections::HashMap::new();
    let mut total_duration_us = 0u128;

    // Parse folded stack format
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let stack = parts[0];
        let time_str = parts[1];

        if let Ok(time_us) = time_str.parse::<u128>() {
            total_duration_us += time_us;

            // Extract function names from the stack
            let functions: Vec<&str> = stack.split(';').collect();
            for func_name in functions {
                let clean_name = clean_function_name(func_name);
                *function_times.entry(clean_name).or_insert(0) += time_us;
            }
        }
    }

    // Create and display analysis
    let mut functions: Vec<_> = function_times.into_iter().collect();
    functions.sort_by(|a, b| b.1.cmp(&a.1));

    println!("📊 Profile Analysis Results");
    println!("═══════════════════════════");
    println!("Total Duration: {:.3}ms", total_duration_us as f64 / 1000.0);
    println!();

    println!("🏆 Top Functions by Execution Time:");
    println!("────────────────────────────────────");

    for (i, (name, time_us)) in functions.iter().enumerate().take(10) {
        let percentage = (*time_us as f64 / total_duration_us as f64) * 100.0;
        let time_ms = *time_us as f64 / 1000.0;

        let icon = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "🏅",
        };

        println!(
            "{} {}. {} - {:.3}ms ({:.1}%)",
            icon,
            i + 1,
            name,
            time_ms,
            percentage
        );
    }

    println!();
    show_performance_insights(&functions, total_duration_us);

    Ok(())
}

fn clean_function_name(name: &str) -> String {
    let clean = name.split("::").last().unwrap_or(name);
    let clean = clean.split('<').next().unwrap_or(clean);
    let clean = clean.split('(').next().unwrap_or(clean);

    match clean {
        s if s.starts_with("thag_demo_") => s.strip_prefix("thag_demo_").unwrap_or(s).to_string(),
        s if s.contains("fibonacci") => {
            if s.contains("cached") {
                "fibonacci_cached"
            } else if s.contains("iter") {
                "fibonacci_iter"
            } else {
                "fibonacci_recursive"
            }
        }
        .to_string(),
        s if s.contains("cpu_intensive") => "cpu_intensive_work".to_string(),
        s if s.contains("simulated_io") => "simulated_io_work".to_string(),
        s if s.contains("nested_function") => "nested_function_calls".to_string(),
        s => s.to_string(),
    }
}

fn show_performance_insights(functions: &[(String, u128)], _total_duration_us: u128) {
    println!("💡 Performance Insights:");
    println!("────────────────────────");

    if functions.len() >= 2 {
        let slowest = &functions[0];
        let fastest = &functions[functions.len() - 1];

        if fastest.1 > 0 {
            let speedup = slowest.1 as f64 / fastest.1 as f64;
            println!(
                "🐌 Slowest: {} ({:.3}ms)",
                slowest.0,
                slowest.1 as f64 / 1000.0
            );
            println!(
                "🚀 Fastest: {} ({:.3}ms)",
                fastest.0,
                fastest.1 as f64 / 1000.0
            );
            println!("⚡ Performance difference: {:.1}x", speedup);

            if speedup > 1000.0 {
                println!("🎯 Consider using faster algorithms in production!");
            }
        }
    }

    // Look for specific patterns
    let has_recursive = functions.iter().any(|(name, _)| name.contains("recursive"));
    let has_cached = functions.iter().any(|(name, _)| name.contains("cached"));
    let has_iter = functions.iter().any(|(name, _)| name.contains("iter"));

    if has_recursive && has_cached {
        println!("🔧 Tip: Caching can dramatically improve recursive algorithms!");
    }

    if has_iter {
        println!("🔄 Tip: Iterative approaches often outperform recursion for large inputs!");
    }

    println!();
}

fn generate_flamechart(
    profile_file: &std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 Generating Interactive Flamechart...");

    // println!("profile_file={profile_file:#?}");
    let content = std::fs::read_to_string(profile_file)?;

    println!("content={content}");
    let stacks: Vec<String> = content.lines().map(|line| line.to_string()).collect();

    if stacks.is_empty() {
        println!("⚠️  No profile data found in file");
        return Ok(());
    }

    // Create flamechart options
    let mut opts = Options::default();
    opts.title = "Basic Profiling Demo - Performance Flamechart".to_string();
    opts.subtitle = Some(format!(
        "Generated: {} | Hover over and click on the bars to explore the function call hierarchy, or use Search ↗️",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    opts.colors = Palette::Multi(MultiPalette::Rust);
    opts.count_name = "μs".to_string();
    opts.min_width = 0.0;
    opts.flame_chart = true; // Use aggregated flamechart

    // Generate flamechart
    let svg_path = "basic_profiling_flamechart.svg";
    let output = std::fs::File::create(svg_path)?;

    flamegraph::from_lines(&mut opts, stacks.iter().rev().map(String::as_str), output)?;

    enhance_svg_accessibility(svg_path)?;

    println!("✅ Flamechart generated: {}", svg_path);

    // Open in browser
    if let Err(e) = open_in_browser(svg_path) {
        println!("⚠️  Could not open browser automatically: {}", e);
        println!("💡 You can manually open: {}", svg_path);
    } else {
        println!("🌐 Flame opened in your default browser!");
        println!("🔍 Hover over and click on the bars to explore the performance visualization, or use Search");
        println!("📊 Function width = time spent, height = call stack depth");
        println!("💡 Notice how the recursive fibonacci dominates the graph!");
    }

    Ok(())
}

fn open_in_browser(svg_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let full_path = std::env::current_dir()?.join(svg_path);
    let url = format!("file://{}", full_path.display());

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &url])
            .spawn()?;
    }

    Ok(())
}
