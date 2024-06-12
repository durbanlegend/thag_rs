/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["cargo", "derive"] }
clap-repl = "0.1.1"
console = "0.15.8"
rustyline = { version = "14.0.0", features=["with-file-history", "default"] }
shlex = "1.3.0"
strum = { version = "0.26.2", features = ["derive"] }
*/

use clap::{CommandFactory, Parser};
use console::style;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::error::Error;
use strum::{EnumIter, EnumProperty, FromRepr, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Parser, EnumIter, EnumProperty, FromRepr, IntoStaticStr)]
#[command(name = "", disable_help_flag = true, disable_help_subcommand = true)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "kebab-case")]
enum LoopCommand {
    /// Enter, paste or modify your code and optionally edit your generated Cargo.toml
    Edit,
    /// Delete generated files
    Delete,
    /// Evaluate an expression. Enclose complex expressions in braces {}.
    Eval,
    /// List generated files
    List,
    /// Exit REPL
    Quit,
    /// Show help information
    Help,
}

impl LoopCommand {
    fn print_help() {
        let mut command = LoopCommand::command();
        let mut buf = Vec::new();
        command.write_help(&mut buf).unwrap();
        let help_message = String::from_utf8(buf).unwrap();
        println!("{}", help_message);
    }
}

/// Experiment with matching REPL commands with a partial match of any length.
fn main() -> Result<(), Box<dyn Error>> {
    let hashmap: HashMap<String, LoopCommand> = LoopCommand::iter()
        .map(|v| {
            let d = v as usize;
            let v = LoopCommand::from_repr(d).unwrap();
            let kebab = <LoopCommand as Into<&'static str>>::into(v).to_string(); //.to_case(Case::Kebab);
            (kebab, LoopCommand::from_repr(d).unwrap())
        })
        .collect();

    // println!("{hashmap:#?}");

    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new().unwrap();
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut cmd_vec = LoopCommand::iter()
        .map(<LoopCommand as Into<&'static str>>::into)
        .map(String::from)
        .collect::<Vec<String>>();
    cmd_vec.sort();
    let cmd_list =
        "Enter full or partial match of one of the following: ".to_owned() + &cmd_vec.join(", ");

    println!("{cmd_list}");
    loop {
        let line = match rl.readline(">> ") {
            Ok(x) => x,
            Err(e) => match e {
                rustyline::error::ReadlineError::Eof
                | rustyline::error::ReadlineError::Interrupted => break,
                rustyline::error::ReadlineError::WindowResized => continue,
                _ => panic!("Error in read line: {e:?}"),
            },
        };
        if line.trim().is_empty() {
            continue;
        }
        _ = rl.add_history_entry(line.as_str());
        match shlex::split(&line) {
            Some(split) => {
                // eprintln!("split={split:?}");
                // TODO look up in hashmap keys, which contains the splt
                let mut matches = 0;
                let mut matching_key = String::new();
                for key in hashmap.keys() {
                    if key.starts_with(split[0].as_str()) {
                        matches += 1;
                        // Selects last match
                        if matches == 1 {
                            matching_key = key.to_string();
                        }
                        // eprintln!("key={key}, split[0]={}", split[0]);
                    }
                }
                if matches == 1 {
                    println!(
                        "matching_key={matching_key}, matching variant={:#?}",
                        hashmap.get(&matching_key).unwrap()
                    );
                    if matching_key == "help" {
                        LoopCommand::print_help();
                    }
                } else {
                    println!("No single matching key found");
                    LoopCommand::print_help();
                }
            }
            None => {
                println!(
                    "{} input was not valid and could not be processed",
                    style("error:").red().bold()
                );
                LoopCommand::print_help();
            }
        }
    }
    Ok(())
}
