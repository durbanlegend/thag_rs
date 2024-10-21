/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["cargo", "derive"] }
clap-repl = "0.1.1"
console = "0.15.8"
rustyline = "14.0.0"
quote = "1.0.36"
syn = { version = "2.0.82", features = ["full"] }
*/

/// Original published example from clap-repl crate, before change
/// from rustyline to reedline.
//# Purpose: Demo building a repl using `clap_repl` with `rustyline`.
use clap::Parser;
use clap_repl::ClapEditor;
use console::style;
use rustyline::DefaultEditor;

#[derive(Debug, Parser)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum SampleCommand {
    Download {
        path: String,
        /// Some explanation about what this flag do.
        #[arg(long)]
        check_sha: bool,
    },
    /// A command to upload things.
    Upload,
    Login {
        /// Optional. You will be prompted if you don't provide it.
        #[arg(short, long)]
        username: Option<String>,
    },
}

fn main() {
    // Use `ClapEditor` instead of the `rustyline::DefaultEditor`.
    let mut rl = ClapEditor::<SampleCommand>::new();
    loop {
        // Use `read_command` instead of `readline`.
        let Some(command) = rl.read_command() else {
            continue;
        };
        match command {
            SampleCommand::Download { path, check_sha } => {
                println!("Downloaded {path} with checking = {check_sha}");
            }
            SampleCommand::Upload => {
                println!("Uploaded");
            }
            SampleCommand::Login { username } => {
                // You can use another `rustyline::Editor` inside the loop.
                let mut rl = DefaultEditor::new().unwrap();
                let username = username.unwrap_or_else(|| {
                    rl.readline(&style("What is your username? ").bold().to_string())
                        .unwrap()
                });
                let password = rl
                    .readline(&style("What is your password? ").bold().to_string())
                    .unwrap();
                println!("Logged in with {username} and {password}");
            }
        }
    }
}
