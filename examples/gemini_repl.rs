//! [dependencies]
//! crossterm = "0.27.0"
//! rustyline = "14.0.0"

use crossterm::{
    cursor, execute,
    terminal::{self, size, ClearType},
};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::io::stdout;

// Function to handle REPL commands
fn handle_repl_command(command: &str) -> Result<()> {
    match command {
        "save" => {
            // Implement logic to save the current editor buffer to the file
            println!("File saved successfully!");
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
    Ok(())
}

fn main() -> Result<()> {
    let mut editor = DefaultEditor::new()?;
    let mut current_file: Option<String> = None;

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
        }

        let mut line = editor.readline(">> ")?;

        loop {
            if line.is_empty() {
                break;
            }

            if line.starts_with(':') {
                handle_repl_command(line.trim_start_matches(':'))?;
            } else {
                // Add the line to the editor buffer (replace with your in-place editing logic)
                println!("Line entered: {}", line);
            }

            // Prompt for another line if needed
            line = editor.readline(">> ")?;
        }

        // Move cursor back to the beginning of the editor pane for the next iteration
        cursor::MoveUp(height as u16 - 3);
        cursor::MoveToColumn(0);
    }
}
