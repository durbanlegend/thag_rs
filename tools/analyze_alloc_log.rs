use chrono::{DateTime, Local, TimeZone};
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
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
    stack: Vec<String>, // Add stack trace
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut file = File::open(&args.log_file)?;

    println!("Reading file: {}", args.log_file.display());

    // Read header (text until first timestamp)
    let mut header = Vec::new();
    let mut byte = [0u8; 1];

    while file.read_exact(&mut byte).is_ok() {
        if !byte[0].is_ascii() || byte[0] == b'+' || byte[0] == b'-' {
            file.seek(std::io::SeekFrom::Current(-1))?;
            break;
        }
        header.push(byte[0]);
    }

    println!("Header:\n{}", String::from_utf8_lossy(&header));

    let mut entries = Vec::new();
    let mut timestamp_buf = [0u8; 16]; // u128
    let mut size_buf = [0u8; 8]; // usize
    let mut stack_len_buf = [0u8; 4]; // u32

    println!("Starting to read entries...");

    while let Ok(()) = file.read_exact(&mut timestamp_buf) {
        println!("Read timestamp bytes: {:?}", &timestamp_buf);
        // Read operation
        if file.read_exact(&mut byte).is_ok() {
            let op = byte[0] as char;
            if op != '+' && op != '-' {
                println!("Invalid operation: {}", op);
                break;
            }
            println!("Read operation byte: {}", byte[0] as char);

            // Read size
            if file.read_exact(&mut size_buf).is_ok() {
                let timestamp = u128::from_ne_bytes(timestamp_buf);
                let size = usize::from_ne_bytes(size_buf);
                println!("Read size bytes: {:?}", &size_buf);

                // Read stack length
                if file.read_exact(&mut stack_len_buf).is_ok() {
                    println!("Read stack length bytes: {:?}", &stack_len_buf);
                    let stack_len = u32::from_ne_bytes(stack_len_buf);

                    // Validate stack length
                    if stack_len > 1024 * 1024 {
                        // Sanity check: max 1MB stack data
                        println!("Invalid stack length: {}", stack_len);
                        break;
                    }

                    let mut stack = Vec::new();
                    if stack_len > 0 {
                        let mut stack_data = vec![0u8; stack_len as usize];
                        if file.read_exact(&mut stack_data).is_ok() {
                            if let Ok(stack_str) = String::from_utf8(stack_data) {
                                stack = stack_str
                                    .split(';')
                                    .filter(|s| !s.is_empty())
                                    .map(String::from)
                                    .collect();
                            }
                        }
                    }

                    // Read newline
                    let _ = file.read_exact(&mut byte);

                    entries.push(LogEntry {
                        timestamp,
                        operation: op,
                        size,
                        stack,
                    });

                    if entries.len() % 100 == 0 {
                        println!("Processed {} entries", entries.len());
                    }
                }
            }
        }
    }

    println!("Read {} entries", entries.len());

    if !entries.is_empty() {
        analyze_stack_traces(&entries);
    }

    // Generate summary
    let total_allocations = entries.iter().filter(|e| e.operation == '+').count();
    let total_deallocations = entries.iter().filter(|e| e.operation == '-').count();
    let total_bytes_allocated: usize = entries
        .iter()
        .filter(|e| e.operation == '+')
        .map(|e| e.size)
        .sum();

    println!("\nSummary:");
    println!("--------");
    println!("Total allocations:   {}", total_allocations);
    println!("Total deallocations: {}", total_deallocations);
    println!("Bytes allocated:     {}", total_bytes_allocated);

    // Analyze allocation patterns
    println!("\nAllocation Patterns:");
    println!("-------------------");
    analyze_patterns(&entries);

    analyze_allocation_lifetimes(&entries);

    // for entry in entries.iter() {
    //     println!("{entry:?}");
    // }
    Ok(())
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

    // Enhanced leak analysis
    let mut outstanding = std::collections::HashMap::new();
    let mut leak_categories = HashMap::new(); // size range -> (count, total bytes)
    let categories = [
        (0, 16, "Tiny (<16 bytes)"),
        (16, 64, "Small (16-63 bytes)"),
        (64, 256, "Medium (64-255 bytes)"),
        (256, 1024, "Large (256-1023 bytes)"),
        (1024, usize::MAX, "Very Large (1024+ bytes)"),
    ];

    // Track allocations and deallocations
    for entry in entries {
        match entry.operation {
            '+' => *outstanding.entry(entry.size).or_insert(0) += 1,
            '-' => *outstanding.entry(entry.size).or_insert(0) -= 1,
            _ => {}
        }
    }

    // Categorize leaks
    let mut total_leaked_bytes = 0;
    let mut total_leak_count = 0;

    println!("\nDetailed Leak Analysis:");
    println!("----------------------");

    let leaks: Vec<_> = outstanding.iter().filter(|(_, &count)| count > 0).collect();

    for (&size, &count) in &leaks {
        total_leaked_bytes += size * count as usize;
        total_leak_count += count;

        // Categorize this leak
        for &(min, max, category) in &categories {
            if size >= min && size < max {
                let entry = leak_categories.entry(category).or_insert((0, 0));
                entry.0 += count;
                entry.1 += size * count as usize;
                break;
            }
        }
    }

    // Print summary by category
    println!("\nLeak Summary by Category:");
    let mut categories_vec: Vec<_> = leak_categories.iter().collect();
    categories_vec.sort_by_key(|&(_, (_, bytes))| std::cmp::Reverse(bytes));

    for (category, (count, bytes)) in categories_vec {
        let percentage = (*bytes as f64 / total_leaked_bytes as f64) * 100.0;
        println!(
            "{:20} {:4} leaks, {:8} bytes ({:5.1}%)",
            category, count, bytes, percentage
        );
    }

    println!(
        "\nTotal Leaks: {} allocations, {} bytes",
        total_leak_count, total_leaked_bytes
    );

    // Show details of largest leaks
    println!("\nLargest Individual Leaks:");
    let mut largest_leaks: Vec<_> = leaks
        .iter()
        .map(|(&size, &count)| (size, count, size * count as usize))
        .collect();
    largest_leaks.sort_by_key(|&(_, _, total_size)| std::cmp::Reverse(total_size));

    for (size, count, total_size) in largest_leaks.iter().take(10) {
        let percentage = (*total_size as f64 / total_leaked_bytes as f64) * 100.0;
        if *count > 1 {
            println!(
                "{:6} bytes × {:3} = {:8} bytes ({:5.1}%)",
                size, count, total_size, percentage
            );
        } else {
            println!(
                "{:6} bytes {:16} = {:8} bytes ({:5.1}%)",
                size, "", total_size, percentage
            );
        }
    }
}

