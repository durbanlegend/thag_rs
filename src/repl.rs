use crate::cmd_args::{Cli, ProcFlags};
use crate::code_utils::{self, clean_up, display_dir_contents, extract_ast, extract_manifest};
use crate::debug_log;
use crate::errors::BuildRunError;
use crate::log;
use crate::logging::Verbosity;
use crate::shared::Ast;
use crate::{
    colors::{nu_resolve_style, MessageLevel},
    gen_build_run, nu_color_println,
    shared::BuildState,
};

use clap::ArgMatches;
use clap::{CommandFactory, Parser};
use lazy_static::lazy_static;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultHinter, DefaultValidator,
    EditCommand, Emacs, FileBackedHistory, KeyCode, KeyModifiers, Keybindings, MenuBuilder, Prompt,
    PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline, ReedlineEvent,
    ReedlineMenu, Signal,
};
use regex::Regex;
use std::borrow::Cow;
use std::error::Error;
use std::str::FromStr;
use std::time::Instant;
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

const HISTORY_FILE: &str = "rs_eval_hist.txt";
pub static DEFAULT_MULTILINE_INDICATOR: &str = "";

#[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
#[command(
    name = "",
    disable_help_flag = true,
    disable_help_subcommand = true,
    verbatim_doc_comment
)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "kebab-case")]
/// REPL mode lets you type or paste a Rust expression to be evaluated.
/// Start by choosing the eval option and entering your expression. Expressions between matching braces,
/// brackets, parens or quotes may span multiple lines.
/// If valid, the expression will be converted into a Rust program, and built and run using Cargo.
/// Dependencies will be inferred from imports if possible using a Cargo search, but the overhead
/// of doing so can be avoided by placing them in Cargo.toml format at the top of the expression in a
/// comment block of the form
/// /*[toml]
/// [dependencies]
/// ...
/// */
/// From here they will be extracted to a dedicated Cargo.toml file.
/// In this case the whole expression must be enclosed in curly braces to include the TOML in the expression.
/// At any stage before exiting the REPL, or at least as long as your TMPDIR is not cleared, you can
/// go back and edit your expression or its generated Cargo.toml file and copy or save them from the
/// editor or directly from their temporary disk locations.
/// The tab key will show command selections and complete partial matching selections."
enum ReplCommand {
    /// Show the REPL banner
    Banner,
    /// Edit the Rust expression. Edit+run can also be used as an alternative to eval for longer snippets and programs.
    Edit,
    /// Edit the generated Cargo.toml
    Toml,
    /// Attempt to build and run the Rust expression
    Run,
    /// Delete all temporary files for this eval (see list)
    Delete,
    /// List temporary files for this eval
    List,
    /// Edit history
    History,
    /// Show help information
    Help,
    /// Exit the REPL
    Quit,
}

impl ReplCommand {
    fn print_help() {
        let mut command = ReplCommand::command();
        let mut buf = Vec::new();
        command.write_help(&mut buf).unwrap();
        let help_message = String::from_utf8(buf).unwrap();
        println!("{}", help_message);
    }
}

#[derive(Debug)]
struct Context<'a> {
    options: &'a mut Cli,
    proc_flags: &'a ProcFlags,
    // cmd_list: String,
    build_state: &'a mut BuildState,
    start: &'a Instant,
}

