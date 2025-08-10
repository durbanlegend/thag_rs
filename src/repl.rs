#![allow(clippy::uninlined_format_args)]
use crate::{
    builder::process_expr,
    code_utils::{self, clean_up, display_dir_contents, extract_ast_expr},
    key, lazy_static_var,
    manifest::extract,
    tui_editor::{
        script_key_handler, tui_edit, EditData, Entry, History, KeyAction, KeyDisplay,
        ManagedTerminal, RataStyle,
    },
    BuildState, Cli, ColorSupport, CrosstermEventReader, EventReader, KeyCombination,
    KeyDisplayLine, ProcFlags, ThagError, ThagResult,
};
use clap::{CommandFactory, Parser};
use edit::edit_file;
use nu_ansi_term::Color as NuColor;
use ratatui::crossterm::event::{KeyEvent, KeyEventKind};
use ratatui::style::Color;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultHinter, DefaultValidator,
    EditCommand, Emacs, ExampleHighlighter, FileBackedHistory, HistoryItem, KeyCode, KeyModifiers,
    Keybindings, MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch,
    PromptHistorySearchStatus, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use regex::Regex;
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Write as _},
    fs::{self, read_to_string, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};
use thag_common::{get_verbosity, re, vprtln, V};
use thag_profiler::profiled;
use thag_styling::{
    cvprtln, display_terminal_attributes, display_theme_details, display_theme_roles,
    Role::{self, Success},
    Style, TermAttributes,
};
use tui_textarea::{Input, TextArea};

/// The filename for the REPL history file.
pub const HISTORY_FILE: &str = "thag_repl_hist.txt";

/// The default multiline indicator string used in the REPL prompt.
pub static DEFAULT_MULTILINE_INDICATOR: &str = "";

const EVENT_DESCS: &[[&str; 2]; 33] = &[
    [
        "HistoryHintComplete",
        "Complete history hint (default in full)",
    ],
    [
        "HistoryHintWordComplete",
        "Complete a single token/word of the history hint",
    ],
    ["CtrlD", "Handle EndOfLine event"],
    ["CtrlC", "Handle SIGTERM key input"],
    [
        "ClearScreen",
        "Clears the screen and sets prompt to first line",
    ],
    [
        "ClearScrollback",
        "Clears the screen and the scrollback buffer, sets the prompt back to the first line",
    ],
    ["Enter", "Handle enter event"],
    ["Submit", "Handle unconditional submit event"],
    [
        "SubmitOrNewline",
        "Submit at the end of the *complete* text, otherwise newline",
    ],
    ["Esc", "Esc event"],
    ["Mouse", "Mouse"],
    ["Resize(u16, u16)", "trigger terminal resize"],
    [
        "Edit(Vec<EditCommand>)",
        "Run §these commands in the editor",
    ],
    ["Repaint", "Trigger full repaint"],
    [
        "PreviousHistory",
        "Navigate to the previous historic buffer",
    ],
    [
        "Up",
        "Move up to the previous line, if multiline, or up into the historic buffers",
    ],
    [
        "Down",
        "Move down to the next line, if multiline, or down through the historic buffers",
    ],
    [
        "Right",
        "Move right to the next column, completion entry, or complete hint",
    ],
    ["Left", "Move left to the next column, or completion entry"],
    ["NextHistory", "Navigate to the next historic buffer"],
    ["SearchHistory", "Search the history for a string"],
    ["Multiple(Vec<ReedlineEvent>)", "Multiple chained (Vi)"],
    ["UntilFound(Vec<ReedlineEvent>)", "Test"],
    [
        "Menu(String)",
        "Trigger a menu event. It activates a menu with the event name",
    ],
    ["MenuNext", "Next element in the menu"],
    ["MenuPrevious", "Previous element in the menu"],
    ["MenuUp", "Moves up in the menu"],
    ["MenuDown", "Moves down in the menu"],
    ["MenuLeft", "Moves left in the menu"],
    ["MenuRight", "Moves right in the menu"],
    ["MenuPageNext", "Move to the next history page"],
    ["MenuPagePrevious", "Move to the previous history page"],
    ["OpenEditor", "Open text editor"],
];

