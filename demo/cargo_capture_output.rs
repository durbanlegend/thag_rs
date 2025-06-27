/// Run a command (in this case a cargo search for the `log` crate),
/// and capture and print its stdout and stderr concurrently in a
/// separate thread.
//# Purpose: Demo process::Command with output capture.
//# Categories: technique
use env_logger::Builder;
use log::debug;
use std::env;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::process::{self, Command};

fn prog() -> Option<String> {
    env::args()
        .next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}

fn main() {
    Builder::new().filter_level(log::LevelFilter::Debug).init();

    eprintln!("Running {:#?}", prog().unwrap());
    // Define the command and arguments
    let mut cmd = Command::new("cargo");
    // cmd.args(["build", "--verbose"]);
    cmd.args(["rustc", "--bin", "thag", "--profile=check"]);

    // Redirect stdout to a pipe
    let mut child = cmd
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("failed to spawn command");

    // Read the captured output from the pipe
    let mut stdout = child.stdout.take().expect("failed to get stdout");
    let mut output = String::new();
    stdout
        .read_to_string(&mut output)
        .expect("failed to read stdout");

    // Print the captured stdout
    debug!("Captured stdout:\n{}", output);

    let mut stderr = child.stderr.take().expect("failed to get stdout");
    stderr
        .read_to_string(&mut output)
        .expect("failed to read stderr");

    // Print the captured stderr
    debug!("Captured stderr:\n{}", output);

    // Wait for the child process to finish
    child.wait().expect("failed to wait for child");
}
