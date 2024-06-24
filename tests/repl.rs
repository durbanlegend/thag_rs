/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["cargo", "derive"] }
reedline = "0.32.0"
rs-script = { path = "/Users/donf/projects/rs-script" }
*/

use clap::Parser;
use reedline::Prompt;
use reedline::Signal;
use rs_script::repl::{Context, ReplPrompt};
use rs_script::BuildState;
use rs_script::Cli;
use rs_script::ProcFlags;
use std::error::Error;
//  use std::process::{Command, Stdio};
use std::time::Instant;

struct MockReedline {
    inputs: Vec<Signal>,
    index: usize,
}

impl MockReedline {
    fn new(inputs: Vec<Signal>) -> Self {
        MockReedline { inputs, index: 0 }
    }

    fn read_line(&mut self, _: &dyn Prompt) -> Result<Signal, Box<dyn Error>> {
        if self.index < self.inputs.len() {
            let input = self.inputs.remove(self.index);
            Ok(input)
        } else {
            Ok(Signal::CtrlD)
        }
    }
}

// fn init_logger() {
//     let _ = env_logger::builder().is_test(true).try_init();
// }

fn create_mock_context() -> (Box<Cli>, ProcFlags, Box<BuildState>, Instant) {
    let options = Box::new(Cli::parse());
    let proc_flags = ProcFlags::default();
    let build_state = Box::<BuildState>::default();
    let start = Instant::now();

    (options, proc_flags, build_state, start)
}

// #[test]
fn test_repl_banner_command() {
    let (mut options, proc_flags, mut build_state, start) = create_mock_context();
    let _context = Context {
        options: &mut options,
        proc_flags: &proc_flags,
        build_state: &mut build_state,
        start,
    };

    let mut line_editor = MockReedline::new(vec![Signal::Success("banner".to_string())]);
    let prompt = ReplPrompt("repl");

    // Mock the reading line
    while let Ok(sig) = line_editor.read_line(&prompt) {
        match sig {
            Signal::Success(ref buffer) => {
                assert_eq!(buffer, "banner");
            }
            Signal::CtrlD | Signal::CtrlC => {
                break;
            }
        }
    }
}

// #[test]
fn test_repl_help_command() {
    let (mut options, proc_flags, mut build_state, start) = create_mock_context();
    let _context = Context {
        options: &mut options,
        proc_flags: &proc_flags,
        build_state: &mut build_state,
        start,
    };

    let mut line_editor = MockReedline::new(vec![Signal::Success("help".to_string())]);
    let prompt = ReplPrompt("repl");

    // Mock the reading line
    while let Ok(sig) = line_editor.read_line(&prompt) {
        match sig {
            Signal::Success(ref buffer) => {
                assert_eq!(buffer, "help");
            }
            Signal::CtrlD | Signal::CtrlC => {
                break;
            }
        }
    }
}

// #[test]
// fn test_help_response() {
//     init_logger();
//     let string = r#"Enter a Rust expression (e.g., 2 + 3 or "Hi!"), or one of: banner, edit, toml, run, delete, list, history, help, quit.
// Expressions in matching braces, brackets or quotes may span multiple lines.
// Use ↑ ↓ to navigate history, →  to select current. Ctrl-U: clear. Ctrl-K: delete to end.
// [6n"#;

//     let child = Command::new("cargo")
//         .arg("run")
//         // .arg("--features=debug-logs")
//         .arg("--")
//         .arg("-ql")
//         .stdin(Stdio::piped())
//         .stdout(Stdio::piped())
//         .spawn()
//         .expect("Failed to spawn child process");

//     let output = child.wait_with_output().expect("Failed to read stdout");

//     assert_eq!(
//         String::from_utf8_lossy(&output.stdout),
//         format!("{}", string)
//     );
// }

// Add more tests for each command as needed

fn main() {
    test_repl_banner_command();
    test_repl_help_command();
}
