use clap::Parser;
use rs_script::{execute, Cli, DYNAMIC_SUBDIR, TMPDIR};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_script_runner_with_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for the test project
    let temp_dir: PathBuf = TMPDIR.join(DYNAMIC_SUBDIR);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp_dir directory");
    // Create a sample script file with a dependency
    let source_path = temp_dir.join("script.rs");
    let mut script_file = File::create(&source_path)?;
    let rs_script_path = env::current_dir()?;
    write!(
        script_file,
        r#"/*[toml]
[dependencies]
nu-ansi-term = "0.50.0"
rs-script = {{ path = {rs_script_path:#?} }}
*/
use rs_script::colors::{{nu_resolve_style, MessageLevel}};
use rs_script::log;
use rs_script::logging::Verbosity;
fn main() {{
    log!(Verbosity::Normal, "nu_resolve_style(MessageLevel::Emphasis)={{:#?}}", nu_resolve_style(MessageLevel::Emphasis));
}}"#
    )?;

    // Simulate command-line arguments
    let args = vec![
        "rs_script", // Typically, this would be the binary name
        source_path.to_str().unwrap(),
        "--",
        "2>&1",
    ];

    // Save the real command-line arguments and replace them with the test ones
    let real_args: Vec<String> = env::args().collect();
    env::set_var("RUST_TEST_ARGS", real_args.join(" "));

    // Set up clap to use the test arguments
    let cli = Cli::parse_from(&args);

    println!("cli={:#?}", cli);
    // rs_script::Cli = cli;

    // Call the execute function directly
    execute(cli)?;

    // Restore the real command-line arguments
    env::set_var("RUST_TEST_ARGS", real_args.join(" "));

    Ok(())
}

// #[test]
// fn test_demo_files_build() {
//     let demo_dir = Path::new("demo");
//     assert!(
//         demo_dir.exists() && demo_dir.is_dir(),
//         "demo directory does not exist"
//     );

//     // Read all files in the demo directory
//     for entry in fs::read_dir(demo_dir).expect("Failed to read demo directory") {
//         let entry = entry.expect("Failed to get directory entry");
//         let path = entry.path();

//         // Filter .rs files
//         if path.extension().and_then(|s| s.to_str()) == Some("rs") {
//             let file_name = path
//                 .file_name()
//                 .and_then(|s| s.to_str())
//                 .expect("Failed to get file name");
//             let file_path = path.to_str().expect("Failed to get file path");

//             println!("Building file: {}", file_name);

//             // Execute the command for each .rs file
//             let output = Command::new("cargo")
//                 .arg("run")
//                 .arg("--")
//                 .arg("-bgnq")
//                 .arg(file_path)
//                 .output()
//                 .expect("Failed to execute command");

//             // Check if the command was successful
//             if !output.status.success() {
//                 panic!(
//                     "Failed to build file: {}\nstdout: {}\nstderr: {}",
//                     file_name,
//                     String::from_utf8_lossy(&output.stdout),
//                     String::from_utf8_lossy(&output.stderr)
//                 );
//             }
//         }
//     }
// }

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
