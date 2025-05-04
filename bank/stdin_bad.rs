/*[toml]
[dependencies]
clap = "4.5.18"
crokey = "1.1.0"
crossterm = "0.29"
edit = "0.1.5"
lazy_static = "1.4.0"
mockall = "0.13.0"
ratatui = "0.29"
reedline = "0.36.0"
regex = "1.10.4"
scopeguard = "1.2.0"
serde = "1.0.219"
serde_json = "1.0.132"
strum = "0.26.3"
thag_rs = { path = "/Users/donf/projects/thag_rs/" }
tui-textarea = { version = "0.6", features = ["search"] }
*/

#![allow(clippy::uninlined_format_args)]
use thag_rs::code_utils::process_expr;
use thag_rs::colors::TUI_SELECTION_BG;
use thag_rs::errors::ThagResult;
use thag_rs::logging::Verbosity;
use thag_rs::repl::{
    add_menu_keybindings, disp_repl_banner, format_edit_commands, format_key_code,
    format_key_modifier, format_non_edit_events, parse_line, show_key_bindings, ReplPrompt,
};
use thag_rs::shared::KeyDisplayLine;
use thag_rs::tui_editor::{
    apply_highlights, normalize_newlines, reset_term, show_popup, CrosstermEventReader, Display,
    EditData, EventReader, History, MAPPINGS, TITLE_BOTTOM, TITLE_TOP,
};
use thag_rs::{
    extract_ast_expr, extract_manifest, log, nu_color_println, nu_resolve_style, BuildState, Cli,
    MessageLevel, ProcFlags,
};

use clap::{CommandFactory, Parser};
use crokey::{crossterm, key, KeyCombinationFormat};
use crossterm::event::{
    EnableBracketedPaste,
    EnableMouseCapture,
    Event::{self, Paste},
    // KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use edit::edit_file;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{block::Block, Borders};
use ratatui::Terminal;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultHinter, DefaultValidator,
    Emacs, MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use std::fs::OpenOptions;
use std::io::{self, BufRead, IsTerminal};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use strum::EnumIter;
use strum::{EnumString, IntoEnumIterator, IntoStaticStr};
use tui_textarea::{CursorMove, Input, TextArea};

/// A struct to allow sharing of necessary context between functions
#[derive(Debug)]
pub struct Context<'a> {
    pub args: &'a mut Cli,
    pub proc_flags: &'a ProcFlags,
    pub build_state: &'a mut BuildState,
    pub start: Instant,
}

#[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
#[command(
    name = "",
    disable_help_flag = true,
    disable_help_subcommand = true,
    verbatim_doc_comment
)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "snake_case")]
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
    /// Edit the Rust expression.
    Edit,
    /// Edit the generated Cargo.toml
    Toml,
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
        let mut command = Self::command();
        println!("{}", command.render_long_help());
    }
}

#[allow(dead_code)]
fn main() -> ThagResult<()> {
    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        log!(Verbosity::Normal, "{line}");
    }
    Ok(())
}

