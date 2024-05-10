#![allow(clippy::uninlined_format_args)]
use crate::cmd_args::{get_opt, get_proc_flags, Opt, ProcFlags};
use crate::code_utils::{
    clean_up, debug_timings, display_dir_contents, display_timings, rustfmt, wrap_snippet,
};
use crate::code_utils::{modified_since_compiled, parse_source, write_source};
use crate::errors::BuildRunError;
use crate::manifest::CargoManifest;
use crate::term_colors::{nu_resolve_style, owo_resolve_style};
use clap::Parser;
use env_logger::{fmt::WriteStyle, Builder, Env};
use homedir::get_my_home;
use lazy_static::lazy_static;
use log::{debug, log_enabled, Level::Debug};
use nu_ansi_term::{Color, Style};
use quote::quote;
use reedline::{
    DefaultHinter, DefaultValidator, FileBackedHistory, Prompt, PromptEditMode,
    PromptHistorySearch, PromptHistorySearchStatus, Reedline, Signal,
};
use reedline_repl_rs::{
    clap::{ArgMatches, Command as ReplCommand},
    Repl,
};
use term_colors::MessageLevel;

// use core::str;
use std::borrow::Cow::{self};
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
// use std::process;
use std::{fs, io::Write as OtherWrite}; // Use PathBuf for paths
use strum::{EnumIter, EnumProperty, IntoEnumIterator, IntoStaticStr};
use syn::{self, Expr};

mod cmd_args;
mod code_utils;
mod errors;
mod manifest;
mod term_colors;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const REPL_SUBDIR: &str = "rs_repl";
const RS_SUFFIX: &str = ".rs";
pub(crate) const TOML_NAME: &str = "Cargo.toml";

lazy_static! {
    static ref TMP_DIR: PathBuf = env::temp_dir();
}

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

#[derive(Debug)]
pub(crate) enum ScriptState {
    /// Repl with no script name provided by user
    #[allow(dead_code)]
    Anonymous,
    /// Repl with script name
    NamedEmpty { script: String },
    /// Script name provided by user
    Named { script: String },
}

