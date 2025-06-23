use std::env;
use std::fs;
use std::sync::Once;

fn file_stem_from_path_str(file_name: &str) -> String {
    let fname_start = file_name.rfind('/').map_or_else(|| 0, |pos| pos + 1);
    let fname_dot = file_name.rfind('.').unwrap_or_else(|| file_name.len());
    file_name[fname_start..fname_dot].to_string()
}

// Set environment variables before running tests
fn set_up() {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }

    let path_str = args[1].clone();

    // Reset terminal state at start
    print!("\x1B[0m\x1B[?1049l"); // Reset all attributes and exit alternate screen

    set_up();

    use std::process::Command;
    let status = Command::new("cargo")
        // Suppress invoking termbg and supports_color on shared terminal.
        // This should already be passed by default after call to set_up(), but just making sure.
        .env("TEST_ENV", "1")
        .arg("run")
        .arg("--")
        .arg("-c")
        .arg(&path_str)
        .status()
        .expect("Failed to execute command");

    if !status.success() {
        panic!("Failed to build file: {path_str}");
    }

    // eprintln!("... finished {pathstr}, starting cargo clean");

    // Get the file stem
    let file_stem = file_stem_from_path_str(&path_str);

    // Construct the destination directory path
    let mut dest_dir = env::temp_dir();
    dest_dir.push("thag_rs");
    dest_dir.push(file_stem);

    // Cargo clean seems to work but is desperately slow - see rev d65b1aed47527f267fcc88f111bec6164b31c8a0
    // for (commented) code.
    // Seems OK
    let target_dir = &dest_dir.join("target/debug");
    // Delete the destination directory after building the file
    if let Err(e) = fs::remove_dir_all(&target_dir) {
        eprintln!(
            "Failed to remove directory: {}, {e:?}",
            target_dir.display()
        );
    }

    // Reset terminal state after
    print!("\x1B[0m\x1B[?1049l");
}
