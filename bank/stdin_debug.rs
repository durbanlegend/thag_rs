/*[toml]
[dependencies]
clap = "4.5.18"
crokey = "1.1.0"
crossterm = "0.28.1"
edit = "0.1.5"
lazy_static = "1.4.0"
log = "0.4.22"
mockall = "0.13.0"
ratatui = "0.28.1"
reedline = "0.36.0"
regex = "1.10.4"
scopeguard = "1.2.0"
serde = "1.0.219"
serde_json = "1.0.132"
simplelog = "0.12.2"
strum = "0.26.3"
thag_rs = { path = "/Users/donf/projects/thag_rs/" }
tui-textarea = { version = "0.6", features = ["search"] }
*/

#![allow(clippy::uninlined_format_args)]

/// A version of `thag_rs`'s `stdin` module for the purpose of debugging an obscure issue with the `edit`
/// function misbehaving under certain code paths. This turned out to be a side effect of `termbg` crate
/// code incorporated into the project, which deliberately switches off raw mode
/// module, `stdin` was originally developed here as a separate script and integrated as a module later.
///
/// E.g. `thag demo/stdin_debug.rs`
//# Purpose: Debugging.
use thag_rs::code_utils::process_expr;
use thag_rs::colors::TUI_SELECTION_BG;
use thag_rs::errors::ThagResult;
use thag_rs::logging::Verbosity;
use thag_rs::repl::{
    add_menu_keybindings, disp_repl_banner, format_edit_commands, format_key_code,
    format_key_modifier, format_non_edit_events, parse_line, show_key_bindings, ReplPrompt,
};
use thag_rs::stdin::{edit_history, toml};
use thag_rs::tui_editor::{
    apply_highlights, normalize_newlines, reset_term, show_popup, History, MAPPINGS, TITLE_BOTTOM,
    TITLE_TOP,
};
use thag_rs::ThagError;
use thag_rs::{
    extract_ast_expr, extract_manifest, log, nu_color_println, nu_resolve_style, BuildState, Cli,
    MessageLevel, ProcFlags,
};

use clap::{CommandFactory, Parser};
use crokey::{crossterm, key, KeyCombinationFormat};
use crossterm::event::{
    DisableMouseCapture,
    EnableBracketedPaste,
    // EnableMouseCapture,
    Event::{self, Paste},
    // KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use log::info;
use mockall::automock;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::Clear;
use ratatui::widgets::{block::Block, Borders};
use ratatui::Terminal;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultHinter, DefaultValidator,
    Emacs, MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use simplelog::*;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufRead, IsTerminal};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use strum::EnumIter;
use strum::{EnumString, IntoEnumIterator, IntoStaticStr};
use tui_textarea::{CursorMove, Input, TextArea};

// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader {
    // Read a terminal event.
    //
    // # Errors
    //
    // This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn read_event(&self) -> ThagResult<Event>;
}

// A struct to implement real-world use of the event reader, as opposed to use in testing.
#[derive(Debug)]
pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> ThagResult<Event> {
        crossterm::event::read().map_err(Into::<ThagError>::into)
    }
}

#[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
#[command(
    name = "",
    disable_help_flag = true,
    disable_help_subcommand = true,
    verbatim_doc_comment
)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "snake_case")]
enum ReplCommand {
    // Show the REPL banner
    Banner,
    // Edit the Rust expression.
    Edit,
    // Edit the generated Cargo.toml
    Toml,
    // Edit history
    History,
    // Show help information
    Help,
    // Show key bindings
    Keys,
    // Exit the REPL
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
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("app.log").unwrap(),
        ),
    ])
    .unwrap();

    info!("Checking the log");

    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        log!(Verbosity::Normal, "{line}");
    }
    Ok(())
}

// Run the REPL.
// # Errors
// Will return `Err` if there is any error in running the REPL.
// # Panics
// Will panic if there is a problem configuring the `reedline` history file.
#[allow(clippy::module_name_repetitions)]
#[allow(clippy::too_many_lines)]
pub fn run_repl(
    args: &Cli,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    start: Instant,
) -> ThagResult<()> {
    #[allow(unused_variables)]
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

// Edit the stdin stream.
#[allow(clippy::too_many_lines)]
pub fn edit<R: EventReader + Debug>(event_reader: &R) -> ThagResult<Vec<String>> {
    let input = std::io::stdin();
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");

    let mut history = History::load_from_file(&history_path);

    let mut saved_to_history = false;

    info!("input.is_terminal()? {:?}", input.is_terminal());
    let initial_content = if input.is_terminal() {
        // String::new()
        String::from("\n")
    } else {
        read()?
    };

    info!("input.initial_content()=|{initial_content}|");

    let mut popup = false;
    let mut alt_highlights = false;

    let mut stdout = io::stdout().lock();
    enable_raw_mode()?;
    info!(
        "0. is_raw_mode_enabled? {:?}",
        crossterm::terminal::is_raw_mode_enabled()
    );

    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        DisableMouseCapture,
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

    info!(
        "1. is_raw_mode_enabled? {:?}",
        crossterm::terminal::is_raw_mode_enabled()
    );

    // info!("term={term:?}");

    let mut textarea = TextArea::from(initial_content.lines());

    info!(
        "1a. is_raw_mode_enabled? {:?}",
        crossterm::terminal::is_raw_mode_enabled()
    );

    let color_index = u8::from(&MessageLevel::Heading);
    info!(
        "1b. is_raw_mode_enabled? {:?}",
        crossterm::terminal::is_raw_mode_enabled()
    );
    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter / paste / edit Rust script.  ^D: submit  ^Q: quit  ^L: keys  ^T: toggle highlighting")
            .title_style(Style::default().fg(Color::Indexed(color_index)).bold()),
    );
    info!(
        "1c. is_raw_mode_enabled? {:?}",
        crossterm::terminal::is_raw_mode_enabled()
    );

    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_selection_style(Style::default().bg(Color::Blue));
    textarea.set_cursor_style(Style::default().on_magenta());
    textarea.set_cursor_line_style(Style::default().on_dark_gray());
    textarea.move_cursor(CursorMove::Bottom);

    info!(
        "2. is_raw_mode_enabled? {:?}",
        crossterm::terminal::is_raw_mode_enabled()
    );

    apply_highlights(&TUI_SELECTION_BG, &mut textarea);

    info!(
        "3. is_raw_mode_enabled? {:?}",
        crossterm::terminal::is_raw_mode_enabled()
    );

    let fmt = KeyCombinationFormat::default();
    loop {
        info!(
            "4. is_raw_mode_enabled? {:?}",
            crossterm::terminal::is_raw_mode_enabled()
        );

        term.clear()?;
        term.draw(|f| {
            f.render_widget(Clear, f.area());
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
        crossterm::terminal::enable_raw_mode()?;
        info!(
            "5. is_raw_mode_enabled? {:?}",
            crossterm::terminal::is_raw_mode_enabled()
        );
        let event = event_reader.read_event();
        // terminal::disable_raw_mode()?;
        info!("event={event:?}");

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

pub fn read() -> Result<String, std::io::Error> {
    log!(Verbosity::Normal, "Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    let buffer = read_to_string(&mut std::io::stdin().lock())?;
    Ok(buffer)
}

pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}
