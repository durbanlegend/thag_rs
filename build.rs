mod build_utils;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use thag_proc_macros::{safe_eprintln, safe_println};

#[allow(clippy::doc_markdown, clippy::too_many_lines)]
/// 1. Compile all built-in themes into the binary.
/// 2. Create a separate test for each individual script in demo/ and src/bin/, to ensure that it builds
///    successfully. We don't try to run them for logistical reasons, but at least we
///    identify undocumented and abandoned scripts. Given that there are so many of these scripts,
///    avoid Cargo's default behaviour of running all tests in parallel.
///
///    NOTE: Tests now use the precompiled `thag` binary (via CARGO_BIN_EXE_thag) instead of
///    `cargo run`, which is much faster and avoids cargo lock contention. You can use
///    `--test-threads=1` for most reliable results, or higher values for faster completion.
///
///    Suggested command: `cargo test --features=simplelog -- --nocapture --test-threads=1`
///    You may want to increase test-threads for faster execution now that lock contention is eliminated.
fn main() {
    // 1. Theme loading
    // NB: Tell cargo to rerun if any theme file changes
    safe_println!("cargo:rerun-if-changed=themes/built_in");

    // if let Err(e) = generate_theme_data() {
    //     // Use cargo:warning to show build script errors
    //     safe_println!("cargo:warning=Theme generation failed: {e:?}"); // Fail the build if we can't generate themes
    //     std::process::exit(1);
    // }

    // 2. Test generation
    // NB: Tell cargo to rerun if any tool file changes
    safe_println!("cargo:rerun-if-changed=src/bin");

    // Check for mutually exclusive features
    let simple = std::env::var("CARGO_FEATURE_SIMPLELOG").is_ok();
    let env = std::env::var("CARGO_FEATURE_ENV_LOGGER").is_ok();

    safe_eprintln!("simple={simple}; env={env}");
    assert!(
        !(simple & env),
        "Features 'simplelog' and 'env_logger' are mutually exclusive.\n\
          Use --no-default-features when enabling env_logger.\n\
          You will then have to explicitly list default features you still need, such as `full` for the bin or `core` for the lib"    );

    // Ensure at least one logger is selected
    assert!(
        !(!simple && !env),
        "One of 'simplelog' or 'env_logger' must be enabled"
    );

    // Get the OUT_DIR environment variable
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    // Note: Cargo suppresses build output. I've tried log and env_logger, ChatGPT, Gemini, Stack Overflow etc.
    // The only way it seems that it will display is looking in a *output file for
    // println! and a *stderr file for eprintln! afterwards. -vv is suggested but
    // doesn't seem to work. `find . -mtime 0 -name "*output" (or "*stderr") -ls`.
    // https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
    safe_eprintln!("OUT_DIR={out_dir}");
    fs::create_dir_all(&out_dir).expect("Failed to create destination directory");
    let out_dir_path = &Path::new(&out_dir);
    let dest_path = out_dir_path.join("generated_tests.rs");
    let mut file = fs::File::create(dest_path).expect("Failed to create generated_tests.rs");

    let subdir_names = vec!["demo"];

    for subdir_name in &subdir_names {
        let source_dir = Path::new(subdir_name);

        safe_eprintln!(
            "source_path = source_dir = {:#?}",
            source_dir.canonicalize()
        );
        assert!(
            source_dir.exists() && source_dir.is_dir(),
            "source directory {} does not exist",
            source_dir.display()
        );

        // Define the source and destination paths
        let dest_dir = &out_dir_path.join(subdir_name);

        // Create the destination directory if it doesn't exist
        fs::create_dir_all(dest_dir)
            .unwrap_or_else(|_| panic!("Failed to create directory {}", dest_dir.display()));

        let skip_scripts_on_windows = [
            "crossbeam_channel_stopwatch.rs",
            "factorial_main_rug.rs",
            "factorial_main_rug_product.rs",
            "fib_4784969_cpp_rug.rs",
            "fib_big_clap_rug.rs",
            "fib_doubling_iterative_purge_rug.rs",
            "fib_fac_rug.rs",
            "fib_matrix_rug.rs",
            "rug_arbitrary_precision_nums.rs",
        ];

        let multimain = [
            "flume_async.rs",
            "flume_async_profile.rs",
            "flume_select.rs",
            "thag_async_benchmark.rs",
        ];

        let stable_only = [
            "duration_main.rs",
            "duration_snippet.rs",
            "displayable_nightly.rs",
            "displayable_nightly1.rs",
        ];

        /*
        let source_stem: &str = source_name
            .strip_suffix(thag_rs::RS_SUFFIX)
            .expect("Problem stripping Rust suffix");
        let target_dir_path = TMPDIR
            .join("thag_rs")
            .join(source_stem)
            .join("target/debug");
        let target_path = #[cfg(windows) {
            target_dir_path.join(source_stem.to_string() + ".exe")
        } #[cfg(Not(windows)) {
            target_dir_path.join(&source_stem)
        };
        */

        for entry in fs::read_dir(source_dir)
            .unwrap_or_else(|_| panic!("Failed to read directory {}", source_dir.display()))
        {
            let entry = entry.expect("Failed to get directory entry");
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let source_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .expect("Failed to get source file name");

                // Skip scripts on Windows
                if cfg!(target_os = "windows") && skip_scripts_on_windows.contains(&source_name) {
                    continue;
                }

                // Skip nightly-only scripts if on stable config
                if cfg!(not(feature = "nightly")) && stable_only.contains(&source_name) {
                    safe_eprintln!("Skipping nightly-only test {source_name}");
                    continue;
                }

                let test_name = source_name.replace('.', "_");

                #[allow(clippy::literal_string_with_formatting_args)]
                writeln!(
                file,
                r#"
#[test]
fn check_{subdir_name}_{test_name}() {{
    {{
        use thag_proc_macros::{{/*safe_eprintln,*/ safe_osc}};
        // Reset terminal state at start
        safe_osc!("\x1B[0m\x1B[?1049l"); // Reset all attributes and exit alternate screen

        set_up();

        use std::process::Command;

        // Use precompiled binary instead of cargo run for much faster tests
        // Construct path to built binary
        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| "target".to_string());
        let profile = std::env::var("PROFILE")
            .unwrap_or_else(|_| "debug".to_string());

        #[cfg(windows)]
        let thag_bin = format!("{{}}/{{}}/thag.exe", target_dir, profile);
        #[cfg(not(windows))]
        let thag_bin = format!("{{}}/{{}}/thag", target_dir, profile);

        let output = Command::new(&thag_bin)
            // Suppress invoking termbg and supports_color on shared terminal.
            // This should already be passed by default after call to set_up(), but just making sure.
            .env("TEST_ENV", "1")
            .arg("-c{more_options}")
            .arg({source_path:?})
            .output()
            .expect("Failed to execute command");
            let err_str = std::str::from_utf8(&output.stderr).expect("Can't parse stderr to &str");
        if !output.status.success() || err_str.contains("Build failed") {{
            panic!(
                "Failed to build file: {source_name}\nstdout: {{}}\nstderr: {{}}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }}
        // safe_eprintln!("{{output:?}}");
        // safe_eprintln!("stdout={{}}", String::from_utf8_lossy(&output.stdout));
        // safe_eprintln!("stderr={{}}", String::from_utf8_lossy(&output.stderr));

        // safe_eprintln!("... finished {source_name}, starting cargo clean");

        // Get the file stem
        let file_stem = {source_name:?}.trim_end_matches(".rs");

        // Construct the destination directory path
        let mut dest_dir = env::temp_dir();
        dest_dir.push("thag_rs");
        dest_dir.push(file_stem);

        // Note: With shared target implementation, per-script target directories no longer exist.
        // Build artifacts are in $TMPDIR/thag_rs_shared_target/ and executables in $TMPDIR/thag_rs_bins/.
        // The old cleanup code that removed $TMPDIR/thag_rs/<script>/target/debug is no longer needed.

        // Reset terminal state after
        safe_osc!("\x1B[0m\x1B[?1049l");
    }}
}}
"#,
                // source_name = &source_name,
                source_path = &path.to_str().expect("Failed to get source path"),
                more_options = if multimain.contains(&source_name) {
                    "mq"
                } else if source_name == "hyper_hello_server.rs" || source_name == "just_a_test_expression.rs" {
                    "v"
                } else {
                    "q"
                }
            )
            .expect("Failed to write test function");
            }
        }
    }
}