const CMD_DESCS: &[[&str; 2]; 59] = &[
    ["MoveToStart", "Move to the start of the buffer"],
    ["MoveToLineStart", "Move to the start of the current line"],
    ["MoveToEnd", "Move to the end of the buffer"],
    ["MoveToLineEnd", "Move to the end of the current line"],
    ["MoveLeft", "Move one character to the left"],
    ["MoveRight", "Move one character to the right"],
    ["MoveWordLeft", "Move one word to the left"],
    ["MoveBigWordLeft", "Move one WORD to the left"],
    ["MoveWordRight", "Move one word to the right"],
    ["MoveWordRightStart", "Move one word to the right, stop at start of word"],
    ["MoveBigWordRightStart", "Move one WORD to the right, stop at start of WORD"],
    ["MoveWordRightEnd", "Move one word to the right, stop at end of word"],
    ["MoveBigWordRightEnd", "Move one WORD to the right, stop at end of WORD"],
    ["MoveToPosition", "Move to position"],
    ["InsertChar", "Insert a character at the current insertion point"],
    ["InsertString", "Insert a string at the current insertion point"],
    ["InsertNewline", "Insert the system specific new line character"],
    ["ReplaceChars", "Replace characters with string"],
    ["Backspace", "Backspace delete from the current insertion point"],
    ["Delete", "Delete in-place from the current insertion point"],
    ["CutChar", "Cut the grapheme right from the current insertion point"],
    ["BackspaceWord", "Backspace delete a word from the current insertion point"],
    ["DeleteWord", "Delete in-place a word from the current insertion point"],
    ["Clear", "Clear the current buffer"],
    ["ClearToLineEnd", "Clear to the end of the current line"],
    ["Complete", "Insert completion: entire completion if there is only one possibility, or else up to shared prefix."],
    ["CutCurrentLine", "Cut the current line"],
    ["CutFromStart", "Cut from the start of the buffer to the insertion point"],
    ["CutFromLineStart", "Cut from the start of the current line to the insertion point"],
    ["CutToEnd", "Cut from the insertion point to the end of the buffer"],
    ["CutToLineEnd", "Cut from the insertion point to the end of the current line"],
    ["CutWordLeft", "Cut the word left of the insertion point"],
    ["CutBigWordLeft", "Cut the WORD left of the insertion point"],
    ["CutWordRight", "Cut the word right of the insertion point"],
    ["CutBigWordRight", "Cut the WORD right of the insertion point"],
    ["CutWordRightToNext", "Cut the word right of the insertion point and any following space"],
    ["CutBigWordRightToNext", "Cut the WORD right of the insertion point and any following space"],
    ["PasteCutBufferBefore", "Paste the cut buffer in front of the insertion point (Emacs, vi P)"],
    ["PasteCutBufferAfter", "Paste the cut buffer in front of the insertion point (vi p)"],
    ["UppercaseWord", "Upper case the current word"],
    ["LowercaseWord", "Lower case the current word"],
    ["CapitalizeChar", "Capitalize the current character"],
    ["SwitchcaseChar", "Switch the case of the current character"],
    ["SwapWords", "Swap the current word with the word to the right"],
    ["SwapGraphemes", "Swap the current grapheme/character with the one to the right"],
    ["Undo", "Undo the previous edit command"],
    ["Redo", "Redo an edit command from the undo history"],
    ["CutRightUntil", "CutUntil right until char"],
    ["CutRightBefore", "CutUntil right before char"],
    ["MoveRightUntil", "MoveUntil right until char"],
    ["MoveRightBefore", "MoveUntil right before char"],
    ["CutLeftUntil", "CutUntil left until char"],
    ["CutLeftBefore", "CutUntil left before char"],
    ["MoveLeftUntil", "MoveUntil left until char"],
    ["MoveLeftBefore", "MoveUntil left before char"],
    ["SelectAll", "Select whole input buffer"],
    ["CutSelection", "Cut selection to local buffer"],
    ["CopySelection", "Copy selection to local buffer"],
    ["Paste", "Paste content from local buffer at the current cursor position"],
];

/// REPL mode lets you type or paste a Rust expression to be evaluated.
///
/// Enter the expression to be evaluated.
///
/// Expressions between matching braces, brackets, parens or quotes may span multiple lines.
///
/// If valid, the expression will be converted into a Rust program, and built and run using Cargo.
///
/// Dependencies will be inferred from imports if possible using a Cargo search, but the overhead
/// of doing so can be avoided by placing them in Cargo.toml format at the top of the expression in a
/// comment block of the form
/// ``` rustdoc
/// /*[toml]
/// [dependencies]
/// ...
/// */
/// ```
/// From here they will be extracted to a dedicated Cargo.toml file.
///
/// In this case the whole expression must be enclosed in curly braces to include the TOML in the expression.
///
/// At any stage before exiting the REPL, or at least as long as your TMPDIR is not cleared, you can
/// go back and edit your expression or its generated Cargo.toml file and copy or save them from the
/// editor or directly from their temporary disk locations.
///
/// The tab key will show command selections and complete partial matching selections.
#[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
#[command(
    name = "",
    disable_help_flag = true,
    disable_help_subcommand = true,
    verbatim_doc_comment
)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "snake_case")]
#[allow(clippy::module_name_repetitions)]
pub enum ReplCommand {
    /// Show the REPL banner
    Banner,
    /// Promote the Rust expression to the TUI REPL, which can handle any script.
    /// This is a one-way process, but the original expression will be saved in history.
    Tui,
    /// Edit the Rust expression in the configured or default editor.
    /// Edit+run is an alternative to prompt-line evaluation or TUI for longer snippets and programs.
    Edit,
    /// Edit the generated Cargo.toml
    Toml,
    /// Attempt to build and run the Rust expression
    Run,
    /// Delete all temporary files for the current evaluation (see `list` command)
    Delete,
    /// List temporary files for this the current evaluation
    List,
    /// Edit history
    History,
    /// Show help information
    Help,
    /// Show key bindings
    Keys,
    /// Show theme and terminal attributes (change via `thag -C` and rerun)
    Theme,
    /// Exit the REPL
    Quit,
}

