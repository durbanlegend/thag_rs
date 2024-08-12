use std::process::Command;
let output = Command::new("cargo")
    .arg("run")
    .arg("--")
    .arg("-cfgnq")
    .arg("demo/tui_scrollview.rs")
    .output()
    .expect("Failed to execute command");

if !output.status.success() {
    panic!(
        "Failed to build file: tui_scrollview.rs\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