pub struct ReplPrompt(&'static str);
impl Prompt for ReplPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(self.0.to_string())
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Owned(String::new())
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

fn add_menu_keybindings(keybindings: &mut Keybindings) {
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
}

pub fn run_repl(
    options: &mut Cli,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    start: Instant,
) -> Result<(), Box<dyn Error>> {
    #[allow(unused_variables)]
    let mut context = Context {
        options,
        proc_flags,
        build_state,
        start: &start,
    };
    let context: &mut Context = &mut context;
    let history_file = context.build_state.cargo_home.clone().join(HISTORY_FILE);
    let history = Box::new(
        FileBackedHistory::with_file(25, history_file)
            .expect("Error configuring history with file"),
    );

    let cmd_vec = ReplCommand::iter()
        .map(<ReplCommand as Into<&'static str>>::into)
        .map(String::from)
        .collect::<Vec<String>>();

    let completer = Box::new(DefaultCompleter::new_with_wordlen(cmd_vec.clone(), 2));

    // Use the interactive menu to select options from the completer
    let columnar_menu = ColumnarMenu::default()
        .with_name("completion_menu")
        .with_columns(4)
        .with_column_width(None)
        .with_column_padding(2);

    let completion_menu = Box::new(columnar_menu);

    let mut keybindings = default_emacs_keybindings();
    add_menu_keybindings(&mut keybindings);

    let edit_mode = Box::new(Emacs::new(keybindings));

    // TODO implement a highlighter?
    // let highlighter = Box::<ExampleHighlighter>::default();
    let mut line_editor = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(nu_resolve_style(MessageLevel::Ghost).italic()),
        ))
        .with_history(history)
        // .with_highlighter(highlighter)
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode);

    let prompt = ReplPrompt("repl");

    let cmd_list = cmd_vec.join(", ");

    disp_repl_banner(&cmd_list);
    loop {
        let sig = line_editor.read_line(&prompt)?;
        let input: &str = match sig {
            Signal::Success(ref buffer) => buffer,
            Signal::CtrlD | Signal::CtrlC => {
                break;
            }
        };

        // Process user input (line)

        let rs_source = input.trim();
        if rs_source.is_empty() {
            continue;
        }

        let (first_word, rest) = parse_line(rs_source);
        let maybe_cmd = {
            let mut matches = 0;
            let mut cmd = String::new();
            for key in cmd_vec.iter() {
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
                let args = clap::Command::new("")
                    .no_binary_name(true)
                    .try_get_matches_from_mut(rest)?;
                match repl_command {
                    ReplCommand::Banner => disp_repl_banner(&cmd_list),
                    ReplCommand::Help => {
                        ReplCommand::print_help();
                    }
                    ReplCommand::Quit => {
                        quit(args.clone(), context)?;
                    }
                    ReplCommand::Edit => {
                        edit(args.clone(), context)?;
                    }
                    ReplCommand::Toml => {
                        toml(args.clone(), context)?;
                    }
                    ReplCommand::Run => {
                        run_expr(args.clone(), context)?;
                    }
                    ReplCommand::Delete => {
                        delete(args.clone(), context)?;
                    }
                    ReplCommand::List => {
                        list(args.clone(), context)?;
                    }
                    ReplCommand::History => {
                        edit_history(args.clone(), context)?;
                    }
                }
                continue;
            }
        }

        let rs_manifest = extract_manifest(rs_source, Instant::now())
            .map_err(|_err| BuildRunError::FromStr("Error parsing rs_source".to_string()))?;
        context.build_state.rs_manifest = Some(rs_manifest);

        let maybe_ast = extract_ast(rs_source);

        if let Ok(expr_ast) = maybe_ast {
            code_utils::process_expr(
                &expr_ast,
                context.build_state,
                rs_source,
                context.options,
                context.proc_flags,
                context.start,
            )
            .map_err(|_err| BuildRunError::Command("Error processing expression".to_string()))?;
        } else {
            nu_color_println!(
                nu_resolve_style(MessageLevel::Error),
                "Error parsing code: {maybe_ast:#?}"
            );
        }
    }
    Ok(())
}

/// Delete our temporary files
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn delete(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let build_state = &context.build_state;
    let clean_up = clean_up(&build_state.source_path, &build_state.target_dir_path);
    if clean_up.is_ok()
        || (!&build_state.source_path.exists() && !&build_state.target_dir_path.exists())
    {
        log!(Verbosity::Quiet, "Deleted");
    } else {
        log!(
            Verbosity::Quiet,
            "Failed to delete all files - enter l(ist) to list remaining files"
        );
    }
    Ok(Some(String::from("End of delete")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn edit_history(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let history_file = context.build_state.cargo_home.clone().join(HISTORY_FILE);
    edit::edit_file(history_file)?;
    Ok(Some(String::from("End of history file edit")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn edit(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let (build_state, _start) = (&mut context.build_state, context.start);

    edit::edit_file(&build_state.source_path)?;

    Ok(Some(String::from("End of source edit")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn toml(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    edit::edit_file(&context.build_state.cargo_toml_path)?;
    Ok(Some(String::from("End of Cargo.toml edit")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn run_expr(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let (options, proc_flags, build_state, start) = (
        &mut context.options,
        context.proc_flags,
        &mut context.build_state,
        context.start,
    );

    debug_log!("In run_expr: build_state={build_state:#?}");
    let result = gen_build_run(options, proc_flags, build_state, None::<Ast>, start);
    if result.is_err() {
        log!(Verbosity::Quiet, "{result:?}");
    }
    Ok(Some(String::from("End of run")))
}

// Borrowed from clap-repl crate.
fn parse_line(line: &str) -> (String, Vec<String>) {
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

fn disp_repl_banner(cmd_list: &str) {
    nu_color_println!(
        nu_resolve_style(MessageLevel::OuterPrompt),
        r#"Enter a Rust expression (e.g., 2 + 3 or "Hi!"), or one of: {cmd_list}."#
    );

    nu_color_println!(
        nu_resolve_style(MessageLevel::InnerPrompt),
        r"Expressions in matching braces, brackets or quotes may span multiple lines.
Use ↑ ↓ to navigate history, →  to select current. Ctrl-U: clear. Ctrl-K: delete to end."
    );
}

/// Display file listing

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn list(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let build_state = &context.build_state;
    let source_path = &build_state.source_path;
    if source_path.exists() {
        log!(Verbosity::Quiet, "File: {:?}", &source_path);
    }

    // Display directory contents
    display_dir_contents(&build_state.target_dir_path)?;

    // Check if neither file nor directory exist
    if !&source_path.exists() && !&build_state.target_dir_path.exists() {
        log!(Verbosity::Quiet, "No temporary files found");
    }
    Ok(Some(String::from("End of list")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn quit(_args: ArgMatches, _context: &mut Context) -> Result<Option<String>, BuildRunError> {
    log!(Verbosity::Quiet, "Done");
    std::process::exit(0);
}
