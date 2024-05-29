// tests/integration_test.rs

use rs_script::{DYNAMIC_SUBDIR, TMPDIR};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
// use tempfile::TempDir; // Assuming your library crate is named rs_script

#[test]
fn test_script_runner_with_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for the test project
    let temp_dir: PathBuf = TMPDIR.join(DYNAMIC_SUBDIR);

    // Create a sample script file with a dependency
    let source_path = temp_dir.join("script.rs");
    let mut script_file = File::create(&source_path)?;
    write!(
        script_file,
        r#"/*[toml]
[dependencies]
nu-ansi-term = "0.50.0"
rs_script = {{ path = "/Users/donf/projects/rs-script" }}
*/
use rs_script::term_colors::{{nu_resolve_style, MessageLevel}};
fn main() {{
    println!("nu_resolve_style(MessageLevel::Emphasis)={{:#?}}", nu_resolve_style(MessageLevel::Emphasis));
}}"#
    )?;

    // Build the command to run your script runner
    let mut cmd = Command::new("target/debug/rs_script"); // Replace with actual path

    // Add the script path as an argument
    cmd.arg(source_path.to_str().unwrap());

    // // Execute the command and capture output (optional)
    // let output = cmd.output()?;

    // Redirect stdout to a pipe
    let mut child = cmd
        .stderr(std::process::Stdio::inherit()) // Inherit stderr
        .stdout(std::process::Stdio::piped()) // Redirect stdout to a pipe
        .arg("--")
        .arg("2>&1") // Combine stdout and stderr
        .spawn()
        .expect("failed to spawn child process");

    // Read the captured output from the pipe
    let mut stdout = child.stdout.take().expect("failed to get stdout");
    let mut buffer = vec![0; 1024]; // Allocate a buffer for reading
    loop {
        let size = match stdout.read(&mut buffer) {
            Ok(0) => break, // End of output reached
            Ok(n) => n,
            Err(err) => {
                eprintln!("Error reading output: {}", err);
                break;
            }
        };

        // Display the read data (combine stdout and stderr)
        print!("{}", String::from_utf8_lossy(&buffer[..size]));
    }

    // Wait for the child process to finish
    let exit_code = child.wait().expect("failed to wait for child");

    // Assert on the output or exit code (replace with your assertions)
    assert!(exit_code.success());

    Ok(())
}
