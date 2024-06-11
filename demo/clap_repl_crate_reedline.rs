/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["cargo", "derive"] }
clap-repl = "0.2.0"

[dependencies.nu-ansi-term]
version = "0.50.0"

[dependencies.reedline]
version = "0.32.0"
*/

// REPL based on clap-repl crate
use clap::{Parser, ValueEnum};
use clap_repl::ClapEditor;
use reedline::{DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Reedline, Signal};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum SampleCommand {
    Download {
        path: PathBuf,
        /// Check the integrity of the downloaded object
        ///
        /// Uses SHA256
        #[arg(long)]
        check_sha: bool,
    },
    /// A command to upload things.
    Upload,
    /// Login into the system.
    Login {
        /// Optional. You will be prompted if you don't provide it.
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long, value_enum, default_value_t = Mode::Secure)]
        mode: Mode,
    },
    Exit,
    Quit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Encrypt the password
    Secure,
    /// Send the password plain
    ///
    /// This paragraph is ignored because there is no long help text for possible values in clap.
    Insecure,
}

fn main() {
    let mut prompt = DefaultPrompt::default();
    prompt.left_prompt = DefaultPromptSegment::Basic("simple-example".to_owned());
    let mut rl = ClapEditor::<SampleCommand>::new_with_prompt(Box::new(prompt), |reed| {
        // Do custom things with `Reedline` instance here
        reed.with_history(Box::new(
            FileBackedHistory::with_file(10000, "/tmp/clap-repl-simple-example-history".into())
                .unwrap(),
        ))
    });
    loop {
        // Use `read_command` instead of `readline`.
        let Some(command) = rl.read_command() else {
            continue;
        };
        match command {
            SampleCommand::Download { path, check_sha } => {
                println!("Downloaded {path:?} with checking = {check_sha}");
            }
            SampleCommand::Upload => {
                println!("Uploaded");
            }
            SampleCommand::Login { username, mode } => {
                // You can use another `reedline::Reedline` inside the loop.
                let mut rl = Reedline::create();
                let username = username
                    .unwrap_or_else(|| read_line_with_reedline(&mut rl, "What is your username? "));
                let password = read_line_with_reedline(&mut rl, "What is your password? ");
                println!("Logged in with {username} and {password} in mode {mode:?}");
            }
            SampleCommand::Exit | SampleCommand::Quit => return,
        }
    }
}

fn read_line_with_reedline(rl: &mut Reedline, prompt: &str) -> String {
    let Signal::Success(x) = rl
        .read_line(&DefaultPrompt::new(
            DefaultPromptSegment::Basic(prompt.to_owned()),
            DefaultPromptSegment::Empty,
        ))
        .unwrap()
    else {
        panic!();
    };
    x
}
