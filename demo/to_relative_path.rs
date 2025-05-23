use std::env;
use std::path::Path;

/// ChatGPT 4.1-generated script expresses an absolute path relative to the current working directory.
//# Purpose: Use `pathdiff` crate to compute a relative path releative to the CWD.
//# Categories: crates, technique
fn main() {
    // Get the argument (absolute path)
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <absolute-path>", args[0]);
        std::process::exit(1);
    }

    let abs_path = Path::new(&args[1]);
    if !abs_path.is_absolute() {
        eprintln!("Error: Provided path is not absolute.");
        std::process::exit(1);
    }

    // Get the current working directory
    let current_dir = env::current_dir().expect("Failed to get current working directory");

    // Compute the relative path
    let relative_path = pathdiff::diff_paths(abs_path, &current_dir).unwrap_or_else(|| {
        eprintln!("Could not compute relative path.");
        std::process::exit(1);
    });

    println!("{}", relative_path.display());
}
