use std::fs;
use std::io;
use std::path::PathBuf;

/// Demo listing files on disk. If you want a sorted list, you will need to amend the
/// program to collect the entries into a Vec and sort that.
//# Purpose: Simple demonstration.
//# Categories: basic, technique
fn display_file_if_exists(path: &PathBuf) -> io::Result<()> {
    if path.exists() {
        println!("File: {:?}", path);
    }
    Ok(())
}

fn display_dir_contents(path: &PathBuf) -> io::Result<()> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?;

        println!("Directory listing for {:?}", path);
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            println!(
                "  {:?} ({})",
                file_name,
                if file_type.is_dir() {
                    "Directory"
                } else {
                    "File"
                }
            );
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let source_path = PathBuf::from("demo/list_files.rs");
    let target_dir_path = PathBuf::from("demo");

    // Display file listing
    display_file_if_exists(&source_path)?;

    // Display directory contents
    display_dir_contents(&target_dir_path)?;

    // Check if neither file nor directory exist
    if !source_path.exists() && !target_dir_path.exists() {
        println!("No files found. You may want to edit the paths and try again");
    }

    Ok(())
}