impl ScriptState {
    pub(crate) fn get_script(&self) -> Option<String> {
        match self {
            ScriptState::Anonymous => None,
            ScriptState::NamedEmpty { script } | ScriptState::Named { script } => {
                Some(script.to_string())
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BuildState {
    pub(crate) source_stem: String,
    pub(crate) source_name: String,
    pub(crate) source_path: PathBuf,
    pub(crate) cargo_home: PathBuf,
    pub(crate) target_dir_path: PathBuf,
    pub(crate) target_dir_str: String,
    pub(crate) target_path: PathBuf,
    pub(crate) cargo_toml_path: PathBuf,
    pub(crate) cargo_manifest: Option<CargoManifest>,
    pub(crate) must_gen: bool,
    pub(crate) must_build: bool,
}

impl BuildState {
    pub(crate) fn pre_configure(
        proc_flags: &ProcFlags,
        script_state: &ScriptState,
    ) -> Result<Self, Box<dyn Error>> {
        let maybe_script = script_state.get_script();
        if maybe_script.is_none() {
            return Err(Box::new(BuildRunError::NoneOption(
                "No script specified".to_string(),
            )));
        }
        let script = (maybe_script).clone().unwrap();
        let path = Path::new(&script);
        let source_name: String = path.file_name().unwrap().to_str().unwrap().to_string();
        // debug!("source_name={source_name}");
        let source_stem = {
            let Some(stem) = source_name.strip_suffix(RS_SUFFIX) else {
                return Err(Box::new(BuildRunError::Command(format!(
                    "Error stripping suffix from {}",
                    source_name
                ))));
            };
            stem.to_string()
        };
        let source_name = source_name.to_string();
        let current_dir_path = std::env::current_dir()?.canonicalize()?;
        let script_path = current_dir_path.join(PathBuf::from(script.clone()));
        // debug!("script_path={script_path:#?}");
        let source_path = script_path.canonicalize()?;
        // debug!("source_dir_path={source_path:#?}");
        if !source_path.exists() {
            return Err(Box::new(BuildRunError::Command(format!(
                "No script named {} or {} in path {source_path:?}",
                source_stem, source_name
            ))));
        }

        let cargo_home_string: String = match std::env::var("CARGO_HOME") {
            Ok(string) if string != String::new() => string,
            _ => {
                let home_dir = get_my_home()?.ok_or("Can't resolve home directory")?;
                debug!("home_dir={}", home_dir.display());
                home_dir.join(".cargo").display().to_string()
            }
        };
        let cargo_home = PathBuf::from(cargo_home_string);
        let target_dir_path = cargo_home.join(&source_stem);
        debug!("target_dir_path={}", target_dir_path.display());
        let target_dir_str = target_dir_path.display().to_string();
        let mut target_path = target_dir_path.join("target").join("debug");
        #[cfg(windows)]
        {
            target_path = target_path.join(source_stem.clone() + ".exe");
        }
        #[cfg(not(windows))]
        {
            target_path = target_path.join(&source_stem);
        }

        let target_path_clone = target_path.clone();

        let cargo_toml_path = target_dir_path.join(TOML_NAME).clone();

        let mut build_state = Self {
            source_stem,
            source_name,
            source_path,
            cargo_home,
            target_dir_path,
            target_dir_str,
            target_path,
            cargo_toml_path,
            ..Default::default()
        };

        let stale_executable = matches!(script_state, ScriptState::NamedEmpty { .. })
            || !target_path_clone.exists()
            || modified_since_compiled(&build_state).is_some();
        let force = proc_flags.contains(ProcFlags::FORCE);
        let gen_requested = proc_flags.contains(ProcFlags::GENERATE);
        let build_requested = proc_flags.contains(ProcFlags::BUILD);
        let repl = proc_flags.contains(ProcFlags::REPL);
        build_state.must_gen = force || repl || (gen_requested && stale_executable);
        build_state.must_build = force || repl || (build_requested && stale_executable);

        debug!("build_state={build_state:#?}");

        Ok(build_state)
    }
}

#[derive(Debug)]
struct Context<'a> {
    options: &'a mut Opt,
    proc_flags: &'a ProcFlags,
    build_state: &'a mut BuildState,
    start: &'a Instant,
}

#[derive(Debug, Parser, EnumIter, EnumProperty, IntoStaticStr)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
#[strum(serialize_all = "kebab-case")]
enum LoopCommand {
    /// Enter, paste or modify your code and optionally edit your generated Cargo.toml
    #[clap(visible_alias = "c")]
    Edit,
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

// #[derive(Debug, Parser)]
// #[command(name = "", arg_required_else_help(true))] // This name will show up in clap's error messages, so it is important to set it to "".
// enum ProcessCommand {
//     /// Cancel and discard this code, restart REPL
//     Cancel,
//     /// Return to editor for another try
//     Retry,
//     /// Attempt to build and run your Rust code
//     Submit,
//     // Exit REPL
//     Quit,
// }

//      TODO:
//          Next: 1. test in Windows, 2. nu_ansi_term color_println macro.
//                3. test reedline partial completions. 4. Print out all colours again
//       1.  In term_colors, detect if terminal is xterm compatible, and if so choose nicer colors.
//       2.  Don't use println{} when wrapping snippet if return type of expression is ()
//       3.  Replace clap_repl in outer eval loop by reedline.
//       4.  Impractical?: I think so. Bigger fish to fry with Styles interfering with reading responses in Windows.
//              Redo term_colors.rs with 4 individual enums. Reconcile nu-ansi-term colours with owo-colors.
//              See https://codebrowser.dev/rust/crates/owo-colors/src/colors/xterm.rs.html
//                  and end section of term_colors.rs.
//       5.  Inferred deps to use a visitor to find embedded use statements
//       6.  bool -> 2-value enums?
//       7.  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
//       8.  Cat files before delete.
//       9.  --quiet option?.
//      10.  Consider making script name optional, with -n/stdin parm as per my runner changes?
//      11.  Clean up debugging
//      12.  "edit"" crate - how to reconfigure editors dynamically - instructions unclear.
//      13.  Cargo search not being done for snippets - maybe AST issue to be resolved by visitor pattern above.
//      14.  Resolved by switch to reedline: Debug left-brace misbehaviour in eval. One of the MatchingBracket mafia?
//      15   Clap aliases not working in REPL.
//
#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    let mut options = get_opt();
    let proc_flags = get_proc_flags(&options)?;

    if log_enabled!(Debug) {
        debug_print_config();
        debug!("proc_flags={proc_flags:#?}");
        debug_timings(&start, "Set up processing flags");

        if !&options.args.is_empty() {
            debug!("... args:");
            for arg in &options.args {
                debug!("{arg}");
            }
        }
    }

    // Access TMP_DIR
    println!("Temporary directory: {:?}", *TMP_DIR);

    let repl = proc_flags.contains(ProcFlags::REPL);

    let script_state = if let Some(ref script) = options.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name must end in {RS_SUFFIX}"
            ))));
        }
        let script = script.to_owned();
        ScriptState::Named { script }
    } else {
        assert!(repl);
        let path = code_utils::create_next_repl_file();
        let script = path.to_str().unwrap().to_string();
        ScriptState::NamedEmpty { script }
    };

    if repl {
        debug!("script_state={script_state:?}");
    }
    let mut build_state = BuildState::pre_configure(&proc_flags, &script_state)?;
    if repl {
        debug!("build_state.source_path={:?}", build_state.source_path);
    }

    println!("script_state={script_state:?}");
    if repl {
        // let dash_line = "-".repeat(50);

        // Using strum
        let cmd_vec = LoopCommand::iter()
            .map(<LoopCommand as Into<&'static str>>::into)
            .map(String::from)
            .collect::<Vec<String>>();
        let cmd_list = cmd_vec.join(", ") + " or help";
        // debug!(
        //     "resolve_style(term_colors::MessageLevel::OuterPrompt){:#?}",
        //     resolve_style(term_colors::MessageLevel::OuterPrompt)
        // );
        // let outer_prompt = || {
        //     color_println!(
        //         resolve_style(term_colors::MessageLevel::OuterPrompt),
        //         "Enter one of: {:#?}",
        //         cmd_list
        //     );
        // };
        #[allow(unused_variables)]
        let outer_prompt = || {
            println!(
                "{}",
                nu_resolve_style(MessageLevel::OuterPrompt)
                    .unwrap_or_default()
                    .paint(format!("Enter one of: {}", cmd_list))
            );
        };
        // outer_prompt();
        let context = Context {
            options: &mut options,
            proc_flags: &proc_flags,
            build_state: &mut build_state,
            start: &start,
        };
        let mut repl = Repl::new(context)
            .with_name("REPL")
            .with_version("v0.1.0")
            .with_description("REPL mode")
            .with_banner(&format!(
                "{}",
                // nu_resolve_style(MessageLevel::OuterPrompt)
                //     .unwrap_or_default()
                nu_ansi_term::Color::LightMagenta.paint(&format!("Enter one of: {}", cmd_list)),
            ))
            .with_command(ReplCommand::new("delete"), delete)
            .with_command(ReplCommand::new("edit"), edit)
            .with_command(
                ReplCommand::new("eval").subcommand(ReplCommand::new("quit")),
                eval,
            )
            .with_command(ReplCommand::new("list"), list)
            .with_command(ReplCommand::new("quit").aliases(["q", "exit"]), quit)
            .with_stop_on_ctrl_c(true);
        // show help with CTRL+h
        // .with_keybinding(
        //     KeyModifiers::CONTROL,
        //     KeyCode::Char('h'),
        //     ReedlineEvent::ExecuteHostCommand("help".to_string()),
        // );
        // .with_error_handler(|ref _err, ref _repl| process::exit(0)),
        repl.run()?;
        // let mut loop_editor = ClapEditor::<LoopCommand>::new();
        // let mut loop_command = loop_editor.read_command();
        // 'level2: loop {
        //     let Some(ref command) = loop_command else {
        //         loop_command = loop_editor.read_command();
        //         continue 'level2;
        //     };
        //     match command {
        //         LoopCommand::Quit => return Ok(()),
        //         LoopCommand::Delete => {}
        //         LoopCommand::List => {}
        //         LoopCommand::Edit => {}
        //         LoopCommand::Eval => {}
        //     }
        //     loop_command = loop_editor.read_command();
        // }
    } else {
        gen_build_run(
            &&mut options,
            &proc_flags,
            &mut build_state,
            // &mut maybe_syntax_tree,
            // &mut maybe_rs_manifest,
            // &maybe_rs_source,
            &start,
        )?;
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
        println!("Deleted");
    } else {
        println!("Failed to delete all files - enter l(ist) to list remaining files");
    }
    Ok(Some(String::from("End of delete")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn edit(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let (options, proc_flags, build_state, _start) = (
        &mut context.options,
        context.proc_flags,
        &mut context.build_state,
        context.start,
    );

    let files = [
        format!("{}", build_state.source_path.display()),
        format!("{}/Cargo.toml", build_state.target_dir_str),
    ]
    .into_iter();
    debug!("files={files:#?}");
    // let editor = &mut Editor::new(files)?;
    // editor.run()?;
    edit::edit_file(&build_state.source_path)?;

    let context = Context {
        options: &mut (**options).clone(),
        proc_flags: &proc_flags.clone(),
        build_state: &mut build_state.clone(),
        start: &Instant::now(),
    };
    let mut repl: Repl<Context, BuildRunError> = Repl::new(context)
        .with_name("edit")
        .with_banner(&format!(
            "{}",
            nu_resolve_style(MessageLevel::InnerPrompt)
                .unwrap_or_default()
                .paint(String::from("Enter cancel, retry, submit, quit or help"))
        ))
        .with_command(ReplCommand::new("cancel").alias("c"), cancel)
        .with_command(ReplCommand::new("quit").aliases(["q", "exit"]), quit)
        .with_command(ReplCommand::new("retry").alias("r"), edit)
        .with_command(ReplCommand::new("submit").alias("s"), submit)
        .with_stop_on_ctrl_c(true);
    repl.run()?;

    //     ProcessCommand::Submit => {
    //         let result = gen_build_run(
    //             options,
    //             proc_flags,
    //             build_state,
    //             // &mut maybe_syntax_tree,
    //             // &mut maybe_rs_manifest,
    //             // &maybe_rs_source,
    //             start,
    //         );
    //         if result.is_err() {
    //             println!("{result:?}");
    //         }

    //         break 'level3;
    //     }
    //     ProcessCommand::Cancel => {
    //         // loop_command = loop_editor.read_command();
    //         // outer_prompt();
    //         return Ok(Some(String::from("Cancel")));
    //     }
    //     ProcessCommand::Retry => {
    //         // loop_command = Some(LoopCommand::Edit);
    //         // outer_prompt();
    //         return Ok(Some(String::from("Retry")));
    //     }
    // }
    // }

    Ok(Some(String::from("End of continue"))) // TODO make nice
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn cancel(_args: ArgMatches, _context: &mut Context) -> Result<Option<String>, BuildRunError> {
    println!("Cancelled");
    Ok(Some(String::from("Cancelled")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn submit(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let (options, proc_flags, build_state, start) = (
        &mut context.options,
        context.proc_flags,
        &mut context.build_state,
        context.start,
    );
    let result = gen_build_run(
        options,
        proc_flags,
        build_state,
        // &mut maybe_syntax_tree,
        // &mut maybe_rs_manifest,
        // &maybe_rs_source,
        start,
    );
    if result.is_err() {
        println!("{result:?}");
    }

    println!("Submitted");
    Ok(Some(String::from("Submitted")))
}

/// From Reedline validation example with enhancements
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn eval(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let (options, proc_flags, build_state, start) = (
        &mut context.options,
        &context.proc_flags,
        &mut context.build_state,
        &context.start,
    );

    let history_file = build_state.cargo_home.join("rs_eval_hist.txt");
    let history = Box::new(
        FileBackedHistory::with_file(20, history_file)
            .expect("Error configuring history with file"),
    );

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(Style::new().italic().fg(Color::Cyan)),
        ))
        .with_history(history);

    let prompt = CustomPrompt("expr");
    // println!("{:#?}", nu_resolve_style(MessageLevel::InnerPrompt)
    //     .unwrap_or_default()
    //     .paint("nu_resolve_style(MessageLevel::InnerPrompt).unwrap_or_default().paint escape codes").to_string());

    loop {
        println!(
            "{}",
            // nu_resolve_style(MessageLevel::InnerPrompt)
            //     .unwrap_or_default()
            //     .paint(
                    nu_ansi_term::Color::Cyan.paint(
                    r"Enter an expression (e.g., 2 + 3), or q to quit. Expressions in matching braces, brackets or quotes may span multiple lines."
                )
        );

        let sig = line_editor.read_line(&prompt)?;
        let input: &str = match sig {
            Signal::Success(ref buffer) => buffer,
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nAborted!");
                break;
            }
        };
        // Process user input (line)

        let s = input.trim();
        let lc = s.to_lowercase();
        if lc == "q" || lc == "quit" {
            break;
        }
        // Parse the expression string into a syntax tree
        let mut expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(s);
        println!(
            r"expr.is_err()={}, str.starts_with('{{')={}, str.ends_with('}}')={}",
            expr.is_err(),
            s.starts_with('{'),
            s.ends_with('}')
        );
        if expr.is_err() && !(s.starts_with('{') && s.ends_with('}')) {
            // Try putting the expression in braces.
            let string = format!(r"{{{s}}}");
            let str = string.as_str();
            println!("str={str}");

            expr = syn::parse_str::<Expr>(str);
        }

        match expr {
            Ok(expr) => {
                // Generate Rust code for the expression
                let rust_code = quote!(println!("result={:?}", #expr););

                let rs_source = format!("{rust_code}");
                debug!("rs_source={rs_source}"); // Careful, needs to impl Display

                write_source(build_state.source_path.clone(), &rs_source)?;

                // // TODO out - not helpful
                // let _script_state = ScriptState::Named {
                //     script: script_state.get_script().unwrap(),
                // };

                rustfmt(build_state)?;

                let result = gen_build_run(
                    options,
                    proc_flags,
                    build_state,
                    // &mut maybe_syntax_tree,
                    // &mut maybe_rs_manifest,
                    // &maybe_rs_source,
                    start,
                );
                if result.is_err() {
                    println!("{result:?}");
                }
                // disp_cmd_list();
                continue;
            }
            Err(err) => {
                owo_color_println!(
                    owo_resolve_style(MessageLevel::Error),
                    "Error parsing code: {}",
                    err
                );
            }
        }
    }

    // let mut line_editor = Reedline::create()
    //         .with_validator(Box::new(DefaultValidator))
    //         .with_hinter(Box::new(
    //             DefaultHinter::default()
    //                 .with_style(Style::new().italic().fg(Color::Cyan)),
    //         ))
    //         // .with_history(history)
    //         ;

    // let prompt = CustomPrompt("expr");
    // loop {
    //     println!(
    //         "{}{}\n{}",
    //         nu_ansi_term::Color::Cyan.paint("Enter an expression (e.g., 2 + 3), or "),
    //         nu_ansi_term::Color::Cyan.bold().paint("quit"),
    //         nu_ansi_term::Color::Cyan.paint(
    //             "Expressions in matching braces, brackets or quotes may span multiple lines."
    //         )
    //     );

    //     let sig = line_editor.read_line(&prompt).expect("Error reading line");
    //     let input: &str = match sig {
    //         Signal::Success(ref buffer) => buffer,
    //         Signal::CtrlD | Signal::CtrlC => {
    //             // println!("quit");
    //             break;
    //         }
    //     };
    //     // Process user input (line)

    //     let str = input.trim();
    //     let x = str.to_lowercase();
    //     if x == "q" || x == "quit" {
    //         break;
    //     }
    // }
    Ok(Some("quit".to_string()))
}

/// Display file listing

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn list(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    let build_state = &context.build_state;
    let source_path = &build_state.source_path;
    if source_path.exists() {
        println!("File: {:?}", &source_path);
    }

    // Display directory contents
    display_dir_contents(&build_state.target_dir_path)?;

    // Check if neither file nor directory exist
    if !&source_path.exists() && !&build_state.target_dir_path.exists() {
        println!("No temporary files found");
    }
    Ok(Some(String::from("End of list")))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn quit(_args: ArgMatches, _context: &mut Context) -> Result<Option<String>, BuildRunError> {
    println!("Done");
    std::process::exit(0);
}

fn debug_print_config() {
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    debug!("REPL_SUBDIR={REPL_SUBDIR}");
}

fn gen_build_run(
    options: &&mut cmd_args::Opt,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    // maybe_syntax_tree: &mut Option<File>,
    // maybe_rs_manifest: &mut Option<CargoManifest>,
    // maybe_rs_source: &Option<String>,
    // script_state: &ScriptState,
    start: &Instant,
) -> Result<(), Box<dyn Error>> {
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);
    let proc_flags = &proc_flags;
    let options = &options;

    // let mut maybe_rs_manifest = None;
    // let mut maybe_rs_source = None;
    // let mut maybe_syntax_tree: Option<File> = None;
    if build_state.must_gen {
        let (rs_manif, rs_source): (CargoManifest, String) =
            parse_source(&build_state.source_path)?;
        let mut maybe_rs_manifest = Some(rs_manif);
        let maybe_rs_source = Some(rs_source.clone());
        println!("&&&&&&&& rs_source={rs_source}");
        let maybe_syntax_tree = code_utils::to_ast(&rs_source);
        let borrowed_syntax_tree = maybe_syntax_tree.as_ref();
        let borrowed_rs_manifest = maybe_rs_manifest.as_mut();
        let borrowed_rs_source = maybe_rs_source.as_ref();
        if let Some(rs_manifest_ref) = borrowed_rs_manifest {
            build_state.cargo_manifest = Some(manifest::merge_manifest(
                build_state,
                borrowed_syntax_tree,
                borrowed_rs_source,
                rs_manifest_ref,
            )?);
        }
        // }
        // } else {
        //     build_state.cargo_manifest = Some(default_manifest(build_state)?);
        // }

        // debug!(
        //     "maybe_syntax_tree.is_some()={}",
        //     maybe_syntax_tree.is_some()
        // );
        // debug!(
        //     "maybe_rs_manifest.is_some()={}",
        //     maybe_rs_manifest.is_some()
        // );
        // debug!("maybe_rs_source.is_some()={}", maybe_rs_source.is_some());
        // debug!("build_state={build_state:#?}");
        // if build_state.must_gen {
        //     let borrowed_syntax_tree = maybe_syntax_tree.as_mut();
        //     let borrowed_rs_manifest = maybe_rs_manifest.as_mut();
        //     if let Some(syntax_tree_mut) = borrowed_syntax_tree {
        //         println!("syntax_tree_mut is Some");
        //         if let Some(rs_manifest_ref) = borrowed_rs_manifest {
        //             println!("rs_manifest_ref is Some");
        //             build_state.cargo_manifest = Some(manifest::merge_manifest(
        //                 build_state,
        //                 syntax_tree_mut,
        //                 rs_manifest_ref,
        //             )?);

        let has_main = if let Some(syntax_tree_ref) = borrowed_syntax_tree {
            code_utils::has_main(syntax_tree_ref)
        } else {
            code_utils::has_main_alt(borrowed_rs_source.ok_or("Missing source")?)
        };
        let borrowed_rs_source = maybe_rs_source.as_ref();
        if let Some(rs_source_ref) = borrowed_rs_source {
            println!("rs_source_ref is Some");
            if has_main {
                generate(build_state, rs_source_ref, proc_flags)?;
            } else {
                if verbose {
                    println!("Source does not contain fn main(), thus a snippet");
                }
                generate(build_state, &wrap_snippet(rs_source_ref), proc_flags)?;
            }
        }
    } else {
        println!("Skipping unnecessary generation step. Use --force (-f) to override.");
        // build_state.cargo_manifest = Some(default_manifest(build_state)?);
        build_state.cargo_manifest = None; // Don't need it in memory, build will find it on disk
    }
    if build_state.must_build {
        build(proc_flags, build_state)?;
    } else {
        println!("Skipping unnecessary build step. Use --force (-f) to override.");
    }
    if proc_flags.contains(ProcFlags::RUN) {
        run(proc_flags, &options.args, build_state)?;
    }
    let process = &format!(
        "{PACKAGE_NAME} completed processing script {}",
        build_state.source_name
    );
    display_timings(start, process, proc_flags);
    Ok(())
}

fn generate(
    build_state: &BuildState,
    rs_source: &String,
    proc_flags: &ProcFlags,
) -> Result<(), Box<dyn Error>> {
    let start_gen = Instant::now();
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("In generate, proc_flags={proc_flags}");

    fs::create_dir_all(&build_state.target_dir_path)?;

    let target_rs_path = build_state.target_dir_path.clone();
    let target_rs_path = target_rs_path.join(&build_state.source_name);
    if verbose {
        println!("GGGGGGGG Creating source file: {target_rs_path:?}");
    }
    let _target_rs_file = write_source(target_rs_path, rs_source)?;

    debug!("cargo_toml_path will be {:?}", &build_state.cargo_toml_path);
    if !Path::try_exists(&build_state.cargo_toml_path)? {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&build_state.cargo_toml_path)?;
    }
    // debug!("cargo_toml: {cargo_toml:?}");

    let cargo_manifest_str: &str = &build_state.cargo_manifest.as_ref().unwrap().to_string();

    debug!(
        "cargo_manifest_str: {}",
        code_utils::disentangle(cargo_manifest_str)
    );

    let mut toml_file = fs::File::create(&build_state.cargo_toml_path)?;
    toml_file.write_all(cargo_manifest_str.as_bytes())?;
    debug!("cargo_toml_path={:?}", &build_state.cargo_toml_path);
    debug!("##### Cargo.toml generation succeeded!");

    display_timings(&start_gen, "Completed generation", proc_flags);

    Ok(())
}

// Configure log level
fn configure_log() {
    let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    let mut binding = Builder::new();
    let builder = binding.parse_env(env);
    builder.write_style(WriteStyle::Always);
    builder.init();

    // Builder::new().filter_level(log::LevelFilter::Debug).init();
}

/// Build the Rust program using Cargo (with manifest path)
fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> Result<(), BuildRunError> {
    let start_build = Instant::now();
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("BBBBBBBB In build");

    let Ok(cargo_toml_path_str) = code_utils::path_to_str(&build_state.cargo_toml_path) else {
        return Err(BuildRunError::OsString(
            build_state.cargo_toml_path.clone().into_os_string(),
        ));
    };
    let mut build_command = Command::new("cargo");
    // Rustc writes to std
    let args = vec!["build", "--manifest-path", &cargo_toml_path_str];
    // if verbose {
    //     args.push("--verbose");
    // };

    build_command.args(&args); // .current_dir(build_dir);

    // Show sign of life in case build takes a while
    eprintln!("Building...");

    // Redirect stdout and stderr to inherit from the parent process (terminal)
    build_command
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    // Execute the command and handle the result
    let output = build_command
        .spawn()
        .expect("failed to spawn cargo build process");

    // Wait for the process to finish
    let exit_status = output
        .wait_with_output()
        .expect("failed to wait on cargo build");

    if exit_status.status.success() {
        debug!("Build succeeded");
    } else {
        return Err(BuildRunError::Command(String::from("Build failed")));
    };

    display_timings(&start_build, "Completed build", proc_flags);

    Ok(())
}

// Run the built program
fn run(
    proc_flags: &ProcFlags,
    args: &[String],
    build_state: &BuildState,
) -> Result<(), BuildRunError> {
    let start_run = Instant::now();
    debug!("RRRRRRRR In run");

    // debug!("BuildState={build_state:#?}");
    let target_path = build_state.target_path.clone();
    // debug!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", target_path.display()));
    run_command.args(args);

    debug!("Run command is {run_command:?}");

    // Sandwich command between two lines of dashes in the terminal
    let dash_line = "-".repeat(50);
    println!("{dash_line}");

    let _exit_status = run_command.spawn()?.wait()?;

    let dash_line = "-".repeat(50);
    println!("{dash_line}");

    // debug!("Exit status={exit_status:#?}");

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
