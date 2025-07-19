/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["demo"] }
*/

/// Test script to verify the profiler demo visualization feature works
//# Purpose: test if the error exists, then periodically to see if it persists.
//# Categories: profiling, testing
use thag_profiler::visualization;

fn main() {
    println!("🧪 Testing thag_profiler demo visualization access...");

    // Test that we can access the visualization module
    match visualization::find_latest_profile_files("test", 1) {
        Ok(files) => {
            println!("✅ Successfully accessed visualization::find_latest_profile_files");
            println!("   Found {} files", files.len());
        }
        Err(e) => {
            println!("✅ Successfully accessed visualization::find_latest_profile_files");
            println!("   Expected error (no test files): {}", e);
        }
    }

    // Test config creation
    let config = visualization::VisualizationConfig::default();
    println!("✅ Successfully created VisualizationConfig");
    println!("   Title: {}", config.title);
    println!("   Count name: {}", config.count_name);

    println!("🎉 All visualization module access tests passed!");
    println!("📚 The demo feature is working correctly and can be used in demo scripts.");
}
