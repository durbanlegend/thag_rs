/*[toml]
[dependencies]
env_logger = "0.11.3"
log = "0.4.21"
*/

/// Run a command (in this case a cargo search for the `log` crate),
/// and capture and print its stdout and stderr concurrently in a
/// separate thread.
//# Purpose: Demo process::Command with output capture.
//# Categories: technique
use env_logger::Builder;
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
    cmd.args(["run", "--", "-cvv", "demo/config.rs"]);

    // Redirect stdout to a pipe
    let mut child = cmd
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("failed to spawn command");

    // Read the captured output from the pipe
    let mut stdout = child.stdout.take().expect("failed to get stdout");
    let mut stderr = child.stderr.take().expect("failed to get stderr");

    let mut stdout_output = String::new();
    stdout
        .read_to_string(&mut stdout_output)
        .expect("failed to read stdout");

    println!("Captured stdout:\n{}", stdout_output);

    let mut stderr_output = String::new();
    stderr
        .read_to_string(&mut stderr_output)
        .expect("failed to read stderr");

    println!("Captured stderr:\n{}", stderr_output);

    // Wait for the child process to finish
    child.wait().expect("failed to wait for child");
}
