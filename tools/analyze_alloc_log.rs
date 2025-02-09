/*[toml]
[dependencies]
chrono = "0.4.39"
clap = { version = "4.4.18", features = ["derive"] }
*/

use chrono::{DateTime, Local, TimeZone};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "analyze_alloc_log")]
#[command(about = "Analyze thag allocation logs")]
struct Args {
    /// Path to the allocation log file
    #[arg(default_value = "thag-profile.alloc.log")]
    log_file: PathBuf,
}

#[derive(Debug)]
struct LogEntry {
    timestamp: u128,
    operation: char,
    size: usize,
    current: usize,
    peak: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = File::open(&args.log_file)?;
    let mut reader = BufReader::new(file);

    // Read and parse header
    let mut header = String::new();
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        if line.trim().is_empty() {
            break;
        }
        header.push_str(&line);
        line.clear();
    }

    println!("Log File Analysis");
    println!("================\n");
    println!("Header:\n{}", header);

    // Process entries
    let mut entries = Vec::new();
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        if let Some(entry) = parse_entry(&line) {
            entries.push(entry);
        }
        line.clear();
    }

    // Generate summary
    let total_allocations = entries.iter().filter(|e| e.operation == '+').count();
    let total_deallocations = entries.iter().filter(|e| e.operation == '-').count();
    let total_bytes_allocated: usize = entries
        .iter()
        .filter(|e| e.operation == '+')
        .map(|e| e.size)
        .sum();
    let peak_memory = entries.iter().map(|e| e.peak).max().unwrap_or(0);
    let final_memory = entries.last().map(|e| e.current).unwrap_or(0);

    println!("\nSummary:");
    println!("--------");
    println!("Total allocations:   {}", total_allocations);
    println!("Total deallocations: {}", total_deallocations);
    println!("Bytes allocated:     {}", total_bytes_allocated);
    println!("Peak memory:         {} bytes", peak_memory);
    println!("Final memory:        {} bytes", final_memory);

    // Analyze allocation patterns
    println!("\nAllocation Patterns:");
    println!("-------------------");
    analyze_patterns(&entries);

    Ok(())
}

fn parse_entry(line: &str) -> Option<LogEntry> {
    let parts: Vec<&str> = line.trim().split('|').collect();
    if parts.len() == 5 {
        Some(LogEntry {
            timestamp: parts[0].parse().ok()?,
            operation: parts[1].chars().next()?,
            size: parts[2].parse().ok()?,
            current: parts[3].parse().ok()?,
            peak: parts[4].parse().ok()?,
        })
    } else {
        None
    }
}

fn analyze_patterns(entries: &[LogEntry]) {
    // Group allocations by size
    let mut size_groups = std::collections::HashMap::new();
    for entry in entries.iter().filter(|e| e.operation == '+') {
        *size_groups.entry(entry.size).or_insert(0) += 1;
    }

    // Show most common allocation sizes
    println!("\nMost Common Allocation Sizes:");
    let mut sizes: Vec<_> = size_groups.into_iter().collect();
    sizes.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for (size, count) in sizes.iter().take(5) {
        println!("  {} bytes: {} times", size, count);
    }

    // Calculate allocation rate
    if entries.len() >= 2 {
        let duration = entries.last().unwrap().timestamp - entries[0].timestamp;
        let rate = entries.len() as f64 / (duration as f64 / 1_000_000.0);
        println!("\nAllocation Rate: {:.1} ops/sec", rate);
    }

    // Identify potential memory leaks
    let mut outstanding = std::collections::HashMap::new();
    for entry in entries {
        match entry.operation {
            '+' => *outstanding.entry(entry.size).or_insert(0) += 1,
            '-' => *outstanding.entry(entry.size).or_insert(0) -= 1,
            _ => {}
        }
    }

    let leaks: Vec<_> = outstanding.iter().filter(|(_, &count)| count > 0).collect();

    if !leaks.is_empty() {
        println!("\nPotential Memory Leaks:");
        for (&size, &count) in leaks {
            println!("  {} bytes: {} allocation(s) not freed", size, count);
        }
    }
}
