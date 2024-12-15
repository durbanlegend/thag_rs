/*[toml]
[dependencies]
clap = { version = "4.5.21", features = ["cargo", "derive"] }
lazy_static = "1.5.0"
regex = "1.10.6"
repl-block = "0.10.0"
strum = { version = "0.26.3", features = ["derive", "phf"] }
*/
/// Early proof of concept of using a different line editor for repl.rs.
//# Purpose: Exploration
//# Categories: crates, REPL, technique
use clap::{CommandFactory, Parser};
use lazy_static::lazy_static;
use regex::Regex;
// use repl_block::prelude::ReplBlockError;
use repl_block::prelude::{ReplBlockResult, ReplBuilder, Utf8PathBuf};
use std::env;
use std::str::FromStr;
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
#[command(
    name = "",
    disable_help_flag = true,
    disable_help_subcommand = true,
    verbatim_doc_comment
)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "kebab-case")]
enum ReplCommand {
    // Show the REPL banner
    Banner,
    // Edit the Rust expression. Edit+run can also be used as an alternative to eval for longer snippets and programs.
    Edit,
    // Edit the generated Cargo.toml
    Toml,
    // Attempt to build and run the Rust expression
    Run,
    // Delete all temporary files for this eval (see list)
    Delete,
    // List temporary files for this eval
    List,
    // Edit history
    History,
    // Show help information
    Help,
    // Show key bindings
    Keys,
}

impl ReplCommand {
    fn print_help() {
        let mut command = Self::command();
        // let mut buf = Vec::new();
        // command.write_help(&mut buf).unwrap();
        // let help_message = String::from_utf8(buf).unwrap();
        println!("{}", command.render_long_help())
    }
}

struct Evaluator;

impl Evaluator {
    fn evaluate(&self, query: &str) -> ReplBlockResult<&str> {
        // Ok("Hello world!")

        let cmd_vec = ReplCommand::iter()
            .map(<ReplCommand as Into<&'static str>>::into)
            .map(String::from)
            .collect::<Vec<String>>();

        let cmd_list = &cmd_vec.join(", ");

        let rs_source = query.trim();
        if rs_source.is_empty() {
            return Ok("");
        }

        let (first_word, rest) = parse_line(rs_source);
        let maybe_cmd = {
            let mut matches = 0;
            let mut cmd = String::new();
            for key in &cmd_vec {
                if key.starts_with(&first_word) {
                    matches += 1;
                    // Selects last match
                    if matches == 1 {
                        cmd = key.to_string();
                    }
                    // eprintln!("key={key}, split[0]={}", split[0]);
                }
            }
            if matches == 1 {
                Some(cmd)
            } else {
                // println!("No single matching key found");
                None
            }
        };

        if let Some(cmd) = maybe_cmd {
            if let Ok(repl_command) = ReplCommand::from_str(&cmd) {
                let _args = clap::Command::new("")
                    .no_binary_name(true)
                    .try_get_matches_from_mut(rest)
                    .or_else(|_| Err("clap error"));
                match repl_command {
                    ReplCommand::Banner => {
                        disp_repl_banner(cmd_list);
                        return Ok("");
                    }
                    ReplCommand::Help => {
                        // ReplCommand::print_help();
                        ReplCommand::print_help();
                        return Ok("");
                    }
                    ReplCommand::Edit => {
                        // edit(&args, context);
                        return Ok("Placeholder for edit(&args, context)");
                    }
                    ReplCommand::Toml => {
                        // toml(&args, context)?;
                        return Ok("Placeholder for toml(&args, context)");
                    }
                    ReplCommand::Run => {
                        // &history.sync();
                        // run_expr(&args, context)?;
                        return Ok("Placeholder for run_expr(&args, context)");
                    }
                    ReplCommand::Delete => {
                        // delete(&args, context)?;
                        return Ok("Placeholder for delete(&args, context)");
                    }
                    ReplCommand::List => {
                        // list(&args, context)?;
                        return Ok("Placeholder for kist(&args, context)");
                    }
                    ReplCommand::History => {
                        // edit_history(&args, context)?;
                        return Ok("Placeholder for edit_history(&args, context)");
                    }
                    _ => return Ok("nada"),
                }
            }
        }
        return Ok("Not implemented");
    }
}

fn main() -> ReplBlockResult<()> {
    let evaluator = Evaluator {};
    // |query: &str| -> ReplBlockResult<&str> { Ok("Hello world!") };
    let path = Utf8PathBuf::try_from(env::current_dir()?)?.join(".repl.history");
    ReplBuilder::default()
        // Explicitly register .repl.history as the history file:
        .history_filepath(path)
        // Register the evaluator; the default evaluator fn is NOP
        .evaluator(|query: &str| {
            match evaluator.evaluate(query) {
                Ok(value) => println!("{value}"),
                Err(err) => println!("{err}"),
            }
            Ok(())
        })
        .build()? // Use `self` to build a REPL
        .start()?;
    Ok(())
}

// Parse the current line. Borrowed from clap-repl crate.
#[must_use]
pub fn parse_line(line: &str) -> (String, Vec<String>) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"("[^"\n]+"|[\S]+)"#).unwrap();
    }
    let mut args = RE
        .captures_iter(line)
        .map(|a| a[0].to_string().replace('\"', ""))
        .collect::<Vec<String>>();
    let command: String = args.drain(..1).collect();
    (command, args)
}

// Display the REPL banner.
pub fn disp_repl_banner(cmd_list: &str) {
    println!(r#"Enter a Rust expression (e.g., 2 + 3 or "Hi!"), or one of: {cmd_list}."#);

    println!(
        r"Expressions in matching braces, brackets or quotes may span multiple lines.
Use ↑ ↓ to navigate history, →  to select current. Ctrl-U: clear. Ctrl-K: delete to end."
    );
}