impl ReplCommand {
    #[profiled]
    fn print_help() {
        let mut command = Self::command();
        // let mut buf = Vec::new();
        // command.write_help(&mut buf).unwrap();
        // let help_message = String::from_utf8(buf).unwrap();
        println!("{}", command.render_long_help());
    }
}

/// A struct to implement the Prompt trait.
#[allow(clippy::module_name_repetitions)]
pub struct ReplPrompt(pub &'static str);
impl Prompt for ReplPrompt {
    #[profiled]
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(self.0.to_string())
    }

    #[profiled]
    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Owned(String::new())
    }

    #[profiled]
    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        Cow::Owned("> ".to_string())
    }

    #[profiled]
    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_MULTILINE_INDICATOR)
    }

    #[profiled]
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

    #[profiled]
    fn get_prompt_color(&self) -> reedline::Color {
        if let Some(color_info) = Style::for_role(Success).foreground {
            Color::Indexed(color_info.index).into()
        } else {
            vprtln!(V::VV, "defaulting to Green");
            Color::Green.into()
        }
    }
}

#[profiled]
fn get_heading_style() -> &'static Style {
    lazy_static_var!(Style, Style::for_role(Role::HD1))
}

#[profiled]
fn get_subhead_style() -> &'static Style {
    lazy_static_var!(Style, Style::for_role(Role::HD2))
}

/// Add menu keybindings to the provided keybindings configuration.
#[profiled]
pub fn add_menu_keybindings(keybindings: &mut Keybindings) {
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
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::F(7),
        ReedlineEvent::PreviousHistory,
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::F(8),
        ReedlineEvent::NextHistory,
    );
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
    let history_path = build_state.cargo_home.join(HISTORY_FILE);
    let hist_staging_path: PathBuf = build_state.cargo_home.join("hist_staging.txt");
    let hist_backup_path: PathBuf = build_state.cargo_home.join("hist_backup.txt");
    let history = Box::new(FileBackedHistory::with_file(25, history_path.clone())?);

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
    let mut line_editor_builder = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_history(history)
        .with_history_exclusion_prefix(Some("q".into()))
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode)
        .with_ansi_colors(TermAttributes::get_or_init().color_support != ColorSupport::None);

    let term_attrs = TermAttributes::get_or_init();

    // Only add highlighting if color support is available
    if matches!(
        term_attrs.color_support,
        ColorSupport::None | ColorSupport::Undetermined
    ) {
        // Add default hinter without styling when no color support
        line_editor_builder = line_editor_builder.with_hinter(Box::new(DefaultHinter::default()));
    } else {
        let mut highlighter = Box::new(ExampleHighlighter::new(cmd_vec.clone()));
        let nu_match = {
            Style::for_role(Role::Code)
                .foreground
                .as_ref()
                .map_or(NuColor::Green, NuColor::from)
        };
        let nu_notmatch = {
            Style::for_role(Role::Info)
                .foreground
                .as_ref()
                .map_or(NuColor::Red, NuColor::from)
        };
        let nu_neutral = {
            Style::for_role(Role::Emphasis)
                .foreground
                .as_ref()
                .map_or(NuColor::DarkGray, NuColor::from)
        };
        highlighter.change_colors(nu_match, nu_notmatch, nu_neutral);

        // Add highlighter to builder
        line_editor_builder = line_editor_builder.with_highlighter(highlighter);

        // Add styled hinter
        let nu_hint = nu_ansi_term::Style::from(&Role::Hint);
        line_editor_builder = line_editor_builder.with_hinter(Box::new(
            DefaultHinter::default().with_style(nu_hint.italic()),
        ));
    }

    let mut line_editor = line_editor_builder;

    let bindings = keybindings.get_keybindings();
    let reedline_events = bindings.values().cloned().collect::<Vec<ReedlineEvent>>();
    let max_cmd_len = get_max_cmd_len(&reedline_events);

    let prompt = ReplPrompt("repl");
    let cmd_list = &cmd_vec.join(", ");
    disp_repl_banner(cmd_list);

    // Collect and format key bindings while user is taking in the display banner
    // NB: Can't extract this to a method either, because reedline does not expose KeyCombination.
    let named_reedline_events = bindings
        .iter()
        .map(|(key_combination, reedline_event)| {
            let key_modifiers = key_combination.modifier;
            let key_code = key_combination.key_code;
            let modifier = format_key_modifier(key_modifiers);
            let key = format_key_code(key_code);
            let key_desc = format!("{modifier}{key}");
            (key_desc, reedline_event)
        })
        // .cloned()
        .collect::<Vec<(String, &ReedlineEvent)>>();
    let formatted_bindings = format_bindings(&named_reedline_events, max_cmd_len);

    // Determine the length of the longest key description for padding
    let max_key_len = lazy_static_var!(usize, deref, get_max_key_len(formatted_bindings));
    // eprintln!("max_key_len={max_key_len}");

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
        // vprtln!(V::VV, "first_word={first_word}, rest={rest:#?}");
        let maybe_cmd = if rest.is_empty() {
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
        } else {
            None
        };

        if let Some(cmd) = maybe_cmd {
            if let Ok(repl_command) = ReplCommand::from_str(&cmd) {
                match repl_command {
                    ReplCommand::Banner => disp_repl_banner(cmd_list),
                    ReplCommand::Help => {
                        ReplCommand::print_help();
                    }
                    ReplCommand::Quit => {
                        break;
                    }
                    ReplCommand::Tui => {
                        if TermAttributes::get_or_init().color_support == ColorSupport::None {
                            println!("Sorry, TUI features require terminal color support");
                            continue;
                        }
                        let source_path = &build_state.source_path;
                        let save_path: PathBuf = build_state.cargo_home.join("repl_tui_save.rs");
                        // let backup_path: PathBuf =
                        //     &build_state.cargo_home.join("repl_tui_backup.rs");

                        let rs_source = read_to_string(source_path)?;
                        tui(
                            rs_source.as_str(),
                            &save_path,
                            build_state,
                            args,
                            proc_flags,
                        )?;
                    }
                    ReplCommand::Edit => {
                        edit(&build_state.source_path)?;
                    }
                    ReplCommand::Toml => {
                        toml(build_state)?;
                    }
                    ReplCommand::Run => {
                        let rs_source = code_utils::read_file_contents(&build_state.source_path)?;
                        process_source(&rs_source, build_state, args, proc_flags, start)?;
                    }
                    ReplCommand::Delete => {
                        delete(build_state)?;
                    }
                    ReplCommand::List => {
                        list(build_state)?;
                    }
                    ReplCommand::History => {
                        if TermAttributes::get_or_init().color_support == ColorSupport::None {
                            println!("Sorry, TUI features require terminal color support");
                            continue;
                        }
                        review_history(
                            &mut line_editor,
                            &history_path,
                            &hist_backup_path,
                            &hist_staging_path,
                        )?;
                    }
                    ReplCommand::Keys => {
                        show_key_bindings(formatted_bindings, max_key_len);
                    }
                    ReplCommand::Theme => {
                        let term_attrs = TermAttributes::get_or_init();
                        let theme = &term_attrs.theme;

                        display_theme_roles(theme);
                        display_theme_details(theme);
                        display_terminal_attributes(theme);
                    }
                }
                continue;
            }
        }

        process_source(rs_source, build_state, args, proc_flags, start)?;
    }
    Ok(())
}

