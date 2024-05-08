/*[toml]
[dependencies]
# nu-ansi-term = { version = "0.50.0", features = ["derive_serde_style"] }
nu-ansi-term = "0.50.0"
reedline = "0.32.0"
reedline-repl-rs = "1.1.1"
*/

use nu_ansi_term::{Color, Style};
use reedline::{
    DefaultHinter, DefaultValidator, FileBackedHistory, Prompt, PromptEditMode,
    PromptHistorySearch, PromptHistorySearchStatus, Reedline, Signal,
};
use reedline_repl_rs::clap::{Arg, ArgAction, ArgMatches, Command};
use reedline_repl_rs::{paint_green_bold, Repl, Result};
use std::borrow::Cow;
use std::collections::VecDeque;

pub struct CustomPrompt(&'static str);
pub static DEFAULT_MULTILINE_INDICATOR: &str = "";
impl Prompt for CustomPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        {
            Cow::Owned(self.0.to_string())
        }
    }

    fn render_prompt_right(&self) -> Cow<str> {
        {
            Cow::Owned(String::from("q: quit"))
        }
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        Cow::Owned("> ".to_string())
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

#[derive(Default)]
struct Context {
    list: VecDeque<String>,
}

fn main() -> Result<()> {
    let mut repl = Repl::new(Context::default())
        .with_name("REPL")
        .with_version("v0.1.0")
        .with_description("REPL mode")
        .with_banner(&format!(
            "{}",
            nu_ansi_term::Color::Cyan
                .bold()
                .paint("Enter one of: continue, delete, eval, list, quit or help"),
        ))
        .with_command(
            Command::new("choose")
                .subcommand(Command::new("eval"))
                .subcommand(Command::new("quit")),
            choose,
        )
        .with_on_after_command(|context| Ok(Some(String::from(">>"))));
    repl.run()
}

fn choose(args: ArgMatches, _ccontext: &mut Context) -> Result<Option<String>> {
    match args.subcommand() {
        Some(("eval", _)) => {
            // let history_file = "cr_eval_hist.txt";
            // let history = Box::new(
            //     FileBackedHistory::with_file(20, history_file)
            //         .expect("Error configuring history with file"),
            // );

            let mut line_editor = Reedline::create()
            .with_validator(Box::new(DefaultValidator))
            .with_hinter(Box::new(
                DefaultHinter::default()
                    .with_style(Style::new().italic().fg(Color::Cyan)),
            ))
            // .with_history(history)
            ;

            let prompt = CustomPrompt("expr");
            loop {
                print!(
                    "{}",
                    nu_ansi_term::Color::Cyan.paint(
                        r"Enter an expression (e.g., 2 + 3), or q to quit.
Expressions in matching braces, brackets or quotes may span multiple lines."
                    )
                );

                let sig = line_editor.read_line(&prompt).expect("Error reading line");
                let input: &str = match sig {
                    Signal::Success(ref buffer) => buffer,
                    Signal::CtrlD | Signal::CtrlC => {
                        println!("\nAborted!");
                        break;
                    }
                };
                // Process user input (line)

                let str = input.trim();
                if str.to_lowercase() == "q" {
                    // outer_prompt();
                    break;
                }
            }

            // Ok(Some("q".to_string()))
        }
        Some(("quit", _)) => return Ok(None),
        _ => panic!("Unknown subcommand {:?}", args.subcommand_name()),
    }
    Ok(Some("Done".to_string()))
}
