/// Interactively test dependency inferency. This script was arbitrarily copied from
/// `demo/iterf_partial_match.rs`.
//# Purpose: Test thag manifest module's dependency inference.
//# Categories: crates, technique, testing
/// Experiment with matching repid iteration mode commands with a partial match of any length.
//# Purpose: Usability: Accept a command as long as the user has typed in enough characters to identify it uniquely.
//# Categories: crates, technique
use clap::{CommandFactory, Parser};
use console::style;
use rustyline::DefaultEditor;
use std::error::Error;
use std::str::FromStr;
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
#[command(name = "", disable_help_flag = true, disable_help_subcommand = true)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "kebab-case")]
enum LoopCommand {
    /// Evaluate an expression. Enclose complex expressions in braces {}.
    Eval,
    /// Enter, paste or modify your code
    Edit,
    /// Enter, paste or modify the generated Cargo.toml file your code
    Toml,
    /// List generated files
    List,
    /// Delete generated files
    Delete,
    /// Exit
    Quit,
    /// Show help information
    Help,
}

impl LoopCommand {
    fn print_help() {
        let mut command = LoopCommand::command();
        let help_message = command.render_help();
        println!("{help_message}");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rl = DefaultEditor::new().unwrap();

    let cmd_vec = LoopCommand::iter()
        .map(<LoopCommand as Into<&'static str>>::into)
        .map(String::from)
        .collect::<Vec<String>>();
    let cmd_list =
        "Enter full or partial match of one of the following: ".to_owned() + &cmd_vec.join(", ");

    println!("{cmd_list}");
    loop {
        let line = match rl.readline(">> ") {
            Ok(x) => x,
            Err(e) => match e {
                rustyline::error::ReadlineError::Eof
                | rustyline::error::ReadlineError::Interrupted => break,
                rustyline::error::ReadlineError::Signal(_) => continue,
                _ => panic!("Error in read line: {e:?}"),
            },
        };
        if line.trim().is_empty() {
            continue;
        }
        _ = rl.add_history_entry(line.as_str());
        let command = if let Some(split) = shlex::split(&line) {
            // eprintln!("split={split:?}");
            let mut matches = 0;
            let first_word = split[0].as_str();
            let mut cmd = String::new();
            for key in &cmd_vec {
                if key.starts_with(first_word) {
                    matches += 1;
                    // Selects last match
                    if matches == 1 {
                        cmd = key.to_string();
                    }
                    // eprintln!("key={key}, split[0]={}", split[0]);
                }
            }
            if matches == 1 {
                cmd
            } else {
                println!("No single matching key found");
                continue;
            }
        } else {
            println!(
                "{} input was not valid and could not be processed",
                style("error:").red().bold()
            );
            LoopCommand::print_help();
            continue;
        };
        println!(
            "command={command}, matching variant={:#?}",
            LoopCommand::from_str(&command)?
        );
        if command == "help" {
            println!();
            LoopCommand::print_help();
            continue;
        }
    }
    Ok(())
}
