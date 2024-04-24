/*[toml]
[dependencies]
clap = { version = "4.5.4", features = ["cargo", "derive"] }
clap-repl = "0.1.1"
console = "0.15.8"
convert_case = "0.6.0"
rustyline = { version = "14.0.0", features=["with-file-history", "default"] }
shlex = "1.3.0"
strum = { version = "0.26.2", features = ["derive"] }
*/

// TODO debug history writing.
use clap::Parser;
use console::style;
use convert_case::{Case, Casing};
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::error::Error;
use strum::{EnumIter, EnumProperty, FromRepr, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Parser, EnumIter, EnumProperty, FromRepr, IntoStaticStr)]
#[command(name = "", arg_required_else_help(true))] // This name will show up in clap's error messages, so it is important to set it to "".
enum LoopCommand {
    /// Enter, paste or modify your code and optionally edit your generated Cargo.toml
    Continue,
    #[clap(visible_alias = "c")]
    /// Delete generated files
    #[clap(visible_alias = "d")]
    Delete,
    /// Evaluate an expression. Enclose complex expressions in braces {}.
    #[clap(visible_alias = "e")]
    Eval,
    /// List generated files
    #[clap(visible_alias = "l")]
    List,
    /// Exit REPL
    #[clap(visible_alias = "q")]
    Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
    let hashmap: HashMap<String, LoopCommand> = LoopCommand::iter()
        .map(|v| {
            let d = v as usize;
            let v = LoopCommand::from_repr(d).unwrap();
            let kebab = <LoopCommand as Into<&'static str>>::into(v).to_case(Case::Kebab);
            (kebab, LoopCommand::from_repr(d).unwrap())
        })
        .collect();

    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new().unwrap();
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

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
                        if matches == 1 {
                            matching_key = String::new();
                            break;
                        }
                        // eprintln!("key={key}, split[0]={}", split[0]);
                        matches += 1;
                        matching_key = key.to_string();
                    }
                }
                if matches == 1 {
                    println!(
                        "matching_key={matching_key}, matching variant={:#?}",
                        hashmap.get(&matching_key).unwrap()
                    );
                }
            }
            None => {
                println!(
                    "{} input was not valid and could not be processed",
                    style("error:").red().bold()
                );
            }
        }
    }
    Ok(())
}
