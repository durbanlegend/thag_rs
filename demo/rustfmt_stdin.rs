/// Read Rust source code from stdin and display the output as formatted by `rustfmt`.
//# Purpose: Format arbitrary Rust code. Does no more than `rustfmt --`.
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    // Create a buffer to hold the input source code.
    let mut source_code = String::new();

    // Read the source code from stdin.
    println!("Please enter Rust source code (end with Ctrl+D):");
    io::stdin().read_to_string(&mut source_code)?;

    // Prepare to invoke rustfmt.
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write the source code to the rustfmt process's stdin.
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(source_code.as_bytes())?;
    }

    // Capture the output from rustfmt.
    let output = child.wait_with_output()?;

    // Check if rustfmt was successful.
    if output.status.success() {
        // Print the formatted output.
        let formatted_code = String::from_utf8_lossy(&output.stdout);
        println!("\nFormatted Code:\n{}", formatted_code);
    } else {
        // Print the error if rustfmt failed.
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error formatting code:\n{}", error_message);
    }

    Ok(())
}
