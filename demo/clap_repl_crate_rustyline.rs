/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["cargo", "derive"] }
clap-repl = "0.1.1"
console = "0.15.8"
rustyline = "14.0.0"
quote = "1.0.36"
syn = { version = "2.0.82", features = ["full"] }
*/

/// Older version of published clap_repl crate example, modified to prototype a
/// (dummy) Rust REPL.
//# Purpose: Yet another REPL demo, this time using `rustyline`.
use clap::Parser;
use clap_repl::ClapEditor;
use console::style;
use quote::quote;
use rustyline::config::Configurer;
use rustyline::DefaultEditor;
use syn::{self, Expr};

#[derive(Debug, Parser)]
// This change didn't do the trick. I want an explanatory prompt up front,
// not expect the user to guess and type "help"".
#[command(name = "", arg_required_else_help(true))] // This name will show up in clap's error messages, so it is important to set it to "".
enum SampleCommand {
    Download {
        path: String,
        /// Some explanation about what this flag do.
        #[arg(long)]
        check_sha: bool,
    },
    /// A command to evaluate a Rust expression.
    Eval,
    /// A command to upload things.
    Upload,
    Login {
        /// Optional. You will be prompted if you don't provide it.
        #[arg(short, long)]
        username: Option<String>,
    },
    Exit,
    Quit,
}

fn main() {
    // Use `ClapEditor` instead of the `rustyline::DefaultEditor`.
    let mut editor = ClapEditor::<SampleCommand>::new();
    loop {
        // Use `read_command` instead of `readline`.
        let Some(command) = editor.read_command() else {
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
            SampleCommand::Exit | SampleCommand::Quit => return,
            SampleCommand::Eval => {
                let mut rl = DefaultEditor::new().unwrap();
                rl.set_auto_add_history(true);
                loop {
                    println!("Enter an expression (e.g., 2 + 3), or q to quit:");

                    let input = rl.readline(">> ").expect("Failed to read input");
                    // Process user input (line)
                    // rl.add_history_entry(&line); // Add current line to history
                    // Parse the expression string into a syntax tree
                    let str = &input.trim();
                    if str.to_lowercase() == "q" {
                        break;
                    }
                    let expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(str);

                    match expr {
                        Ok(expr) => {
                            // Generate Rust code for the expression
                            let rust_code = quote!(println!("result={}", #expr););

                            eprintln!("rust_code={rust_code}");
                        }
                        Err(err) => println!("Error parsing expression: {}", err),
                    }
                }
            }
        }
    }
}
