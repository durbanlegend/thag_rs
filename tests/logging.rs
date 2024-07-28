#[cfg(test)]
mod tests {
    use rs_script::debug_log;
    use rs_script::logging::{set_global_verbosity, Logger, Verbosity, LOGGER};
    use sequential_test::{parallel, sequential};
    use std::env;
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::sync::Once;

    // Set environment variables before running tests
    fn set_up()() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
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
    fn test_logger_new() {
        set_up()();
        let logger = Logger::new(Verbosity::Quiet);
        assert_eq!(logger.verbosity, Verbosity::Quiet);

        let logger = Logger::new(Verbosity::Normal);
        assert_eq!(logger.verbosity, Verbosity::Normal);

        let logger = Logger::new(Verbosity::Verbose);
        assert_eq!(logger.verbosity, Verbosity::Verbose);
    }

    #[test]
    #[parallel]
    fn test_logger_set_verbosity() {
        set_up()();
        let mut logger = Logger::new(Verbosity::Quiet);
        assert_eq!(logger.verbosity, Verbosity::Quiet);

        logger.set_verbosity(Verbosity::Verbose);
        assert_eq!(logger.verbosity, Verbosity::Verbose);
    }

    #[test]
    #[sequential]
    fn test_logger_log() {
        init_logger();
        let rs_script_path = env::current_dir().expect("Error getting current directory");

        let input = format!(
            r#"/*[toml]
[dependencies]
rs-script = {{ path = {rs_script_path:#?} }}
*/

use rs_script::log;
use rs_script::logging::Verbosity;

fn main() {{
    log!(Verbosity::Quiet, "Quiet message");
    log!(Verbosity::Normal, "Normal message");
    log!(Verbosity::Verbose, "Verbose message");
}}
"#
        );
        debug_log!("input={input}");

        let output = run(input);

        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "Quiet message\nNormal message\n"
        );
    }

    #[test]
    #[parallel]
    fn test_global_logger() {
        reset_global_logger();

        set_global_verbosity(Verbosity::Verbose);
        {
            let logger = LOGGER.lock().unwrap();
            assert_eq!(logger.verbosity, Verbosity::Verbose);
        }

        set_global_verbosity(Verbosity::Quiet);
        {
            let logger = LOGGER.lock().unwrap();
            assert_eq!(logger.verbosity, Verbosity::Quiet);
        }
    }

    #[test]
    #[sequential]
    fn test_macro_log() {
        set_up()();
        init_logger();
        let rs_script_path = env::current_dir().expect("Error getting current directory");

        let input = format!(
            r#"/*[toml]
[dependencies]
rs-script = {{ path = {rs_script_path:#?} }}
*/

use rs_script::log;
use rs_script::logging::Verbosity;

fn main() {{
    log!(Verbosity::Quiet, "Macro quiet message");
    log!(Verbosity::Normal, "Macro normal message");
    log!(Verbosity::Verbose, "Macro verbose message");
}}
"#
        );
        debug_log!("input={input}");

        let output = run(input);

        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "Macro quiet message\nMacro normal message\n"
        );
    }

    fn run(input: String) -> std::process::Output {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--features=debug-logs")
            .arg("--")
            .arg("-q")
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
        set_global_verbosity(Verbosity::Normal);
        output
    }
}
