#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::{
        env,
        io::Write,
        process::{Command, Stdio},
        sync::Once,
    };
    use thag_proc_macros::safe_eprintln;
    use thag_rs::{debug_log, set_global_verbosity, Verbosity, OUTPUT_MANAGER};

    #[cfg(feature = "simplelog")]
    use simplelog::{
        ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
    };

    #[cfg(feature = "simplelog")]
    use std::fs::File;

    // Set environment variables before running tests
    fn set_up() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            unsafe {
                std::env::set_var("TEST_ENV", "1");
                std::env::set_var("VISUAL", "cat");
                std::env::set_var("EDITOR", "cat");
            }
            init_logger();
        });
    }

    #[cfg(feature = "env_logger")]
    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[cfg(not(feature = "env_logger"))]
    fn init_logger() {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create("app.log").unwrap(),
            ),
        ])
        .unwrap();
    }

    // A utility function to reset the global logger for testing.
    fn reset_global_output_manager() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            drop(OUTPUT_MANAGER.lock());
        });
    }

    // #[test]
    // #[parallel]
    // fn test_logging_logger_set_verbosity() {
    //     set_up();
    //     let mut logger = Logger::new(Verbosity::Quiet);
    //     assert_eq!(logger.verbosity, Verbosity::Quiet);

    //     logger.set_verbosity(Verbosity::Verbose);
    //     assert_eq!(logger.verbosity, Verbosity::Verbose);
    // }

    #[test]
    #[serial]
    fn test_logging_logger_log() {
        set_up();
        // init_logger();
        let thag_rs_path = env::current_dir().expect("Error getting current directory");

        let input = format!(
            r#"/*[toml]
[dependencies]
thag_rs = {{ path = {thag_rs_path:#?}, features = ["core", "simplelog"]}}
*/

use thag_rs::vprtln;
use thag_rs::Verbosity;

fn main() {{
    vprtln!(Verbosity::Quieter, "Quieter message");
    vprtln!(Verbosity::Quiet, "Quiet message");
    vprtln!(Verbosity::Normal, "Normal message");
    vprtln!(Verbosity::Verbose, "Verbose message");
}}
"#
        );
        debug_log!("input={input}");

        let output = run(input);
        debug_log!("output={output:?}");

        let out_str = String::from_utf8_lossy(&output.stdout);
        eprintln!("out_str=[{out_str}]");
        assert!(out_str.ends_with("Quieter message\nQuiet message\nNormal message\n"));
    }

    #[test]
    #[serial]
    fn test_logging_macro_log() {
        set_up();
        // init_logger();
        let thag_rs_path = env::current_dir().expect("Error getting current directory");

        let input = format!(
            r#"/*[toml]
[dependencies]
thag_rs = {{ path = {thag_rs_path:#?} }}
*/

use thag_rs::vprtln;
use thag_rs::Verbosity;

fn main() {{
    vprtln!(Verbosity::Quieter, "Macro quieter message");
    vprtln!(Verbosity::Quiet, "Macro quiet message");
    vprtln!(Verbosity::Normal, "Macro normal message");
    vprtln!(Verbosity::Verbose, "Macro verbose message");
}}
"#
        );

        debug_log!("input={input}");

        let output = run(input);
        safe_eprintln!("output={output:?}");

        let result = String::from_utf8_lossy(&output.stdout);
        safe_eprintln!("result={result}");
        assert!(
            result.ends_with("Macro quieter message\nMacro quiet message\nMacro normal message\n")
        );
    }

    #[test]
    #[cfg(feature = "env_logger")]
    #[serial]
    fn test_logging_env_logger() {
        set_up();
        // init_logger();
        let thag_rs_path = env::current_dir().expect("Error getting current directory");

        let input = format!(
            r#"/*[toml]
[dependencies]
thag_rs = {{ path = {thag_rs_path:#?}, default-features = false, features = ["core", "env_logger"]}}
*/

use thag_rs::vprtln;
use thag_rs::Verbosity;

fn main() {{
    vprtln!(Verbosity::Quieter, "Macro quieter message");
    vprtln!(Verbosity::Quiet, "Macro quiet message");
    vprtln!(Verbosity::Normal, "Macro normal message");
    vprtln!(Verbosity::Verbose, "Macro verbose message");
}}
"#
        );

        debug_log!("input={input}");

        let output = run(input);
        safe_eprintln!("output={output:?}");

        assert!(String::from_utf8_lossy(&output.stdout)
            .ends_with("Macro quieter message\nMacro quiet message\nMacro normal message\n"));
    }

    fn run(input: String) -> std::process::Output {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--features=debug_logging")
            .arg("--")
            .arg("-qq")
            .arg("-s")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        reset_global_output_manager();
        set_global_verbosity(Verbosity::Normal);
        output
    }
}
