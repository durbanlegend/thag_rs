use std::env;
use std::fs;
use std::path::Path;

/// Prototype of creating files named sequentially from repl_000000.rs to
/// repl_999999.rs in a rs_script/demo subdirectory of the OS's temporary
/// directory. The need is to generate well-behaved and consistent human-readable
/// names for temporary programs generated from REPL expressions.
//# Purpose: Demo sequential file creation and the kind of code that is well suited to generation by an LLM.
fn main() {
    let demo_dir = env::temp_dir().join("rs_script").join("demo");

    // Ensure demo subdirectory exists
    fs::create_dir_all(&demo_dir).expect("Failed to create demo directory");

    // Find existing files with the pattern repl_<nnnnnn>.rs
    let existing_files: Vec<_> = fs::read_dir(&demo_dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            // println!("path={path:?}, path.is_file()={}, path.extension()?.to_str()={:?}, path.file_stem()?.to_str()={:?}", path.is_file(), path.extension()?.to_str(), path.file_stem()?.to_str());
            if path.is_file()
                && path.extension()?.to_str() == Some("rs")
                && path.file_stem()?.to_str()?.starts_with("repl_")
            {
                let stem = path.file_stem().unwrap();
                let num_str = stem.to_str().unwrap().trim_start_matches("repl_");
                // println!("stem={stem:?}; num_str={num_str}");
                if num_str.len() == 6 && num_str.chars().all(|c| c.is_numeric()) {
                    Some(num_str.parse::<u32>().unwrap())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    println!("existing_files={existing_files:?}");

    let next_file_num = match existing_files.as_slice() {
        [] => 0, // No existing files, start with 000000
        _ if existing_files.contains(&999999) => {
            // Wrap around and find the first gap
            for i in 0..999999 {
                if !existing_files.contains(&i) {
                    return create_file(&demo_dir, i);
                }
            }
            panic!("Cannot create new file: all possible filenames already exist in the demo directory.");
        }
        _ => existing_files.iter().max().unwrap() + 1, // Increment from highest existing number
    };

    create_file(&demo_dir, next_file_num);
}

fn create_file(demo_dir: &Path, num: u32) {
    let padded_num = format!("{:06}", num);
    let filename = format!("repl_{}.rs", padded_num);
    let path = demo_dir.join(&filename);
    fs::File::create(path.clone()).expect("Failed to create file");
    println!("Created file: {path:#?}");
}
