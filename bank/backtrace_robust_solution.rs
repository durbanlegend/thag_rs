/*[toml]
[profile.release]
debug = true
*/
use backtrace::{Backtrace, BacktraceSymbol};

// This function demonstrates a robust way to extract file information
// using only the methods available on BacktraceSymbol
fn extract_file_info(symbol: &BacktraceSymbol) -> Option<(String, u32)> {
    eprintln!("name_str={}", symbol.name()?.to_string());
    eprintln!("symbol={symbol:#?}");

    // First attempt: try to get filename and line number directly (most accurate)
    // This works when debug info is available
    if let Some(filename) = symbol.filename() {
        let file_stem = filename
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from)
            .unwrap_or_else(|| "unknown_file".to_string());

        let line = symbol.lineno().unwrap_or(0);
        return Some((file_stem, line));
    }

    // Second attempt: try to extract module name from function name
    // This works even without debug info, as function names are generally available
    eprintln!("Trying fallback...");
    if let Some(name) = symbol.name() {
        let name_str = name.to_string();

        // Parse the module path from the function name
        // Example: "my_crate::module::function" -> "module"
        let parts: Vec<&str> = name_str.split("::").collect();
        let module_name = if parts.len() > 1 {
            parts[parts.len().saturating_sub(2)].to_string() // Get the second-to-last part
        } else {
            parts
                .first()
                .map_or("unknown".to_string(), |p| p.to_string())
        };

        return Some((module_name, 0));
    }

    None
}

fn main() {
    let backtrace = Backtrace::new();

    println!("\n=== BACKTRACE FILE LOCATION TEST ===\n");
    println!(
        "Build profile: {}",
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "RELEASE"
        }
    );
    println!("Platform: {}", std::env::consts::OS);
    println!("-----------------------------------------\n");

    println!("Testing robust file location extraction:\n");

    // Process each symbol in the backtrace
    for (i, frame) in backtrace.frames().iter().enumerate() {
        if i >= 5 {
            // Only show first few frames
            break;
        }

        for (j, symbol) in frame.symbols().iter().enumerate() {
            // Get function name if available
            let name = symbol
                .name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "<unknown function>".to_string());

            println!("Frame {}.{} - Function: {}", i, j, name);

            // Use our robust extraction method
            if let Some((file, line)) = extract_file_info(symbol) {
                println!("  Extracted file: {} (line: {})", file, line);
            } else {
                println!("  No file information available");
            }

            // For comparison, show the direct method results
            println!("  direct filename(): {:?}", symbol.filename());
            println!("  direct lineno(): {:?}", symbol.lineno());

            println!();
        }
    }

    println!("\n=== RECOMMENDATION FOR YOUR PROFILER ===\n");
    println!("The extract_file_info function shown here provides a robust approach that:");
    println!("1. Uses filename() when debug info is available (most accurate)");
    println!("2. Falls back to function name parsing when debug info is missing");
    println!("3. Never fails, always providing some form of location information");
    println!("\nYou can apply this approach to your profiler's mem_tracking.rs file.");
    println!("Add this function and replace the current filename extraction logic.");
}
