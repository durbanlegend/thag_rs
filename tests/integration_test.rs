// tests/integration_test.rs

use clap::Parser;
use rs_script::{execute, Cli, DYNAMIC_SUBDIR, TMPDIR};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
