#[cfg(test)]
mod tests {
    use sequential_test::{parallel, sequential};
    use simplelog::{
        ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
    };
    use std::{
        env,
        fs::File,
        io::Write,
        process::{Command, Stdio},
        sync::Once,
    };
    use thag_rs::{
        debug_log,
        logging::{set_global_verbosity, Logger, Verbosity, LOGGER},
    };

    // Set environment variables before running tests
    fn set_up() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            env::set_var("TEST_ENV", "1");
            env::set_var("VISUAL", "cat");
            env::set_var("EDITOR", "cat");
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
                LevelFilter::Info,
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
    fn reset_global_logger() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            drop(LOGGER.lock().unwrap());
        });
    }

    #[test]
    #[parallel]
    fn test_logging_logger_new() {
        set_up();
        let logger = Logger::new(Verbosity::Quiet);
        assert_eq!(logger.verbosity, Verbosity::Quiet);

        let logger = Logger::new(Verbosity::Normal);
        assert_eq!(logger.verbosity, Verbosity::Normal);

        let logger = Logger::new(Verbosity::Verbose);
        assert_eq!(logger.verbosity, Verbosity::Verbose);
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
    #[sequential]
    fn test_logging_logger_log() {
        set_up();
        // init_logger();
        let thag_rs_path = env::current_dir().expect("Error getting current directory");

        let input = format!(
            r#"/*[toml]
[dependencies]
thag_rs = {{ path = {thag_rs_path:#?} }}
*/

use thag_rs::vlog;
use thag_rs::logging::Verbosity;

fn main() {{
    vlog!(Verbosity::Quieter, "Quieter message");
    vlog!(Verbosity::Quiet, "Quiet message");
    vlog!(Verbosity::Normal, "Normal message");
    vlog!(Verbosity::Verbose, "Verbose message");
}}
"#
        );
        debug_log!("input={input}");

        let output = run(input);

        assert!(String::from_utf8_lossy(&output.stdout)
            .ends_with("Quieter message\nQuiet message\nNormal message\n"));
    }

    #[test]
    #[parallel]
    fn test_logging_global_logger() {
        set_up();
        reset_global_logger();

        set_global_verbosity(Verbosity::Verbose).expect("Error setting global verbosity");
        {
            let logger = LOGGER.lock().unwrap();
            assert_eq!(logger.verbosity, Verbosity::Verbose);
        }

        set_global_verbosity(Verbosity::Quiet).expect("Error setting global verbosity");
        {
            let logger = LOGGER.lock().unwrap();
            assert_eq!(logger.verbosity, Verbosity::Quiet);
        }
    }

    #[test]
    #[sequential]
    fn test_logging_macro_log() {
        set_up();
        // init_logger();
        let thag_rs_path = env::current_dir().expect("Error getting current directory");

        let input = format!(
            r#"/*[toml]
[dependencies]
thag_rs = {{ path = {thag_rs_path:#?} }}
*/

use thag_rs::vlog;
use thag_rs::logging::Verbosity;

fn main() {{
    vlog!(Verbosity::Quieter, "Macro quieter message");
    vlog!(Verbosity::Quiet, "Macro quiet message");
    vlog!(Verbosity::Normal, "Macro normal message");
    vlog!(Verbosity::Verbose, "Macro verbose message");
}}
"#
        );

        debug_log!("input={input}");

        let output = run(input);

        assert!(String::from_utf8_lossy(&output.stdout)
            .ends_with("Macro quieter message\nMacro quiet message\nMacro normal message\n"));
    }

    fn run(input: String) -> std::process::Output {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--features=debug-logs")
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

        reset_global_logger();
        set_global_verbosity(Verbosity::Normal).expect("Error setting global verbosity");
        output
    }
}