fn analyze_allocation_lifetimes(entries: &[LogEntry]) {
    println!("\nAllocation Lifetime Analysis:");
    println!("--------------------------");

    // Track allocation start times
    let mut allocation_starts: HashMap<usize, Vec<u128>> = HashMap::new();
    let mut size_lifetimes: HashMap<usize, Vec<u128>> = HashMap::new();

    let start_time = entries.first().map(|e| e.timestamp).unwrap_or(0);

    for entry in entries {
        match entry.operation {
            '+' => {
                allocation_starts
                    .entry(entry.size)
                    .or_default()
                    .push(entry.timestamp);
            }
            '-' => {
                if let Some(starts) = allocation_starts.get_mut(&entry.size) {
                    if let Some(start_time) = starts.pop() {
                        let lifetime = entry.timestamp - start_time;
                        size_lifetimes.entry(entry.size).or_default().push(lifetime);
                    }
                }
            }
            _ => {}
        }
    }

    println!("\nLifetime Analysis for Largest Leaks:");
    println!("Size      Allocs  Deallocs  Avg Lifetime    Leaked");
    println!("------------------------------------------------------");

    let sizes_to_analyze: Vec<_> = allocation_starts
        .iter()
        .map(|(&size, starts)| (size, starts.len()))
        .filter(|&(_, count)| count > 0)
        .collect();

    for (size, remaining) in sizes_to_analyze {
        let dealloc_count = size_lifetimes
            .get(&size)
            .map_or(0, |lifetimes| lifetimes.len());
        let alloc_count = remaining + dealloc_count; // Total allocations = remaining + deallocated

        let avg_lifetime = size_lifetimes.get(&size).map_or(0.0, |lifetimes| {
            if lifetimes.is_empty() {
                0.0
            } else {
                lifetimes.iter().sum::<u128>() as f64 / lifetimes.len() as f64
            }
        });

        // Convert to milliseconds
        let avg_ms = avg_lifetime / 1000.0;

        println!(
            "{:6}  {:8}  {:8}  {:8.1}ms  {:8}",
            size,
            alloc_count,
            dealloc_count,
            avg_ms,
            remaining // Number of leaks = allocations still in starts
        );
    }
}

fn analyze_stack_traces(entries: &[LogEntry]) {
    println!("\nStack Trace Analysis:");
    println!("-------------------");

    // Group allocations by stack trace
    let mut stack_allocs: HashMap<Vec<String>, Vec<&LogEntry>> = HashMap::new();

    for entry in entries.iter().filter(|e| e.operation == '+') {
        stack_allocs
            .entry(entry.stack.clone())
            .or_default()
            .push(entry);
    }

    // Sort by total bytes allocated
    let mut stack_stats: Vec<_> = stack_allocs
        .iter()
        .map(|(stack, allocs)| {
            let total_bytes: usize = allocs.iter().map(|e| e.size).sum();
            let count = allocs.len();
            (stack, count, total_bytes)
        })
        .collect();

    stack_stats.sort_by_key(|&(_, _, bytes)| std::cmp::Reverse(bytes));

    println!("\nTop Allocation Sites:");
    println!("Size      Count  Stack Trace");
    println!("--------------------------------");

    for (stack, count, bytes) in stack_stats.iter().take(10) {
        println!("{:8}  {:5}  {}", bytes, count, stack.join(";"));
    }
}