/// Run the REPL.
/// # Errors
/// Will return `Err` if there is any error in running the REPL.
/// # Panics
/// Will panic if there is a problem configuring the `reedline` history file.
#[allow(clippy::module_name_repetitions)]
#[allow(clippy::too_many_lines)]
pub fn run_repl(
    args: &Cli,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    start: Instant,
) -> ThagResult<()> {
    #[allow(unused_variables)]
    // let mut context = Context {
    //     args,
    //     proc_flags,
    //     build_state,
    //     start,
    // };
    // // get_emacs_keybindings();
    // let context: &mut Context = &mut context;
    // // let history_file = context.build_state.cargo_home.join(HISTORY_FILE);
    // // let history = Box::new(FileBackedHistory::with_file(25, history_file)?);
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
    // println!("{:#?}", keybindings.get_keybindings());

    let edit_mode = Box::new(Emacs::new(keybindings.clone()));

    // let highlighter = Box::<ExampleHighlighter>::default();
    let mut line_editor = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(nu_resolve_style(MessageLevel::Ghost).italic()),
        ))
        // .with_history(history)
        // .with_highlighter(highlighter)
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode);

    let bindings = keybindings.get_keybindings();

    let prompt = ReplPrompt("repl");
    let cmd_list = &cmd_vec.join(", ");

    disp_repl_banner(cmd_list);
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

        let (first_word, _rest) = parse_line(rs_source);
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
                // let args = clap::Command::new("")
                //     .no_binary_name(true)
                //     .try_get_matches_from_mut(rest)?;
                match repl_command {
                    ReplCommand::Banner => disp_repl_banner(cmd_list),
                    ReplCommand::Help => {
                        ReplCommand::print_help();
                    }
                    ReplCommand::Quit => {
                        break;
                    }
                    ReplCommand::Edit => {
                        let event_reader = CrosstermEventReader;
                        eval(&event_reader, build_state, args, proc_flags)?;
                    }
                    ReplCommand::Toml => {
                        toml(&build_state.cargo_toml_path)?;
                    }
                    ReplCommand::History => {
                        edit_history()?;
                    }
                    ReplCommand::Keys => {
                        // Calculate max command len for padding
                        // Can't extract this to a method because for some reason KeyCmmbination is not exposed.
                        let max_cmd_len = {
                            // Determine the length of the longest command for padding
                            let max_cmd_len = bindings
                                .values()
                                .map(|reedline_event| {
                                    if let ReedlineEvent::Edit(edit_cmds) = reedline_event {
                                        edit_cmds
                                            .iter()
                                            .map(|cmd| {
                                                let key_desc =
                                                    nu_resolve_style(MessageLevel::Subheading)
                                                        .paint(format!("{cmd:?}"));
                                                let key_desc = format!("{key_desc}");
                                                key_desc.len()
                                            })
                                            .max()
                                            .unwrap_or(0)
                                    } else if !format!("{reedline_event}").starts_with("UntilFound")
                                    {
                                        let event_desc = nu_resolve_style(MessageLevel::Subheading)
                                            .paint(format!("{reedline_event:?}"));
                                        let event_desc = format!("{event_desc}");
                                        event_desc.len()
                                    } else {
                                        0
                                    }
                                })
                                .max()
                                .unwrap_or(0);
                            // Add 2 bytes of padding
                            max_cmd_len + 2
                        };

                        // Collect and format key bindings
                        // Can't extract this to a method either, because KeyCmmbination is not exposed.
                        let mut formatted_bindings = {
                            let mut formatted_bindings = Vec::new();
                            for (key_combination, reedline_event) in bindings {
                                let key_modifiers = key_combination.modifier;
                                let key_code = key_combination.key_code;
                                let modifier = format_key_modifier(key_modifiers);
                                let key = format_key_code(key_code);
                                let key_desc = format!("{}{}", modifier, key);
                                if let ReedlineEvent::Edit(edit_cmds) = reedline_event {
                                    let cmd_desc = format_edit_commands(edit_cmds, max_cmd_len);
                                    formatted_bindings.push((key_desc.clone(), cmd_desc));
                                } else {
                                    let event_name = format!("{reedline_event:?}");
                                    if !event_name.starts_with("UntilFound") {
                                        let event_desc =
                                            format_non_edit_events(&event_name, max_cmd_len);
                                        formatted_bindings.push((key_desc, event_desc));
                                    }
                                }
                            }
                            formatted_bindings
                        };

                        // Sort the formatted bindings alphabetically by key combination description
                        formatted_bindings.sort_by(|a, b| a.0.cmp(&b.0));

                        // Determine the length of the longest key description for padding
                        let max_key_len = formatted_bindings
                            .iter()
                            .map(|(key_desc, _)| {
                                let key_desc =
                                    nu_resolve_style(MessageLevel::Heading).paint(key_desc);
                                let key_desc = format!("{key_desc}");
                                key_desc.len()
                            })
                            .max()
                            .unwrap_or(0);
                        // eprintln!("max_key_len={max_key_len}");

                        show_key_bindings(&formatted_bindings, max_key_len);
                    }
                }
                continue;
            }
        }

        let rs_manifest = extract_manifest(rs_source, Instant::now())?;
        build_state.rs_manifest = Some(rs_manifest);

        let maybe_ast = extract_ast_expr(rs_source);

        if let Ok(expr_ast) = maybe_ast {
            process_expr(expr_ast, build_state, rs_source, args, proc_flags, &start)?;
        } else {
            nu_color_println!(
                nu_resolve_style(MessageLevel::Error),
                "Error parsing code: {maybe_ast:#?}"
            );
        }
    }
    Ok(())
}

