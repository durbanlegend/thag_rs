use clap::Parser;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use thag::{execute, Cli, DYNAMIC_SUBDIR, TMPDIR};

// Set environment variables before running tests
fn set_up() {
    std::env::set_var("TEST_ENV", "1");
    std::env::set_var("VISUAL", "cat");
    std::env::set_var("EDITOR", "cat");
}

#[test]
fn test_script_runner_with_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    set_up();
    // Create a temporary directory for the test project
    let temp_dir: PathBuf = TMPDIR.join(DYNAMIC_SUBDIR);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp_dir directory");
    // Create a sample script file with a dependency
    let source_path = temp_dir.join("script.rs");
    let mut script_file = File::create(&source_path)?;
    let thag_path = env::current_dir()?;
    write!(
        script_file,
        r#"/*[toml]
[dependencies]
nu-ansi-term = "0.50.0"
thag = {{ path = {thag_path:#?} }}
*/
use nu_ansi_term::Style;
use thag::colors::Lvl;
use thag::vlog;
use thag::logging::Verbosity;
fn main() {{
    vlog!(Verbosity::Normal, "Style::from(&Lvl::EMPH)={{:#?}}", Style::from(&Lvl::EMPH));
}}"#
    )?;

    // Simulate command-line arguments
    let args = vec![
        "thag", // Typically, this would be the binary name
        source_path.to_str().unwrap(),
        "--",
        "2>&1",
    ];

    // Save the real command-line arguments and replace them with the test ones
    let real_args: Vec<String> = env::args().collect();
    env::set_var("RUST_TEST_ARGS", real_args.join(" "));

    // Set up clap to use the test arguments
    let mut cli = Cli::parse_from(&args);

    println!("cli={:#?}", cli);
    // thag::Cli = cli;

    // Call the execute function directly
    execute(&mut cli)?;

    // Restore the real command-line arguments
    env::set_var("RUST_TEST_ARGS", real_args.join(" "));

    Ok(())
}

// Include tests to ensure that every single script in the demo directory will
// compile (not run, since we would have to pass many of them different arguments).
// These tests are built by thag/build.rs.
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
