use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Parser)]
#[command(name = "analyze_alloc_log")]
#[command(about = "Analyze thag allocation logs")]
struct Args {
    /// Path to the allocation log file
    #[arg(default_value = "thag-profile.alloc.log")]
    log_file: PathBuf,
}

#[derive(Debug)]
#[allow(dead_code)]
struct LogEntry {
    timestamp: u128,
    operation: char,
    size: usize,
    stack: Vec<String>,
}

fn read_entry(reader: &mut BufReader<File>) -> io::Result<LogEntry> {
    let start_pos = reader.stream_position()?;

    // Try to read timestamp (16 bytes)
    let mut timestamp_buf = [0u8; 16];
    if reader.read_exact(&mut timestamp_buf).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "EOF during timestamp",
        ));
    }
    let timestamp = u128::from_ne_bytes(timestamp_buf);

    // Read operation (1 byte)
    let mut op_byte = [0u8; 1];
    reader.read_exact(&mut op_byte)?;
    let operation = op_byte[0] as char;

    // Validate operation
    if operation != '+' && operation != '-' {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid operation '{operation}' at position {start_pos}"),
        ));
    }

    // Read size (8 bytes)
    let mut size_buf = [0u8; 8];
    reader.read_exact(&mut size_buf)?;
    let size = usize::from_ne_bytes(size_buf);

    // Validate size (use reasonable limits)
    if size > 1024 * 1024 * 1024 {
        // 1GB max allocation
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid size {size} at position {start_pos}"),
        ));
    }

    // Read stack length (4 bytes)
    let mut stack_len_buf = [0u8; 4];
    reader.read_exact(&mut stack_len_buf)?;
    let stack_len = u32::from_ne_bytes(stack_len_buf);

    // Validate stack length
    if stack_len > 100 {
        // reasonable max stack depth
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid stack length {stack_len} at position {start_pos}"),
        ));
    }

    // Read stack data
    let mut stack = Vec::new();
    if stack_len > 0 {
        let mut stack_data = Vec::new();
        let mut byte = [0u8; 1];
        let mut frames_read = 0;
        let mut frame_size = 0;

        while frames_read < stack_len {
            if reader.read_exact(&mut byte).is_err() {
                println!(
                    "EOF during stack data at position {}",
                    reader.stream_position()?
                );
                break;
            }

            // Validate frame size
            frame_size += 1;
            if frame_size > 256 {
                // reasonable max frame name length
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Frame too long at position {}", reader.stream_position()?),
                ));
            }

            stack_data.push(byte[0]);
            if byte[0] == b';' {
                frames_read += 1;
                frame_size = 0;
            }
        }

        if let Ok(stack_str) = String::from_utf8(stack_data) {
            stack = stack_str
                .split(';')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
    }

    // Try to read newline
    let _ = reader.read_exact(&mut op_byte);

    println!(
        "Read entry at {}: op={}, size={}, stack_len={}, actual_frames={}",
        start_pos,
        operation,
        size,
        stack_len,
        stack.len()
    );

    Ok(LogEntry {
        timestamp,
        operation,
        size,
        stack,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = File::open(&args.log_file)?;
    let mut reader = BufReader::new(file);

    // Debug file size
    let file_size = reader.seek(SeekFrom::End(0))?;
    println!("File size: {file_size} bytes");
    reader.rewind()?;

    // Skip header (read until double newline)
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        if line.trim().is_empty() {
            break;
        }
        println!("Header: {}", line.trim());
        line.clear();
    }

    // Debug position after header
    let start_pos = reader.stream_position()?;
    println!("\nStarting to read entries at position: {start_pos}");

    // Try to read first few bytes to verify format
    let mut peek_buf = [0u8; 32];
    if let Ok(n) = reader.read(&mut peek_buf) {
        println!("First {} bytes after header: {:?}", n, &peek_buf[..n]);
        // Reset position
        reader.seek(SeekFrom::Start(start_pos))?;
    }

    let mut entries = Vec::new();
    while reader.stream_position()? < file_size {
        match read_entry(&mut reader) {
            Ok(entry) => {
                static COUNT: AtomicUsize = AtomicUsize::new(0);
                let count = COUNT.fetch_add(1, Ordering::SeqCst);
                if true {
                    // count < 5 || count % 100 == 0 {
                    println!(
                        "\nEntry {}: op={}, size={}, stack_len={}, stack={:?}",
                        count,
                        entry.operation,
                        entry.size,
                        entry.stack.len(),
                        entry.stack
                    );
                }
                entries.push(entry);
            }
            Err(e) => {
                println!(
                    "Error reading entry at position {}: {}",
                    reader.stream_position()?,
                    e
                );
                break;
            }
        }
    }

    println!("\nRead {} entries", entries.len());

    if !entries.is_empty() {
        analyze_stack_traces(&entries);
    }

    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn analyze_stack_traces(entries: &[LogEntry]) {
    println!("\nStack Trace Analysis:");
    println!("-------------------");

    // Group allocations by stack trace
    let mut allocation_totals: HashMap<String, (usize, usize)> = HashMap::new(); // (count, total_bytes)

    for entry in entries.iter().filter(|e| e.operation == '+') {
        let stack_key = entry.stack.join(";");
        let totals = allocation_totals.entry(stack_key).or_insert((0, 0));
        totals.0 += 1; // increment count
        totals.1 += entry.size; // add bytes
    }

    // Sort by total bytes
    let mut sorted_allocs: Vec<_> = allocation_totals.into_iter().collect();
    sorted_allocs.sort_by_key(|(_, (_, bytes))| std::cmp::Reverse(*bytes));

    println!("\nTop Allocation Sites:");
    println!("Size      Count  Stack Trace");
    println!("--------------------------------");

    for (stack, (count, bytes)) in sorted_allocs.iter().take(10) {
        if stack.is_empty() {
            println!("{bytes:8}  {count:5}  (no stack trace)");
        } else {
            println!("{bytes:8}  {count:5}  {stack}");
        }
    }

    // Add summary
    if !entries.is_empty() {
        println!("\nStack Trace Summary:");
        println!("------------------");
        let total_entries = entries.len();
        let entries_with_stack = entries.iter().filter(|e| !e.stack.is_empty()).count();
        println!("Total entries: {total_entries}");
        println!(
            "Entries with stack trace: {entries_with_stack} ({:.1}%)",
            (entries_with_stack as f64 / total_entries as f64) * 100.0
        );
    }
}