fn eval(
    event_reader: &CrosstermEventReader,
    build_state: &mut BuildState,
    args: &Cli,
    proc_flags: &ProcFlags,
) -> ThagResult<()> {
    let vec = edit(event_reader)?;
    let start = Instant::now();
    let input = vec.join("\n");
    let rs_source = input.trim();
    let rs_manifest = extract_manifest(rs_source, Instant::now())?;
    build_state.rs_manifest = Some(rs_manifest);
    let maybe_ast = extract_ast_expr(rs_source);
    if let Ok(expr_ast) = maybe_ast {
        process_expr(expr_ast, build_state, rs_source, args, proc_flags, &start)?;
    } else {
        nu_color_println!(
            nu_resolve_style(MessageLevel::Error),
            "Error parsing code: {maybe_ast:#?}"
        );
    };
    Ok(())
}

/// Open the generated Cargo.toml file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
#[allow(clippy::unnecessary_wraps)]
pub fn toml(cargo_toml_file: &PathBuf) -> ThagResult<Option<String>> {
    if cargo_toml_file.exists() {
        edit_file(cargo_toml_file)?;
    } else {
        log!(
            Verbosity::Quieter,
            "No Cargo.toml file found - have you run anything?"
        );
    }
    Ok(Some(String::from("End of Cargo.toml edit")))
}

