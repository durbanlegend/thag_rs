use std::fs;

fn main() {
    let files = [
        "demo/factorial_main.rs",
        ".cargo/factorial_main/Cargo.toml",
        ".cargo/factorial_main/target/debug/factorial_main",
    ];

    let mut latest_file = None;
    let mut latest_modified = None;

    for file in files.iter() {
        let metadata = match fs::metadata(file) {
            Ok(val) => val,
            Err(_) => continue, // Ignore non-existent files
        };

        let modified_time = metadata.modified().unwrap(); // Handle potential errors

        println!("???????? file={file:#?}, modified_time={modified_time:#?}");

        if latest_modified.is_none() || modified_time > latest_modified.unwrap() {
            latest_file = Some(file);
            latest_modified = Some(modified_time);
        }
    }

    if let Some(file) = latest_file {
        println!("The most recently modified file is: {}", file);
    } else {
        println!("Error: No files found or error accessing them");
    }
}