/// Process a source string through to completion according to the arguments passed in.
///
/// # Errors
///
/// This function will bubble up any error encountered in processing.
#[profiled]
pub fn process_source(
    rs_source: &str,
    build_state: &mut BuildState,
    args: &Cli,
    proc_flags: &ProcFlags,
    start: Instant,
) -> ThagResult<()> {
    let rs_manifest = extract(rs_source, Instant::now())?;
    build_state.rs_manifest = Some(rs_manifest);
    let maybe_ast = extract_ast_expr(rs_source);
    if let Ok(expr_ast) = maybe_ast {
        build_state.ast = Some(crate::Ast::Expr(expr_ast));
        process_expr(build_state, rs_source, args, proc_flags, &start)?;
    } else {
        cvprtln!(
            Role::ERR,
            get_verbosity(),
            "Error parsing code: {maybe_ast:#?}"
        );
    }
    Ok(())
}

#[profiled]
fn tui(
    initial_content: &str,
    save_path: &Path,
    build_state: &mut BuildState,
    args: &Cli,
    proc_flags: &ProcFlags,
) -> ThagResult<()> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");
    let mut history = History::load_from_file(&history_path);
    let initial_content = if initial_content.trim().is_empty() {
        history.get_last().map_or_else(String::new, Entry::contents)
    } else {
        history.add_entry(initial_content);
        history.save_to_file(&history_path)?;
        initial_content.to_string()
    };

    let event_reader = CrosstermEventReader;
    let mut edit_data = EditData {
        return_text: true,
        initial_content: &initial_content,
        save_path: Some(save_path.to_path_buf()),
        history_path: Some(&history_path),
        history: Some(history),
    };
    let add_keys = [
        KeyDisplayLine::new(371, "Ctrl+Alt+s", "Save a copy"),
        KeyDisplayLine::new(372, "F3", "Discard saved and unsaved changes, and exit"),
        // KeyDisplayLine::new(373, "F4", "Clear text buffer (Ctrl+y or Ctrl+u to restore)"),
    ];

    let style = Style::for_role(Role::HD2);
    let display = KeyDisplay {
        title: "Edit TUI script.  ^d: submit  ^q: quit  ^s: save  F3: abandon  ^l: keys  ^t: toggle highlighting",
        title_style: RataStyle::from(&style),
        remove_keys: &[""; 0],
        add_keys: &add_keys,
    };
    let (key_action, maybe_text) = tui_edit(
        &event_reader,
        &mut edit_data,
        &display,
        |key_event,
         maybe_term,
         /*maybe_save_file,*/ textarea,
         edit_data,
         popup,
         saved,
         status_message| {
            script_key_handler(
                key_event,
                maybe_term, // maybe_save_file,
                textarea,
                edit_data,
                popup,
                saved,
                status_message,
            )
        },
    )?;
    let _ = match key_action {
        // KeyAction::Quit(_saved) => false,
        KeyAction::Save
        | KeyAction::ShowHelp
        | KeyAction::ToggleHighlight
        | KeyAction::TogglePopup => {
            return Err(
                format!("Logic error: {key_action:?} should not return from tui_edit").into(),
            )
        }
        // KeyAction::SaveAndExit => false,
        KeyAction::Submit => {
            return maybe_text.map_or(Err(ThagError::Cancelled), |v| {
                let rs_source = v.join("\n");
                process_source(&rs_source, build_state, args, proc_flags, Instant::now())
            });
        }
        _ => false,
    };
    Ok(())
}