/// Edit the stdin stream.
///
///
/// # Examples
///
/// ```no_run
/// use thag_rs::stdin::edit;
/// use thag_rs::tui_editor::{CrosstermEventReader, MockEventReader};
/// use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers };
///
/// let mut event_reader = MockEventReader::new();
/// event_reader.expect_read_event().return_once(|| {
///     Ok(Event::Key(KeyEvent::new(
///         KeyCode::Char('d'),
///         KeyModifiers::CONTROL,
///     )))
/// });
/// let actual = edit(&event_reader);
/// let buf = vec![""];
/// assert!(matches!(actual, Ok(buf)));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
/// # Panics
///
/// If the terminal cannot be reset.
#[allow(clippy::too_many_lines)]
pub fn edit<R: EventReader>(event_reader: &R) -> ThagResult<Vec<String>> {
    let input = std::io::stdin();
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");

    let mut history = History::load_from_file(&history_path);

    let mut saved_to_history = false;

    let initial_content = if input.is_terminal() {
        String::new()
    } else {
        read()?
    };

    let mut popup = false;
    let mut alt_highlights = false;

    let mut stdout = io::stdout().lock();
    enable_raw_mode()?;

    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .map_err(|e| {
        // println!("Error executing terminal commands: {:?}", e);
        e
    })?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?; // Ensure terminal will get reset when it goes out of scope.
    let mut term = scopeguard::guard(terminal, |term| {
        reset_term(term).expect("Error resetting terminal");
    });

    let mut textarea = TextArea::from(initial_content.lines());

    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter / paste / edit Rust script.  ^D: submit  ^Q: quit  ^L: keys  ^T: toggle highlighting")
            .title_style(Style::from(&Lvl::HEAD).bold(),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_selection_style(Style::default().bg(Color::Blue));
    textarea.set_cursor_style(Style::default().on_magenta());
    textarea.set_cursor_line_style(Style::default().on_dark_gray());

    textarea.move_cursor(CursorMove::Bottom);

    apply_highlights(&TUI_SELECTION_BG, &mut textarea);

    let fmt = KeyCombinationFormat::default();
    loop {
        term.draw(|f| {
            f.render_widget(&textarea, f.area());
            if popup {
                show_popup(MAPPINGS, f, TITLE_TOP, TITLE_BOTTOM, &[""; 0], &[]);
            }
            apply_highlights(&TUI_SELECTION_BG, &mut textarea);
        })
        .map_err(|e| {
            println!("Error drawing terminal: {:?}", e);
            e
        })?;
        // NB: leave in raw mode until end of session to avoid random appearance of OSC codes on screen
        // let event = crossterm::event::read();
        let event = event_reader.read_event();
        // terminal::disable_raw_mode()?;

        if let Ok(Paste(ref data)) = event {
            textarea.insert_str(normalize_newlines(data));
        } else {
            match event {
                Ok(Event::Key(key_event)) => {
                    // let Some(key_combination) = combiner.transform(key_event) else {
                    //     continue;
                    // };
                    let key_combination = key_event.into();
                    let key = fmt.to_string(key_combination);
                    match key_combination {
                        #[allow(clippy::unnested_or_patterns)]
                        key!(ctrl - c) | key!(ctrl - q) => {
                            println!("You typed {} which gracefully quits", key.green());
                            return Ok(vec![]);
                        }
                        // key!(ctrl - q - q - q) => {
                        //     println!("You typed {} which gracefully quits", key.green());
                        //     return Ok(());
                        // }
                        key!(ctrl - d) => {
                            // 6 >5,4,3,2,1 -> 6 >6,5,4,3,2,1
                            history.add_entry(textarea.lines().to_vec().join("\n"));
                            history.current_index = Some(0);
                            history.save_to_file(&history_path);
                            break;
                        }
                        key!(ctrl - l) => popup = !popup,
                        key!(ctrl - t) => {
                            alt_highlights = !alt_highlights;
                            term.draw(|_| {
                                apply_highlights(&TUI_SELECTION_BG, &mut textarea);
                            })?;
                        }
                        key!(f1) => {
                            let mut found = false;
                            // 6 5,4,3,2,1 -> >5,4,3,2,1
                            if saved_to_history {
                                if let Some(entry) = history.get_previous() {
                                    // 5
                                    found = true;
                                    textarea.select_all();
                                    textarea.cut(); // 6
                                    textarea.insert_str(entry); // 5
                                }
                            } else {
                                // println!("Not already saved to history: calling history.get_current()");
                                if let Some(entry) = history.get_current() {
                                    found = true;
                                    textarea.select_all();
                                    textarea.cut(); // 6
                                    textarea.insert_str(entry); // 5
                                }
                            }
                            if found && !saved_to_history && !textarea.yank_text().is_empty() {
                                // 5 >5,4,3,2,1 -> 5 6,>5,4,3,2,1
                                history.add_entry(
                                    textarea.yank_text().lines().collect::<Vec<_>>().join("\n"),
                                );
                                saved_to_history = true;
                            }
                            continue;
                        }
                        key!(f2) => {
                            // 5 >6,5,4,3,2,1 ->
                            if let Some(entry) = history.get_next() {
                                textarea.select_all();
                                textarea.cut();
                                textarea.insert_str(entry);
                            }
                            continue;
                        }
                        key!(f3) => {
                            println!("You typed {} which represents `edit toml`", key.green());
                            continue;
                        }
                        key!(f4) => {
                            println!("You typed {} which represents nothing yet", key.green());
                            continue;
                        }
                        #[allow(clippy::unnested_or_patterns)]
                        key!('?') | key!(shift - '?') => {
                            println!("{}", "You typed {} which represents nothing yet".blue());
                        }
                        _ => {
                            // println!("You typed {} which represents nothing yet", key.blue());
                            let input = Input::from(event?);
                            textarea.input(input);
                        }
                    }
                }
                _ => {
                    // any other event, for example a resize, we quit
                    // eprintln!("Quitting on {:?}", e);
                    continue;
                }
            }
        }
    }

    Ok(textarea.lines().to_vec())
}

/// Edit the stdin stream.
///
///
/// # Examples
///
/// ```no_run
/// use thag_rs::stdin::edit;
/// use thag_rs::tui_editor::{CrosstermEventReader, MockEventReader};
/// use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers };
///
/// let mut event_reader = MockEventReader::new();
/// event_reader.expect_read_event().return_once(|| {
///     Ok(Event::Key(KeyEvent::new(
///         KeyCode::Char('d'),
///         KeyModifiers::CONTROL,
///     )))
/// });
/// let actual = edit(&event_reader);
/// let buf = vec![""];
/// assert!(matches!(actual, Ok(buf)));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
/// # Panics
///
/// If the terminal cannot be reset.
#[allow(clippy::too_many_lines)]
pub fn edit_new<R: EventReader>(event_reader: &R) -> ThagResult<Vec<String>> {
    let input = std::io::stdin();
    let cargo_home_string = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let cargo_home = &PathBuf::from(&cargo_home_string);
    let history_path = cargo_home.join("rs_stdin_history.json");

    let mut history = History::load_from_file(&history_path);
    let mut saved_to_history = false;
    let save_path: PathBuf = cargo_home.join("stdin_edit_save.rs");

    let initial_content = if input.is_terminal() {
        String::new()
    } else {
        read()?
    };

    let edit_data = EditData {
        return_text: true,
        initial_content: initial_content.as_str(),
        save_path: Some(save_path),
        history_path: None,
        history: None::<History>,
    };
    let binding = [KeyDisplayLine::new(
        371,
        "F3",
        "Discard saved and unsaved changes and exit",
    )];
    let display = Display {
        title: "Edit REPL script.  ^d: submit  ^q: quit  ^s: save  F3: abandon  ^l: keys  ^t: toggle highlighting",
        title_style: Style::from(&Lvl::SUBH).bold(),
        remove_keys: &[""; 0],
        add_keys: &binding,
    };

    let mut popup = false;
    let mut alt_highlights = false;

    let mut stdout = io::stdout().lock();
    enable_raw_mode()?;

    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .map_err(|e| {
        // println!("Error executing terminal commands: {:?}", e);
        e
    })?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?; // Ensure terminal will get reset when it goes out of scope.
    let mut term = scopeguard::guard(terminal, |term| {
        reset_term(term).expect("Error resetting terminal");
    });

    let mut textarea = TextArea::from(initial_content.clone().lines());

    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter / paste / edit Rust script.  ^D: submit  ^Q: quit  ^L: keys  ^T: toggle highlighting")
            .title_style(Style::from(&Lvl::HEAD).bold()),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_selection_style(Style::default().bg(Color::Blue));
    textarea.set_cursor_style(Style::default().on_magenta());
    textarea.set_cursor_line_style(Style::default().on_dark_gray());

    textarea.move_cursor(CursorMove::Bottom);

    apply_highlights(&TUI_SELECTION_BG, &mut textarea);

    let fmt = KeyCombinationFormat::default();
    loop {
        term.draw(|f| {
            f.render_widget(&textarea, f.area());
            if popup {
                show_popup(MAPPINGS, f, TITLE_TOP, TITLE_BOTTOM, &[""; 0], &[]);
            }
            apply_highlights(&TUI_SELECTION_BG, &mut textarea);
        })
        .map_err(|e| {
            println!("Error drawing terminal: {:?}", e);
            e
        })?;
        // NB: leave in raw mode until end of session to avoid random appearance of OSC codes on screen
        // let event = crossterm::event::read();
        let event = event_reader.read_event();
        // terminal::disable_raw_mode()?;

        if let Ok(Paste(ref data)) = event {
            textarea.insert_str(normalize_newlines(data));
        } else {
            match event {
                Ok(Event::Key(key_event)) => {
                    // let Some(key_combination) = combiner.transform(key_event) else {
                    //     continue;
                    // };
                    let key_combination = key_event.into();
                    let key = fmt.to_string(key_combination);
                    match key_combination {
                        #[allow(clippy::unnested_or_patterns)]
                        key!(ctrl - c) | key!(ctrl - q) => {
                            println!("You typed {} which gracefully quits", key.green());
                            return Ok(vec![]);
                        }
                        // key!(ctrl - q - q - q) => {
                        //     println!("You typed {} which gracefully quits", key.green());
                        //     return Ok(());
                        // }
                        key!(ctrl - d) => {
                            // 6 >5,4,3,2,1 -> 6 >6,5,4,3,2,1
                            history.add_entry(textarea.lines().to_vec().join("\n"));
                            history.current_index = Some(0);
                            history.save_to_file(&history_path);
                            break;
                        }
                        key!(ctrl - l) => popup = !popup,
                        key!(ctrl - t) => {
                            alt_highlights = !alt_highlights;
                            term.draw(|_| {
                                apply_highlights(&TUI_SELECTION_BG, &mut textarea);
                            })?;
                        }
                        key!(f1) => {
                            let mut found = false;
                            // 6 5,4,3,2,1 -> >5,4,3,2,1
                            if saved_to_history {
                                if let Some(entry) = history.get_previous() {
                                    // 5
                                    found = true;
                                    textarea.select_all();
                                    textarea.cut(); // 6
                                    textarea.insert_str(entry); // 5
                                }
                            } else {
                                // println!("Not already saved to history: calling history.get_current()");
                                if let Some(entry) = history.get_current() {
                                    found = true;
                                    textarea.select_all();
                                    textarea.cut(); // 6
                                    textarea.insert_str(entry); // 5
                                }
                            }
                            if found && !saved_to_history && !textarea.yank_text().is_empty() {
                                // 5 >5,4,3,2,1 -> 5 6,>5,4,3,2,1
                                history.add_entry(
                                    textarea.yank_text().lines().collect::<Vec<_>>().join("\n"),
                                );
                                saved_to_history = true;
                            }
                            continue;
                        }
                        key!(f2) => {
                            // 5 >6,5,4,3,2,1 ->
                            if let Some(entry) = history.get_next() {
                                textarea.select_all();
                                textarea.cut();
                                textarea.insert_str(entry);
                            }
                            continue;
                        }
                        key!(f3) => {
                            println!("You typed {} which represents `edit toml`", key.green());
                            continue;
                        }
                        key!(f4) => {
                            println!("You typed {} which represents nothing yet", key.green());
                            continue;
                        }
                        #[allow(clippy::unnested_or_patterns)]
                        key!('?') | key!(shift - '?') => {
                            println!("{}", "You typed {} which represents nothing yet".blue());
                        }
                        _ => {
                            // println!("You typed {} which represents nothing yet", key.blue());
                            let input = Input::from(event?);
                            textarea.input(input);
                        }
                    }
                }
                _ => {
                    // any other event, for example a resize, we quit
                    // eprintln!("Quitting on {:?}", e);
                    continue;
                }
            }
        }
    }

    Ok(textarea.lines().to_vec())
}

/// Prompt for and read Rust source code from stdin.
///
/// # Examples
///
/// ```
/// use thag_rs::stdin::read;
///
/// let hello = String::from("Hello world!");
/// assert!(matches!(read(), Ok(hello)));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read() -> Result<String, std::io::Error> {
    log!(Verbosity::Normal, "Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    let buffer = read_to_string(&mut std::io::stdin().lock())?;
    Ok(buffer)
}

/// Read the input from a `BufRead` implementing item into a String.
///
/// # Examples
///
/// ```
/// use thag_rs::stdin::read_to_string;
///
/// let stdin = std::io::stdin();
/// let mut input = stdin.lock();
/// let hello = String::from("Hello world!");
/// assert!(matches!(read_to_string(&mut input), Ok(hello)));
/// ```
///
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Open the history file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
#[allow(clippy::unnecessary_wraps)]
pub fn edit_history() -> ThagResult<Option<String>> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");
    println!("history_path={history_path:#?}");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&history_path)?;
    edit_file(&history_path)?;
    Ok(Some(String::from("End of history file edit")))
}
