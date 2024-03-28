use env_logger::Builder;
use log::debug;
use std::io::Read;
use std::process::{self, Command};

fn main() {
    Builder::new().filter_level(log::LevelFilter::Debug).init();

    // Define the command and arguments
    let mut cmd = Command::new("ls");
    cmd.arg("-l"); // Add argument for long listing

    // Redirect stdout to a pipe
    let mut child = cmd
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to spawn command");

    // Read the captured output from the pipe
    let mut stdout = child.stdout.take().expect("failed to get stdout");
    let mut output = String::new();
    stdout
        .read_to_string(&mut output)
        .expect("failed to read stdout");

    // Print the captured output
    debug!("Captured output:\n{}", output);

    // Wait for the child process to finish
    child.wait().expect("failed to wait for child");
}
