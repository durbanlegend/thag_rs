use std::fs;
use std::path::Path;

fn main() {
    let examples_dir = Path::new("examples");

    // Ensure examples subdirectory exists
    fs::create_dir_all(examples_dir).expect("Failed to create examples directory");

    // Find existing files with the pattern repl_<nnnnnn>.rs
    let existing_files: Vec<_> = fs::read_dir(examples_dir)
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
                    return create_file(examples_dir, i);
                }
            }
            panic!("Cannot create new file: all possible filenames already exist in the examples directory.");
        }
        _ => existing_files.iter().max().unwrap() + 1, // Increment from highest existing number
    };

    create_file(examples_dir, next_file_num);
}

fn create_file(examples_dir: &Path, num: u32) {
    let padded_num = format!("{:06}", num);
    let filename = format!("repl_{}.rs", padded_num);
    let path = examples_dir.join(&filename);
    fs::File::create(path).expect("Failed to create file");
    println!("Created file: {path:#?}");
}
