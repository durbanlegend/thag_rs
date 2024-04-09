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
use std::io::{stdout, BufReader, ErrorKind, Read, Write};

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

        // Print separator line
        for _ in 0..width {
            print!("-");
        }
        println!();

        // Print editor pane
        if let Some(ref file) = current_file {
            println!("Editing file: {}", file);

            // Read the file content into a buffer (moved outside the inner loop)
            // let editor_buffer_string = std::fs::read_to_string(file)?;

            // Open the in-place editor with the existing file
            let mut in_place_editor: InPlaceFile = recycle(file)?;
            let mut reader = BufReader::new(in_place_editor.reader());

            // in_place_editor.save(); // test

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

            loop {
                let line = editor.readline(">> ")?;

                if line.is_empty() {
                    break;
                }

                if line.starts_with(':') {
                    {
                        let command = line.trim_start_matches(':');
                        match command {
                            "save" => {
                                // Implement logic to save the current editor buffer to the file
                                in_place_editor.save();
                                in_place_editor = recycle(file)?;
                                println!("File saved successfully!");
                            }
                            "run" => {
                                // Implement logic to save the buffer, build and run the program using Cargo
                                println!("Running the program...");
                                // (Call your existing build and run functions here)
                            }
                            "help" => println!("Available commands: save, run, quit"),
                            "quit" => return Err(ReadlineError::Eof),
                            _ => println!(
                                "Unknown command {command}. Type ':help' for a list of commands."
                            ),
                        }
                        // Ok(())
                    };
                } else {
                    // Process user input line
                    // (replace with logic to modify the in-place editor buffer)
                    // in_place_editor.write_all(line.as_bytes())?; // Example write operation
                    // writeln!(writer, "{line}")?;
                    // for line in editor_buffer_vec.iter() {
                    //     writeln!(writer, "{line}")?;
                    // }
                    editor_buffer_vec.push(line.to_string());
                }
            }

            // Save the buffer to the file (moved inside the if block)
            if true {
                // std::fs::write(current_file.as_ref().unwrap(), editor_buffer.as_bytes())?;
                let mut writer = in_place_editor.writer();
                for line in editor_buffer_vec.iter() {
                    writeln!(writer, "{line}");
                }
            }
        } else {
            // Handle scenario where no file is loaded (show message?)
            println!("TODO: No file loaded");
        }

        // Move cursor back to the beginning of the editor pane for the next iteration
        cursor::MoveUp(height as u16 - 3);
        cursor::MoveToColumn(0);
    }
}

fn recycle(file: &String) -> Result<InPlaceFile> {
    let in_place_editor = InPlace::new(file.as_str())
        .open()
        .map_err(|_err| ReadlineError::from(ErrorKind::Other))?;
    Ok(in_place_editor)
}
