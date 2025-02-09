use chrono::{DateTime, Local, TimeZone};
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
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
    operation: char,
    size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut file = File::open(&args.log_file)?;

    // Read and parse header (text until we hit a non-UTF8 byte)
    let mut header = Vec::new();
    let mut byte = [0u8; 1];

    while file.read_exact(&mut byte).is_ok() {
        if byte[0] == b'+' || byte[0] == b'-' {
            // Found first entry, break
            break;
        }
        header.push(byte[0]);
    }

    println!("Log File Analysis");
    println!("================\n");
    println!("Header:\n{}", String::from_utf8_lossy(&header));

    // Process entries
    let mut entries = Vec::new();
    let size_bytes = std::mem::size_of::<usize>();
    let mut size_buf = vec![0u8; size_bytes];

    // Handle the first entry (we already read the operation)
    if byte[0] == b'+' || byte[0] == b'-' {
        if file.read_exact(&mut size_buf).is_ok() {
            let size = usize::from_ne_bytes(size_buf.clone().try_into().unwrap());
            entries.push(LogEntry {
                operation: byte[0] as char,
                size,
            });
        }
    }

    // Read remaining entries
    while file.read_exact(&mut byte).is_ok() {
        if byte[0] != b'\n' {
            if let Ok(()) = file.read_exact(&mut size_buf) {
                let size = usize::from_ne_bytes(size_buf.clone().try_into().unwrap());
                if byte[0] == b'+' || byte[0] == b'-' {
                    entries.push(LogEntry {
                        operation: byte[0] as char,
                        size,
                    });
                }
            }
        }
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
}