#[profiled]
fn review_history(
    line_editor: &mut Reedline,
    history_path: &PathBuf,
    backup_path: &PathBuf,
    staging_path: &PathBuf,
) -> ThagResult<()> {
    let event_reader = CrosstermEventReader;
    line_editor.sync_history()?;
    fs::copy(history_path, backup_path)?;
    let history_string = read_to_string(history_path)?;
    let confirm = edit_history(&history_string, staging_path, &event_reader)?;
    if confirm {
        let history_mut = line_editor.history_mut();
        let saved_history = fs::read_to_string(staging_path)?;
        eprintln!("staging_path={}", staging_path.display());
        eprintln!("saved_history={saved_history}");
        history_mut.clear()?;
        for line in saved_history.lines() {
            let entry = decode(line);
            // eprintln!("saving entry={entry}");
            let _ = history_mut.save(HistoryItem::from_command_line(entry))?;
        }
        history_mut.sync()?;
    }
    Ok(())
}

/// Convert the `reedline` file-backed history newline sequence <\n> into the '\n' (0xa) character for which it stands.
#[must_use]
#[allow(clippy::missing_panics_doc)]
#[profiled]
pub fn decode(input: &str) -> String {
    let re = re!(r"(<\\n>)");
    let lf = std::str::from_utf8(&[10_u8]).unwrap();
    re.replace_all(input, lf).to_string()
}

/// Edit the history.
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
#[profiled]
pub fn edit_history<R: EventReader + Debug>(
    initial_content: &str,
    staging_path: &Path,
    event_reader: &R,
) -> ThagResult<bool> {
    let mut edit_data = EditData {
        return_text: false,
        initial_content,
        save_path: Some(staging_path.to_path_buf()),
        history_path: None,
        history: None::<History>,
    };
    let binding = [
        KeyDisplayLine::new(372, "F3", "Discard saved and unsaved changes, and exit"),
        // KeyDisplayLine::new(373, "F4", "Clear text buffer (Ctrl+y or Ctrl+u to restore)"),
    ];
    let style = Style::for_role(Role::HD2);
    let display = KeyDisplay {
        title: "Enter / paste / edit REPL history.  ^d: save & exit  ^q: quit  ^s: save  F3: abandon  ^l: keys  ^t: toggle highlighting",
        title_style: RataStyle::from(&style),
        remove_keys: &["F7", "F8"],
        add_keys: &binding,
    };
    let (key_action, _maybe_text) = tui_edit(
        event_reader,
        &mut edit_data,
        &display,
        |key_event, maybe_term, textarea, edit_data, popup, saved, status_message| {
            history_key_handler(
                key_event,
                maybe_term, // maybe_save_file,
                textarea,
                edit_data,
                popup,
                saved,
                status_message,
            )
        },
    )?;
    Ok(match key_action {
        KeyAction::Quit(saved) => saved,
        KeyAction::Save
        | KeyAction::ShowHelp
        | KeyAction::ToggleHighlight
        | KeyAction::TogglePopup => {
            return Err(format!("Logic error: {key_action:?} should not return from tui_edit").into())
        }
        KeyAction::SaveAndSubmit => {
            return Err(format!("Logic error: {key_action:?} should not be implemented in tui_edit or history_key_handler").into()
            )
        }
        KeyAction::SaveAndExit => true,
        _ => false,
    })
}

