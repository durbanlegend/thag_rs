/*[toml]
[dependencies]
clap = { version = "4.5.4", features = ["derive"] }
crossterm = "*"
reedline = "0.32.0"
shlex = "0.1.1"
*/

use clap::{
    error::{Error, ErrorKind},
    Command, FromArgMatches, Parser, Subcommand as _, ValueEnum,
};
use reedline::{
    ColumnarMenu, DefaultCompleter, DefaultValidator, KeyCode, KeyModifiers, ReedlineEvent,
};

#[derive(Clone, Parser, Debug)]
enum ReplCommand {
    Eval,
    Edit,
    List,
    Delete,
    Quit,
}

// impl FromArgMatches for ReplCommand {
//     fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
//         match matches.subcommand() {
//             Some(("eval", _)) => Ok(Self::Eval),
//             Some(("edit", _)) => Ok(Self::Edit),
//             Some(("list", _)) => Ok(Self::List),
//             Some(("delete", _)) => Ok(Self::Delete),
//             Some(("quit", _)) => Ok(Self::Quit),
//             _ => Err(Error::new(ErrorKind::InvalidSubcommand)),
//         }
//     }

//     fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
//         todo!()
//     }
// }

// impl ValueEnum for ReplCommand {
//     fn from_str(s: &str) -> Option<Self> {
//         match s {
//             "eval" => Some(ReplCommand::Eval),
//             "edit" => Some(ReplCommand::Edit),
//             "list" => Some(ReplCommand::List),
//             "delete" => Some(ReplCommand::Delete),
//             "quit" => Some(ReplCommand::Quit),
//             _ => None,
//         }
//     }
// }

fn main() {
    let mut history = Vec::new();
    // let mut line_editor = reedline::Reedline::create();
    let mut valid_commands: Vec<String> = self
        .commands
        .iter()
        .map(|(_, command)| command.name.clone())
        .collect();
    valid_commands.push("help".to_string());
    let completer = Box::new(ReplCompleter::new(&self.commands));
    let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
    let validator = Box::new(DefaultValidator);
    let mut line_editor = Reedline::create()
        .with_edit_mode(Box::new(Emacs::new(self.keybindings.clone())))
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_highlighter(Box::new(ExampleHighlighter::new(valid_commands.clone())))
        .with_validator(validator)
        .with_partial_completions(self.partial_completions)
        .with_quick_completions(self.quick_completions)
        .with_external_printer(self.external_printer.clone());

    if self.hinter_enabled {
        line_editor = line_editor.with_hinter(Box::new(
            DefaultHinter::default().with_style(self.hinter_style),
        ));
    }

    if let Some(history_path) = &self.history {
        let capacity = self.history_capacity.unwrap();
        let history = FileBackedHistory::with_file(capacity, history_path.to_path_buf()).unwrap();
        line_editor = line_editor.with_history(Box::new(history));
    }

    // Ok(line_editor)

    line_editor.set_prompt(">> ");

    let completer = DefaultCompleter::new().insert(vec![
        "eval".to_string(),
        "edit".to_string(),
        "list".to_string(),
        "delete".to_string(),
        "quit".to_string(),
    ]);
    line_editor.set_completer(Box::new(completer));

    // Add Ctrl-h for help binding
    line_editor.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('h'),
        ReedlineEvent::MenuDown,
    );

    let cli = Command::new("REPL").
        version("0.1.0")
        .author("durbanlegend")
            .about("A simple REPL")
        // (@arg command: ValueEnum::from_bounded::<ReplCommand>(ReplCommand::values()) -h "Command to run")
;
    let cli = ReplCommand::augment_subcommands(cli);
    loop {
        let line = line_editor.read_line().unwrap();
        history.push(line.clone());

        if line.is_empty() {
            continue;
        }

        let mut has_error = false;
        let mut inner_loop = false;

        let matches = cli.parse_from(line.split_whitespace());

        if let Err(err) = matches {
            eprintln!("Error: {}", err);
            has_error = true;
        } else {
            let command = matches
                .get_str("command")
                .unwrap()
                .parse::<ReplCommand>()
                .unwrap();

            match command {
                ReplCommand::Eval => {
                    line_editor.set_prompt(">> (eval) > ".to_string());
                    loop {
                        let inner_line = line_editor.read_line().unwrap();
                        history.push(inner_line.clone());
                        if inner_line.is_empty() {
                            break;
                        } else if inner_line == "back" {
                            line_editor.set_prompt(">> ".to_string());
                            inner_loop = true;
                            break;
                        }
                        // Simulate evaluation here, print instead
                        println!("Eval Result: {}", inner_line);
                    }
                }
                ReplCommand::Edit => println!("You chose the edit function"),
                ReplCommand::List => println!("You chose the list function"),
                ReplCommand::Delete => println!("You chose the delete function"),
                ReplCommand::Quit => break,
            }
        }

        if has_error || inner_loop {
            continue;
        }
    }

    // Print the history at the end
    for line in history {
        println!("{}", line);
    }
}
