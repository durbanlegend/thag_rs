//! [dependencies]
//! crossterm = "0.27.0"
//! in-place = "0.2.0"
//! rustyline = "14.0.0"

use crossterm::{
    cursor, execute,
    terminal::{self, size, ClearType},
};
use in_place::{InPlace, InPlaceFile};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::fs;
use std::io::{stdout, BufReader, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let mut editor = DefaultEditor::new()?; // Use DefaultEditor for history
    let current_file: Option<String> = Some("examples/repl_000005.rs".to_string());

    loop {
        // Get terminal size for pane management
        let (width, height) = size()?;

        // Clear the terminal
        execute!(stdout(), terminal::Clear(ClearType::All));

        // Print output pane (replace with your actual output logic)

        println!("--- Output Pane ---");
        let file: String = if let Some(ref file) = current_file {
            println!("Editing file: {}", file);
            file.to_string()
        } else {
            // Handle scenario where no file is loaded (show message?)
            println!("TODO: No file loaded");
            create_next_repl_file().display().to_string()
        };

        // Read the file content into a buffer (moved outside the inner loop)
        // let editor_buffer_string = std::fs::read_to_string(file)?;

        // Open the in-place editor with the existing file
        let mut in_place_editor: InPlaceFile = new_editor(&file)?;
        let mut reader = BufReader::new(in_place_editor.reader());

        let mut editor_buffer_string = String::new();
        reader.read_to_string(&mut editor_buffer_string);

        let mut editor_buffer_vec = editor_buffer_string
            .lines()
            .map(str::to_string)
            .collect::<Vec<String>>();

        // Display it to stdout:
        for line in editor_buffer_vec.iter() {
            println!("{line}");
        }

        // Print separator line
        for _ in 0..width {
            print!("-");
        }
        println!();

        // Print editor pane
        loop {
            let line = editor.readline(">> ")?;

            if line.is_empty() {
                break;
            }

            if line.starts_with(':') {
                in_place_editor =
                    handle_repl_command(&line, in_place_editor, &editor_buffer_vec, &file)?;
                // Ok(());
            } else {
                // Process user input line
                // (replace with logic to modify the in-place editor buffer)
                // in_place_editor.write_all(line.as_bytes())?; // Example write operation
                editor_buffer_vec.push(line.to_string());
            }
        }

        // Save the buffer to the file (moved inside the if block)
        if true {
            // std::fs::write(current_file.as_ref().unwrap(), editor_buffer.as_bytes())?;
            write_buffer_to_tempfile(in_place_editor, editor_buffer_vec);
        }

        // Move cursor back to the beginning of the editor pane for the next iteration
        cursor::MoveUp(height as u16 - 3);
        cursor::MoveToColumn(0);
    }
}

fn handle_repl_command(
    line: &str, // TODO get this from in_place_editor?
    in_place_editor: InPlaceFile,
    editor_buffer_vec: &Vec<String>,
    file: &String,
) -> Result<InPlaceFile> {
    let command = line.trim_start_matches(':');
    match command {
        "save" => {
            // Implement logic to save the current editor buffer to the file
            // let path = in_place_editor.path();
            let mut writer = in_place_editor.writer();
            for line in editor_buffer_vec.iter() {
                writeln!(writer, "{line}");
            }

            in_place_editor.save();
            println!("File saved successfully!");
            return new_editor(&file);
        }
        "run" => {
            // Implement logic to save the buffer, build and run the program using Cargo
            println!("Running the program...");
            // (Call your existing build and run functions here)
        }
        "help" => println!("Available commands: save, run, quit"),
        "quit" => return Err(ReadlineError::Eof),
        _ => println!("Unknown command {command}. Type ':help' for a list of commands."),
    }
    Ok(in_place_editor)
}

fn write_buffer_to_tempfile(in_place_editor: InPlaceFile, editor_buffer_vec: Vec<String>) {
    let mut writer = in_place_editor.writer();
    for line in editor_buffer_vec.iter() {
        writeln!(writer, "{line}");
    }
}

fn new_editor(file: &String) -> Result<InPlaceFile> {
    let in_place_editor = InPlace::new(file.as_str())
        .open()
        .map_err(|_err| ReadlineError::from(ErrorKind::Other))?;
    Ok(in_place_editor)
}

pub(crate) fn create_next_repl_file() -> PathBuf {
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
                if num_str.len() == 6 && num_str.chars().all(char::is_numeric) {
                    Some(num_str.parse::<u32>().unwrap())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // debug!("existing_files={existing_files:?}");

    let next_file_num = match existing_files.as_slice() {
        [] => 0, // No existing files, start with 000000
        _ if existing_files.contains(&999_999) => {
            // Wrap around and find the first gap
            for i in 0..999_999 {
                if !existing_files.contains(&i) {
                    return create_file(examples_dir, i);
                }
            }
            panic!("Cannot create new file: all possible filenames already exist in the examples directory.");
        }
        _ => existing_files.iter().max().unwrap() + 1, // Increment from highest existing number
    };

    create_file(examples_dir, next_file_num)
}

pub(crate) fn create_file(examples_dir: &Path, num: u32) -> PathBuf {
    let padded_num = format!("{:06}", num);
    let filename = format!("repl_{}.rs", padded_num);
    let path = examples_dir.join(&filename);
    fs::File::create(path.clone()).expect("Failed to create file");
    println!("Created file: {}", filename);
    path
}