/// Key handler function to be passed into `tui_edit` for editing REPL history.
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
#[profiled]
pub fn history_key_handler(
    key_event: KeyEvent,
    _maybe_term: Option<&mut ManagedTerminal>,
    // maybe_save_path: &mut Option<&mut PathBuf>,
    textarea: &mut TextArea,
    edit_data: &mut EditData,
    popup: &mut bool,
    saved: &mut bool,
    status_message: &mut String,
) -> ThagResult<KeyAction> {
    // Make sure for Windows
    if !matches!(key_event.kind, KeyEventKind::Press) {
        return Ok(KeyAction::Continue);
    }
    let maybe_save_path = &edit_data.save_path;
    let key_combination = KeyCombination::from(key_event); // Derive KeyCombination

    match key_combination {
        #[allow(clippy::unnested_or_patterns)]
        key!(esc) | key!(ctrl - c) | key!(ctrl - q) => Ok(KeyAction::Quit(*saved)),
        key!(ctrl - d) => {
            // Save logic
            save_file(maybe_save_path.as_ref(), textarea)?;
            // println!("Saved");
            Ok(KeyAction::SaveAndExit)
        }
        key!(ctrl - s) => {
            // Save logic
            let save_file = save_file(maybe_save_path.as_ref(), textarea)?;
            // eprintln!("Saved {:?} to {save_file:?}", textarea.lines());
            *saved = true;
            status_message.clear();
            let _ = write!(status_message, "Saved to {save_file}");
            Ok(KeyAction::Save)
        }
        key!(ctrl - l) => {
            // Toggle popup
            *popup = !*popup;
            Ok(KeyAction::TogglePopup)
        }
        key!(f3) => {
            // Ask to revert
            Ok(KeyAction::AbandonChanges)
        }
        _ => {
            // Update the textarea with the input from the key event
            textarea.input(Input::from(key_event)); // Input derived from Event
            Ok(KeyAction::Continue)
        }
    }
}

#[profiled]
fn save_file(maybe_save_path: Option<&PathBuf>, textarea: &TextArea<'_>) -> ThagResult<String> {
    let staging_path = maybe_save_path.ok_or("Missing save_path")?;
    let staging_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(staging_path)?;
    let mut f = BufWriter::new(&staging_file);
    for line in textarea.lines() {
        Write::write_all(&mut f, line.as_bytes())?;
        Write::write_all(&mut f, b"\n")?;
    }
    Ok(staging_path.display().to_string())
}

/// Return the maximum length of the key descriptor for a set of styled and
/// formatted key / description bindings to be displayed on screen.
#[profiled]
fn get_max_key_len(formatted_bindings: &[(String, String)]) -> usize {
    let style = get_heading_style();
    formatted_bindings
        .iter()
        .map(|(key_desc, _)| {
            let key_desc = style.paint(key_desc);
            key_desc.len()
        })
        .max()
        .unwrap_or(0)
}

#[profiled]
fn format_bindings(
    named_reedline_events: &[(String, &ReedlineEvent)],
    max_cmd_len: usize,
) -> &'static Vec<(String, String)> {
    lazy_static_var!(Vec<(String, String)>, {
        let mut formatted_bindings = named_reedline_events
            .iter()
            .filter_map(|(key_desc, reedline_event)| {
                if let ReedlineEvent::Edit(edit_cmds) = reedline_event {
                    let cmd_desc = format_edit_commands(edit_cmds, max_cmd_len);
                    Some((key_desc.clone(), cmd_desc))
                } else {
                    let event_name = format!("{reedline_event:?}");
                    if event_name.starts_with("UntilFound") {
                        None
                    } else {
                        let event_desc = format_non_edit_events(&event_name, max_cmd_len);
                        Some((key_desc.clone(), event_desc))
                    }
                }
            })
            .collect::<Vec<(String, String)>>();
        // Sort the formatted bindings alphabetically by key combination description
        formatted_bindings.sort_by(|a, b| a.0.cmp(&b.0));
        formatted_bindings
    })
}

