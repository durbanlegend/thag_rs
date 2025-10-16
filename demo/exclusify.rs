use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Process a folded file to calculate exclusive times
///
/// This function converts inclusive time profiling data to exclusive time:
/// - Inclusive time: total time spent in a function including all child calls
/// - Exclusive time: time spent only in the function itself, excluding child calls
//# Purpose: Prototype converting inclusive elapsed times to exclusive for flamegraphs in order to avoid double counting.
//# Categories: profiling, prototype
fn process_folded_file(input_path: &Path, output_path: &Path) -> std::io::Result<()> {
    println!("Processing: {}", input_path.display());
    println!("Converting inclusive time profile to exclusive time profile");

    // Read input file
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    // Store header lines to preserve them
    let mut header_lines = Vec::new();

    // Store stack lines as (stack_str, time) pairs
    let mut stack_lines: Vec<(String, u64)> = Vec::new();
    let mut line_count = 0;

    // First pass: Parse the file and separate headers from stack lines
    for line in reader.lines() {
        let line = line?;
        line_count += 1;

        // Preserve comment/header lines
        if line.starts_with('#') || line.trim().is_empty() {
            header_lines.push(line);
            continue;
        }

        // Parse line: "stack time"
        let parts: Vec<&str> = line.rsplitn(2, ' ').collect();
        if parts.len() != 2 {
            eprintln!("Warning: Invalid line format at line {line_count}: {line}");
            continue;
        }

        let stack_str = parts[1].trim();
        let time = match parts[0].parse::<u64>() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Warning: Invalid time value at line {line_count}: {e}");
                continue;
            }
        };

        // Store the stack line
        stack_lines.push((stack_str.to_string(), time));
    }

    let mut stack_lines: Vec<(String, u64)> = stack_lines.into_iter().rev().collect();

    // Calculate exclusive times using a sequential approach
    let mut exclusive_times: Vec<(String, u64)> = vec![];
    // let mut inclusive_times: Vec<(String, u64)> = stack_lines.clone();

    let len = stack_lines.len();

    for _i in 1..=len {
        let mut parent = stack_lines.remove(0);

        eprintln!("parent=({}, {})", parent.0, parent.1);

        // For each stack, find its direct descendants and subtract their inclusive time from the parent
        for (candidate, time_ref) in &mut stack_lines {
            if !candidate.starts_with(&parent.0) {
                break;
            }
            let parts: Vec<&str> = candidate.split(';').collect();
            let parent_stack = parts[..parts.len() - 1].join(";");
            if parent_stack == parent.0 {
                let before = parent.1;
                parent.1 = parent.1.saturating_sub(*time_ref);
                eprintln!("candidate=({candidate}, {before}->{})", parent.1);
            }
        }
        exclusive_times.push(parent);
    }

    let exclusive_times: Vec<(String, u64)> = exclusive_times.into_iter().rev().collect();

    // Write output file
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;
    let mut writer = BufWriter::new(output_file);

    // Write original headers
    for header in &header_lines {
        writeln!(writer, "{header}")?;
    }

    // Add a note about this being exclusive time
    writeln!(writer, "# Converted to exclusive time by thag-exclusify")?;
    writeln!(writer)?;

    for (stack, exclusive) in &exclusive_times {
        writeln!(writer, "{stack} {exclusive}")?;
    }

    writer.flush()?;

    println!("Successfully processed {line_count} lines");
    println!("Output written to: {}", output_path.display());
    println!("Found {len} stacks");

    // Sum up exclusive times to validate (should equal root inclusive times)
    let total_exclusive: u64 = exclusive_times.iter().map(|(_, time)| time).sum();
    println!("Total exclusive time: {total_exclusive} µs");

    println!("Successfully converted time profile from inclusive to exclusive time");

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("thag-exclusify: Converts inclusive time profiles to exclusive time");
        eprintln!(
            "Usage: {} <input_folded_file> <output_folded_file>",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    // Ensure input file exists
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist: {}", input_path.display());
        std::process::exit(1);
    }

    match process_folded_file(input_path, output_path) {
        Ok(()) => {
            // Success message is printed in the function
        }
        Err(e) => {
            eprintln!("Error: Failed to process profile: {e}");
            std::process::exit(1);
        }
    }
}
