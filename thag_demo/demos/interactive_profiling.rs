/*[toml]
[dependencies]
thag_demo_proc_macros = { version = "0.1, thag-auto" }
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Interactive profiling demo with embedded visualization
/// This demo shows how to use thag_profiler and immediately analyze results
//# Purpose: Demonstrate interactive profiling analysis with embedded visualization
//# Categories: profiling, demo, timing, interactive, visualization
use ibig::{ubig, UBig};
use num_traits::identities::One;
use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::successors;
use std::thread;
use std::time::Duration;
use thag_demo_proc_macros::{cached, timing};
use thag_profiler::{enable_profiling, profiled};

const FIB_N: usize = 30; // Smaller for better demo experience
const LARGE_N: usize = FIB_N * 20;

#[profiled]
#[timing]
fn fibonacci_recursive_demo(n: usize) -> u64 {
    println!("Computing fibonacci({}) recursively...", n);
    let result = fibonacci_recursive(n);
    println!("fibonacci({}) = {}", n, result);
    result
}

// Recursive implementation - intentionally inefficient for demo purposes
fn fibonacci_recursive(n: usize) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2)
    }
}

#[profiled]
#[timing]
fn fibonacci_cached_demo(n: usize) -> UBig {
    println!("Computing fibonacci({}) with caching...", n);
    let result = fibonacci_cached(UBig::from(n));
    println!("fibonacci({}) (cached) = {}", n, result);
    result
}

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
fn fibonacci_iterative_demo(n: usize) -> UBig {
    println!("Computing fibonacci({}) iteratively...", n);
    let result = successors(Some((ubig!(0), ubig!(1))), |(a, b)| {
        Some((b.clone(), (a + b).into()))
    })
    .map(|(a, _b)| a)
    .nth(n)
    .unwrap();
    println!("fibonacci({}) (iterative) = {}", n, result);
    result
}

#[profiled]
#[timing]
fn cpu_intensive_work() {
    println!("Running CPU-intensive work...");
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum += i * i;
    }
    println!("CPU work completed: sum = {}", sum);
}

#[profiled]
#[timing]
fn simulated_io_work() {
    println!("Simulating I/O work...");
    thread::sleep(Duration::from_millis(100));
    println!("I/O work completed");
}

#[profiled]
fn algorithm_comparison() {
    println!("\n🔬 Algorithm Performance Comparison");
    println!("=====================================");

    println!("\n1. Recursive Fibonacci (O(2^n) - exponential time):");
    fibonacci_recursive_demo(FIB_N);

    pause_between_demos();

    println!("\n2. Cached Fibonacci (O(n) with memoization):");
    fibonacci_cached_demo(LARGE_N);

    pause_between_demos();

    println!("\n3. Iterative Fibonacci (O(n) - linear time):");
    fibonacci_iterative_demo(LARGE_N);

    pause_between_demos();
}

#[profiled]
fn performance_workloads() {
    println!("\n⚡ Different Performance Workloads");
    println!("===================================");

    cpu_intensive_work();
    pause_between_demos();

    simulated_io_work();
    pause_between_demos();
}

fn pause_between_demos() {
    thread::sleep(Duration::from_millis(500));
}

fn show_simple_analysis() {
    println!("\n🎯 Profile Analysis");
    println!("===================");

    // Wait for profile files to be written
    thread::sleep(Duration::from_millis(1000));

    match find_and_analyze_profile() {
        Ok(()) => {
            println!("\n✅ Analysis completed!");
        }
        Err(e) => {
            println!("\n⚠️  Could not analyze profile: {}", e);
            println!("💡 This is normal in some environments - the demo still worked!");
        }
    }

    println!("\n🔍 Want to explore more?");
    println!("Try adjusting the fibonacci numbers or adding your own functions!");
}

fn find_and_analyze_profile() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let mut profile_files = Vec::new();

    for entry in std::fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("thag_demo_interactive_profiling") && name.ends_with(".folded") {
                profile_files.push(path);
            }
        }
    }

    if profile_files.is_empty() {
        return Err("No profile files found".into());
    }

    // Get the most recent file
    profile_files.sort_by(|a, b| {
        let time_a = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let time_b = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    let profile_file = &profile_files[0];
    analyze_profile_file(profile_file)?;

    Ok(())
}

fn analyze_profile_file(file_path: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut function_times: HashMap<String, u128> = HashMap::new();
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

    // Display results
    let mut functions: Vec<_> = function_times.into_iter().collect();
    functions.sort_by(|a, b| b.1.cmp(&a.1));

    println!("📊 Profile Results:");
    println!("Total Duration: {:.3}ms", total_duration_us as f64 / 1000.0);
    println!();

    println!("🏆 Top Functions by Time:");
    for (i, (name, time_us)) in functions.iter().enumerate().take(8) {
        let percentage = (*time_us as f64 / total_duration_us as f64) * 100.0;
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
            *time_us as f64 / 1000.0,
            percentage
        );
    }

    println!();
    show_insights(&functions, total_duration_us);

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
                "fibonacci_iterative"
            } else {
                "fibonacci_recursive"
            }
        }
        .to_string(),
        s if s.contains("cpu_intensive") => "cpu_intensive_work".to_string(),
        s if s.contains("simulated_io") => "simulated_io_work".to_string(),
        s if s.contains("algorithm_comparison") => "algorithm_comparison".to_string(),
        s if s.contains("performance_workloads") => "performance_workloads".to_string(),
        s => s.to_string(),
    }
}

fn show_insights(functions: &[(String, u128)], total_duration_us: u128) {
    println!("💡 Performance Insights:");
    println!("─────────────────────");

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
            println!("⚡ Speedup: {:.1}x faster", speedup);

            if speedup > 100.0 {
                println!("🎯 Huge performance difference! Algorithm choice matters!");
            }
        }
    }

    // Look for patterns
    let has_recursive = functions.iter().any(|(name, _)| name.contains("recursive"));
    let has_cached = functions.iter().any(|(name, _)| name.contains("cached"));
    let has_iterative = functions.iter().any(|(name, _)| name.contains("iterative"));

    if has_recursive && has_cached {
        println!("🔧 Tip: Caching transforms exponential algorithms into linear ones!");
    }

    if has_iterative {
        println!("🔄 Tip: Iterative approaches avoid recursion overhead!");
    }

    println!();
    println!("🎓 Key Takeaways:");
    println!("• Algorithm complexity matters more than micro-optimizations");
    println!("• Caching can provide dramatic speedups for recursive algorithms");
    println!("• Profiling helps identify the real bottlenecks");
}

#[enable_profiling(time)]
fn main() {
    println!("🔥 Interactive Profiling Demo");
    println!("=============================");
    println!("This demo shows algorithm performance differences and analyzes the results!");
    println!();

    // Run the profiled workloads
    algorithm_comparison();
    performance_workloads();

    println!("\n✅ Profiling completed!");

    // Show simple analysis
    show_simple_analysis();

    println!("\n🎯 Demo Summary:");
    println!("• Compared different algorithm complexities");
    println!("• Analyzed performance data automatically");
    println!("• Learned about optimization opportunities");
    println!("\n💡 Next steps: Try the 'flamegraph' demo for visual analysis!");
}