#[profiled]
fn get_max_cmd_len(reedline_events: &[ReedlineEvent]) -> usize {
    // Calculate max command len for padding
    lazy_static_var!(usize, deref, {
        // Determine the length of the longest command for padding
        // NB: Can't extract this to a method because for some reason reedline does not expose KeyCombination.
        let style = get_subhead_style();
        let max_cmd_len = reedline_events
            .iter()
            .map(|reedline_event| {
                if let ReedlineEvent::Edit(edit_cmds) = reedline_event {
                    edit_cmds
                        .iter()
                        .map(|cmd| {
                            let key_desc = style.paint(format!("{cmd:?}"));
                            key_desc.len()
                        })
                        .max()
                        .unwrap_or(0)
                } else if !format!("{reedline_event}").starts_with("UntilFound") {
                    let event_desc = style.paint(format!("{reedline_event:?}"));
                    event_desc.len()
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0);
        // Add 2 bytes of padding
        max_cmd_len + 2
    })
}

/// Display key bindings with their descriptions.
#[profiled]
pub fn show_key_bindings(formatted_bindings: &[(String, String)], max_key_len: usize) {
    println!();
    cvprtln!(
        Role::EMPH,
        get_verbosity(),
        "Key bindings - subject to your terminal settings"
    );

    // Print the formatted and sorted key bindings
    let style = get_heading_style();
    for (key_desc, cmd_desc) in formatted_bindings {
        let key_desc = style.paint(key_desc);
        println!("{key_desc:<width$}    {cmd_desc}", width = max_key_len);
    }
    println!();
}

/// Helper function to convert `KeyModifiers` to string
#[must_use]
#[profiled]
pub fn format_key_modifier(modifier: KeyModifiers) -> String {
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
        mods_str
    } else {
        mods_str + "-"
    }
}

/// Helper function to convert `KeyCode` to string
#[must_use]
#[profiled]
pub fn format_key_code(key_code: KeyCode) -> String {
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

/// Helper function to format `ReedlineEvents` other than `Edit`, and their doc comments
/// # Panics
/// Will panic if it fails to split a `EVENT_DESC_MAP` entry, indicating a problem with the `EVENT_DESC_MAP`.
#[allow(clippy::too_many_lines)]
#[must_use]
#[profiled]
pub fn format_non_edit_events(event_name: &str, max_cmd_len: usize) -> String {
    let event_desc_map = lazy_static_var!(HashMap<&'static str, &'static str>, {
        EVENT_DESCS
            .iter()
            .map(|[k, d]| (*k, *d))
            .collect::<HashMap<&'static str, &'static str>>()
    });

    let event_highlight = get_subhead_style().paint(event_name);
    let event_desc = format!(
        "{:<max_cmd_len$} {}",
        event_highlight,
        event_desc_map.get(event_name).unwrap_or(&"")
    );
    event_desc
}

/// Helper function to format `EditCommand` and include its doc comments
/// # Panics
/// Will panic if it fails to split a `CMD_DESC_MAP` entry, indicating a problem with the `CMD_DESC_MAP`.
#[must_use]
#[profiled]
pub fn format_edit_commands(edit_cmds: &[EditCommand], max_cmd_len: usize) -> String {
    let cmd_desc_map: &HashMap<&str, &str> =
        lazy_static_var!(HashMap<&'static str, &'static str>, {
            CMD_DESCS
                .iter()
                .map(|[k, d]| (*k, *d))
                .collect::<HashMap<&'static str, &'static str>>()
        });
    let cmd_descriptions = edit_cmds
        .iter()
        .map(|cmd| format_cmd_desc(cmd, cmd_desc_map, max_cmd_len))
        .collect::<Vec<String>>();

    cmd_descriptions.join(", ")
}

#[allow(clippy::too_many_lines)]
#[profiled]
fn format_cmd_desc(
    cmd: &EditCommand,
    cmd_desc_map: &HashMap<&str, &str>,
    max_cmd_len: usize,
) -> String {
    let style = get_subhead_style();

    let cmd_highlight = style.paint(format!("{cmd:?}"));
    match cmd {
        EditCommand::MoveToStart { select }
        | EditCommand::MoveToLineStart { select }
        | EditCommand::MoveToEnd { select }
        | EditCommand::MoveToLineEnd { select }
        | EditCommand::MoveLeft { select }
        | EditCommand::MoveRight { select }
        | EditCommand::MoveWordLeft { select }
        | EditCommand::MoveBigWordLeft { select }
        | EditCommand::MoveWordRight { select }
        | EditCommand::MoveWordRightStart { select }
        | EditCommand::MoveBigWordRightStart { select }
        | EditCommand::MoveWordRightEnd { select }
        | EditCommand::MoveBigWordRightEnd { select } => format!(
            "{:<max_cmd_len$} {}{}",
            cmd_highlight,
            cmd_desc_map
                .get(format!("{cmd:?}").split_once(' ').unwrap().0)
                .unwrap_or(&""),
            if *select {
                ". Select the text between the current cursor position and destination"
            } else {
                ", without selecting"
            }
        ),
        EditCommand::InsertString(_)
        | EditCommand::InsertNewline
        | EditCommand::ReplaceChar(_)
        | EditCommand::ReplaceChars(_, _)
        | EditCommand::Backspace
        | EditCommand::Delete
        | EditCommand::CutChar
        | EditCommand::BackspaceWord
        | EditCommand::DeleteWord
        | EditCommand::Clear
        | EditCommand::ClearToLineEnd
        | EditCommand::Complete
        | EditCommand::CutCurrentLine
        | EditCommand::CutFromStart
        | EditCommand::CutFromLineStart
        | EditCommand::CutToEnd
        | EditCommand::CutToLineEnd
        | EditCommand::CutWordLeft
        | EditCommand::CutBigWordLeft
        | EditCommand::CutWordRight
        | EditCommand::CutBigWordRight
        | EditCommand::CutWordRightToNext
        | EditCommand::CutBigWordRightToNext
        | EditCommand::PasteCutBufferBefore
        | EditCommand::PasteCutBufferAfter
        | EditCommand::UppercaseWord
        | EditCommand::InsertChar(_)
        | EditCommand::CapitalizeChar
        | EditCommand::SwitchcaseChar
        | EditCommand::SwapWords
        | EditCommand::SwapGraphemes
        | EditCommand::Undo
        | EditCommand::Redo
        | EditCommand::CutRightUntil(_)
        | EditCommand::CutRightBefore(_)
        | EditCommand::CutLeftUntil(_)
        | EditCommand::CutLeftBefore(_)
        | EditCommand::CutSelection
        | EditCommand::CopySelection
        | EditCommand::Paste
        | EditCommand::SelectAll
        | EditCommand::LowercaseWord => format!(
            "{:<max_cmd_len$} {}",
            cmd_highlight,
            cmd_desc_map.get(format!("{cmd:?}").as_str()).unwrap_or(&"")
        ),
        EditCommand::MoveRightUntil { c: _, select }
        | EditCommand::MoveRightBefore { c: _, select }
        | EditCommand::MoveLeftUntil { c: _, select }
        | EditCommand::MoveLeftBefore { c: _, select } => format!(
            "{:<max_cmd_len$} {}. {}",
            cmd_highlight,
            cmd_desc_map
                .get(format!("{cmd:?}").split_once(' ').unwrap().0)
                .unwrap_or(&""),
            if *select {
                "Select the text between the current cursor position and destination"
            } else {
                "without selecting"
            }
        ),
        EditCommand::MoveToPosition { position, select } => format!(
            "{:<max_cmd_len$} {} {} {}",
            cmd_highlight,
            cmd_desc_map
                .get(format!("{cmd:?}").split_once(' ').unwrap().0)
                .unwrap_or(&""),
            position,
            if *select {
                "Select the text between the current cursor position and destination"
            } else {
                "without selecting"
            }
        ),
        // Add other EditCommand variants and their descriptions here
        _ => format!("{:<width$}", cmd_highlight, width = max_cmd_len + 2),
    }
}

/// Delete the temporary files used by the current REPL instance.
/// # Errors
/// Currently will not return any errors.
#[allow(clippy::unnecessary_wraps)]
#[profiled]
pub fn delete(build_state: &BuildState) -> ThagResult<Option<String>> {
    // let build_state = &context.build_state;
    let clean_up = clean_up(&build_state.source_path, &build_state.target_dir_path);
    if clean_up.is_ok()
        || (!&build_state.source_path.exists() && !&build_state.target_dir_path.exists())
    {
        vprtln!(V::QQ, "Deleted");
    } else {
        vprtln!(
            V::QQ,
            "Failed to delete all files - enter l(ist) to list remaining files"
        );
    }
    Ok(Some(String::from("End of delete")))
}

/// Open the generated destination Rust source code file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
#[allow(clippy::unnecessary_wraps)]
#[profiled]
pub fn edit(source_path: &PathBuf) -> ThagResult<Option<String>> {
    edit_file(source_path)?;

    Ok(Some(String::from("End of source edit")))
}

/// Open the generated Cargo.toml file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
#[allow(clippy::unnecessary_wraps)]
#[profiled]
pub fn toml(build_state: &BuildState) -> ThagResult<Option<String>> {
    let cargo_toml_file = &build_state.cargo_toml_path;
    if cargo_toml_file.exists() {
        edit_file(cargo_toml_file)?;
    } else {
        vprtln!(V::QQ, "No Cargo.toml file found - have you run anything?");
    }
    Ok(Some(String::from("End of Cargo.toml edit")))
}

/// Parse the current line. Borrowed from clap-repl crate.
#[must_use]
#[profiled]
pub fn parse_line(line: &str) -> (String, Vec<String>) {
    let re: &Regex = re!(r#"("[^"\n]+"|[\S]+)"#);

    let mut args = re
        .captures_iter(line)
        .map(|a| a[0].to_string().replace('\"', ""))
        .collect::<Vec<String>>();
    let command: String = args.drain(..1).collect();
    (command, args)
}

/// Display the REPL banner.
#[profiled]
pub fn disp_repl_banner(cmd_list: &str) {
    cvprtln!(
        Role::HD1,
        get_verbosity(),
        r#"Enter a Rust expression (e.g., 2 + 3 or "Hi!"), or one of: {cmd_list}."#
    );

    println!();

    cvprtln!(
        Role::HD2,
        get_verbosity(),
        r"Expressions in matching braces, brackets or quotes may span multiple lines."
    );

    cvprtln!(
        Role::HD2,
        get_verbosity(),
        r"Use F7 & F8 to navigate prev/next history, →  to select current. Ctrl-U: clear. Ctrl-K: delete to end."
    );
}

/// Display a list of the temporary files used by the current REPL instance.
/// # Errors
/// This function will return an error in the following situations, but is not limited to just these cases:
/// The provided path doesn't exist.
/// The process lacks permissions to view the contents.
/// The path points at a non-directory file.
#[allow(clippy::unnecessary_wraps)]
#[profiled]
pub fn list(build_state: &BuildState) -> ThagResult<Option<String>> {
    let source_path = &build_state.source_path;
    if source_path.exists() {
        vprtln!(V::QQ, "File: {source_path:?}");
    }

    // Display directory contents
    display_dir_contents(&build_state.target_dir_path)?;

    // Check if neither file nor directory exist
    if !&source_path.exists() && !&build_state.target_dir_path.exists() {
        vprtln!(V::QQ, "No temporary files found");
    }
    Ok(Some(String::from("End of list")))
}
