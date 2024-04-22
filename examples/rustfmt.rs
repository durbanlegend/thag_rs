use std::process::Command;

fn main() {
    let source_file = "./examples/repl_000034.rs"; // Replace with your actual file path

    // Check if rustfmt is available
    if Command::new("rustfmt").arg("--version").output().is_ok() {
        // Run rustfmt on the source file
        let mut command = Command::new("rustfmt");
        command.arg(source_file);
        let output = command.output().expect("Failed to run rustfmt");

        if output.status.success() {
            println!("Successfully formatted {} with rustfmt.", source_file);
        } else {
            eprintln!(
                "Failed to format {} with rustfmt:\n{}",
                source_file,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        eprintln!("`rustfmt` not found. Please install it to use this script.");
    }
}
