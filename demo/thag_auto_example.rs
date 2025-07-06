/*[toml]
[dependencies]
# The 'thag-auto' keyword automatically resolves dependencies based on your environment:
# - Default: Uses crates.io (no environment variables needed)
# - Development: Set THAG_DEV_PATH=/absolute/path/to/thag_rs (e.g. $PWD not .)
# - Git: Set THAG_GIT_REF=main to use git repository instead of crates.io
# Note: Run with 'thag script.rs' not 'cargo build' to enable thag-auto processing
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
thag_proc_macros = { version = "0.2, thag-auto" }
thag_profiler = { version = "0.1, thag-auto" }
*/

/// Example script demonstrating proper thag-auto usage.
/// This shows how to use the thag-auto keyword for automatic dependency resolution.
///
/// The thag-auto system allows scripts to work in different environments:
/// - Development: Uses local path when THAG_DEV_PATH is set
/// - Git: Uses git repository when THAG_GIT_REF is set
/// - Default: Uses crates.io versions (may require published versions)
///
/// If you get a "version not found" error, it means the specified version
/// doesn't exist on crates.io yet. Set THAG_DEV_PATH or THAG_GIT_REF to use
/// local or git versions instead.
//# Purpose: Demonstrate thag-auto dependency resolution system
//# Categories: example, documentation
use thag_rs::logging::*;

fn main() {
    init_logger();

    println!("This is an example of thag-auto dependency resolution!");
    println!("The dependencies in this script use 'thag-auto' to automatically");
    println!("resolve to the appropriate source based on your environment.");

    println!("\nEnvironment variables that affect thag-auto:");
    println!("- THAG_DEV_PATH: Use local development path");
    println!("- THAG_GIT_REF: Use git repository");
    println!("- (none): Use crates.io (default)");

    println!("\nCurrent environment:");
    if let Ok(dev_path) = std::env::var("THAG_DEV_PATH") {
        println!("✓ THAG_DEV_PATH is set to: {}", dev_path);
    } else {
        println!("✗ THAG_DEV_PATH is not set");
    }

    if let Ok(git_ref) = std::env::var("THAG_GIT_REF") {
        println!("✓ THAG_GIT_REF is set to: {}", git_ref);
    } else {
        println!("✗ THAG_GIT_REF is not set");
    }

    println!("\nTo test different modes:");
    println!("1. Default (crates.io): unset THAG_DEV_PATH && unset THAG_GIT_REF && thag demo/thag_auto_example.rs");
    println!("2. Development: export THAG_DEV_PATH=$PWD && thag demo/thag_auto_example.rs");
    println!("3. Git: export THAG_GIT_REF=main && thag demo/thag_auto_example.rs");
}
