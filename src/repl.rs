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
    /// Show key bindings
    Keys,
    /// Exit the REPL
    Quit,
}

impl ReplCommand {
    fn print_help() {
        let mut command = ReplCommand::command();
        // let mut buf = Vec::new();
        // command.write_help(&mut buf).unwrap();
        // let help_message = String::from_utf8(buf).unwrap();
        println!("{}", command.render_long_help());
    }
}

#[derive(Debug)]
pub struct Context<'a> {
    pub options: &'a mut Cli,
    pub proc_flags: &'a ProcFlags,
    pub build_state: &'a mut BuildState,
    pub start: Instant,
}

pub struct ReplPrompt(pub &'static str);
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

// fn get_emacs_keybindings() {
//     println!("\n--Default Keybindings--");
//     for (mode, modifier, code, event) in get_reedline_default_keybindings() {
//         if mode == "emacs" {
//             println!("mode: {mode}, keymodifiers: {modifier}, keycode: {code}, event: {event}");
//         }
//     }
//     println!();
// }

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
        start,
    };
    // get_emacs_keybindings();
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
    // keybindings.add_binding(
    //     KeyModifiers::CONTROL,
    //     KeyCode::Char('b'),
    //     ReedlineEvent::Edit(vec![EditCommand::SwapWords]),
    // );
    add_menu_keybindings(&mut keybindings);
    // println!("{:#?}", keybindings.get_keybindings());

    let edit_mode = Box::new(Emacs::new(keybindings.clone()));

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

    let bindings = keybindings.bindings;

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
                    ReplCommand::Keys => {
                        // Can't extract this to a method because for some reason KeyCmmbination is not exposed.
                        // Collect and format key bindings
                        let mut formatted_bindings = Vec::new();
                        for (key_combination, reedline_event) in &bindings {
                            let key_modifiers = key_combination.modifier;
                            let key_code = key_combination.key_code;
                            let modifier = format_key_modifier(key_modifiers);
                            let key = format_key_code(key_code);
                            let key_desc = format!("{}{}", modifier, key);
                            if let ReedlineEvent::Edit(edit_cmds) = reedline_event {
                                let cmd_desc = format_edit_commands(edit_cmds);
                                formatted_bindings.push((key_desc, cmd_desc));
                            }
                        }

                        // Sort the formatted bindings alphabetically by key combination description
                        formatted_bindings.sort_by(|a, b| a.0.cmp(&b.0));

                        // Determine the length of the longest key description for padding
                        let max_key_len = formatted_bindings
                            .iter()
                            .map(|(key, _)| key.len())
                            .max()
                            .unwrap_or(0);

                        // Print the formatted and sorted key bindings
                        for (key_desc, cmd_desc) in formatted_bindings {
                            println!("{:<width$}    {}", key_desc, cmd_desc, width = max_key_len);
                        }
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
                &context.start,
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

// Helper function to convert KeyModifiers to string
fn format_key_modifier(modifier: KeyModifiers) -> String {
    let mut modifiers = Vec::new();
    if modifier.contains(KeyModifiers::CONTROL) {
        modifiers.push("CONTROL");
    }
    if modifier.contains(KeyModifiers::SHIFT) {
        modifiers.push("SHIFT");
    }
    if modifier.contains(KeyModifiers::ALT) {
        modifiers.push("ALT");
    }
    let mods_str = modifiers.join("+");
    if modifiers.is_empty() {
        mods_str + "-"
    } else {
        mods_str
    }
}

// Helper function to convert KeyCode to string
fn format_key_code(key_code: KeyCode) -> String {
    match key_code {
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::F(num) => format!("F{}", num),
        KeyCode::Char(c) => format!("{}", c.to_uppercase()),
        KeyCode::Null => "Null".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::CapsLock => "CapsLock".to_string(),
        KeyCode::ScrollLock => "ScrollLock".to_string(),
        KeyCode::NumLock => "NumLock".to_string(),
        KeyCode::PrintScreen => "PrintScreen".to_string(),
        KeyCode::Pause => "Pause".to_string(),
        KeyCode::Menu => "Menu".to_string(),
        KeyCode::KeypadBegin => "KeypadBegin".to_string(),
        KeyCode::Media(media) => format!("Media({:?})", media),
        KeyCode::Modifier(modifier) => format!("Modifier({:?})", modifier),
    }
}

// Helper function to format EditCommand and include its doc comments
fn format_edit_commands(edit_cmds: &Vec<EditCommand>) -> String {
    let mut cmd_descriptions = Vec::new();
    for cmd in edit_cmds {
        let cmd_desc = match cmd {
            EditCommand::InsertNewline => {
                "InsertNewline: Inserts the system specific new line character/s".to_string()
            }
            // Add other EditCommand variants and their descriptions here
            _ => format!("{:?}", cmd),
        };
        cmd_descriptions.push(cmd_desc);
    }
    cmd_descriptions.join(", ")
}
/// Delete our temporary files
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn delete(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
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

// #[allow(clippy::needless_pass_by_value)]
// #[allow(clippy::unnecessary_wraps)]
pub fn edit_history(
    _args: ArgMatches,
    context: &mut Context,
) -> Result<Option<String>, BuildRunError> {
    let history_file = context.build_state.cargo_home.clone().join(HISTORY_FILE);
    println!("history_file={history_file:#?}");
    edit::edit_file(history_file)?;
    Ok(Some(String::from("End of history file edit")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn edit(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let (build_state, _start) = (&mut context.build_state, context.start);

    edit::edit_file(&build_state.source_path)?;

    Ok(Some(String::from("End of source edit")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn toml(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    edit::edit_file(&context.build_state.cargo_toml_path)?;
    Ok(Some(String::from("End of Cargo.toml edit")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn run_expr(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let (options, proc_flags, build_state, start) = (
        &mut context.options,
        context.proc_flags,
        &mut context.build_state,
        context.start,
    );

    debug_log!("In run_expr: build_state={build_state:#?}");
    let result = gen_build_run(options, proc_flags, build_state, None::<Ast>, &start);
    if result.is_err() {
        log!(Verbosity::Quiet, "{result:?}");
    }
    Ok(Some(String::from("End of run")))
}

// Borrowed from clap-repl crate.
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

/// Function to display the REPL banner
pub fn disp_repl_banner(cmd_list: &str) {
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
pub fn list(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
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
pub fn quit(_args: ArgMatches, _context: &mut Context) -> Result<Option<String>, BuildRunError> {
    log!(Verbosity::Quiet, "Done");
    std::process::exit(0);
}
