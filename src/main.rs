use std::process::Command;
use std::{fs, io::Write};

use std::io;
use std::path::PathBuf; // Use PathBuf for paths

#[derive(Debug)]
enum BuildRunError {
    Io(io::Error),   // For I/O errors
    Command(String), // For errors during Cargo build or program execution
}

impl From<io::Error> for BuildRunError {
    fn from(err: io::Error) -> Self {
        BuildRunError::Io(err)
    }
}

impl From<String> for BuildRunError {
    fn from(err: String) -> Self {
        BuildRunError::Command(err)
    }
}

impl std::fmt::Display for BuildRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

fn build_and_run_rust(source: &str, cargo_manifest: &str) -> Result<String, BuildRunError> {
    // Define the dedicated directory for the manifest
    let build_dir = PathBuf::from(".cargo/build_run");

    // Ensure the directory exists (create it if needed)
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir)?; // Use fs::create_dir_all for directories
    }

    // Write source code to a file within the directory
    let source_path = build_dir.join("source.rs"); // Join paths with PathBuf
    let mut source_file = fs::File::create(&source_path)?;
    source_file.write_all(source.as_bytes())?;
    eprintln!("Source path: {source_path:?}");
    eprintln!(
        "{:?}",
        fs::read_to_string(PathBuf::from(
            // "/Users/donf/projects/build_run/.cargo/build_run/source.rs"
            "/Users/donf/projects/build_run/.cargo/build_run/source.rs"
        ))
    );

    let relative_path = source_path;
    let mut absolute_path = std::env::current_dir()?;
    absolute_path.push(relative_path);
    eprintln!("Absolute path: {absolute_path:?}");

    // Write Cargo.toml content to a file within the directory
    let cargo_toml_path = build_dir.join("Cargo.toml");
    let mut cargo_toml = fs::File::create(&cargo_toml_path)?;
    cargo_toml.write_all(cargo_manifest.as_bytes())?;
    eprintln!(
        "Cargo.toml contents: {:?}",
        fs::read_to_string(cargo_toml_path)
    );

    // Build the Rust program using Cargo (with manifest path)
    let mut build_command = Command::new("cargo");
    build_command.arg("build").current_dir(build_dir.clone());
    let build_output = build_command.output()?;

    if !build_output.status.success() {
        let error_msg = String::from_utf8_lossy(&build_output.stderr);
        return Err(BuildRunError::Command(format!(
            "Cargo build failed: {error_msg}"
        )));
    }

    // Run the built program (no changes here)
    let mut run_command = Command::new("./target/debug/my_generated_program"); // Replace with actual program name
    run_command.current_dir(build_dir);
    let run_output = run_command.output()?;

    if !run_output.status.success() {
        let error_msg = String::from_utf8_lossy(&run_output.stderr);
        return Err(BuildRunError::Command(format!(
            "Program execution failed: {error_msg}"
        )));
    }

    let output = String::from_utf8_lossy(&run_output.stdout);
    Ok(output.to_string())
}

fn main() {
    // Example source code and Cargo.toml content
    let source = r#"
  fn main() {
    println!("Hello from a generated Rust program!");
  }
  "#;
    let cargo_manifest = r#"
  [package]
  name = "my_generated_program"
  version = "0.0.1"

  [dependencies]
  # Add any dependencies here

  [[bin]]
  name = "my_generated_program"
  path = "/Users/donf/projects/build_run/.cargo/build_run/source.rs"
  "#;

    let result = build_and_run_rust(source, cargo_manifest);

    match result {
        Ok(output) => println!("Program output: {output}"),
        Err(error) => println!("Error: {error}"),
    }
}
