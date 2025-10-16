use memory_stats::memory_stats;

if let Some(usage) = memory_stats() {
    println!("Current physical memory usage: {}", thousands(usage.physical_mem));
    println!("Current virtual memory usage: {}", thousands(usage.virtual_mem));
} else {
    println!("Couldn't get the current memory usage :(");
}

fn thousands(n: usize) -> String {
 n.to_string()
    .as_bytes()
    .rchunks(3)
    .rev()
    .map(std::str::from_utf8)
    .collect::<Result<Vec<&str>, _>>()
    .unwrap()
    .join(",")  // separator
}
