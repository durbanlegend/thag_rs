use crate::cmd_args::{Opt, ProcFlags};
use crate::code_utils::{self, clean_up, display_dir_contents, extract_ast, extract_manifest, Ast};
use crate::errors::BuildRunError;
use crate::term_colors::nu_resolve_style;
use crate::{gen_build_run, nu_color_println, BuildState, MessageLevel};
use clap::Parser;
// use code_utils::Ast;
use log::debug;
// use quote::quote;
use reedline::{
    DefaultHinter, DefaultValidator, FileBackedHistory, Prompt, PromptEditMode,
    PromptHistorySearch, PromptHistorySearchStatus, Reedline, Signal,
};
use reedline_repl_rs::{
    clap::{ArgMatches, Command as ReplCommand},
    Repl,
};
use std::borrow::Cow;
use std::error::Error;
use std::time::Instant;
use strum::{EnumIter, EnumProperty, IntoEnumIterator, IntoStaticStr};

const HISTORY_FILE: &str = "rs_eval_hist.txt";
pub static DEFAULT_MULTILINE_INDICATOR: &str = "";

// Legacy enum, still useful but not sure if it still pays its way.
#[derive(Debug, Parser, EnumIter, EnumProperty, IntoStaticStr)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
#[strum(serialize_all = "kebab-case")]
enum LoopCommand {
    /// Enter/paste and evaluate a Rust expression. This is the convenient option to use for snippets or even short programs.
    Eval,
    /// Edit the Rust expression. Edit/run can also be used as an alternative to eval for longer snippets and programs.
    Edit,
    /// Edit the generated Cargo.toml
    Toml,
    /// Attempt to build and run the Rust expression
    Run,
    /// Delete all temporary files for this eval (see list)
    Delete,
    /// List temporary files
    List,
    /// Edit history
    History,
    /// Exit the REPL
    Quit,
}

#[derive(Debug)]
struct Context<'a> {
    options: &'a mut Opt,
    proc_flags: &'a ProcFlags,
    // cmd_list: String,
    build_state: &'a mut BuildState,
    start: &'a Instant,
}

pub struct EvalPrompt(&'static str);
impl Prompt for EvalPrompt {
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

pub(crate) fn run_repl(
    options: &mut Opt,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    start: Instant,
) -> Result<(), Box<dyn Error>> {
    let mut cmd_vec = LoopCommand::iter()
        .filter(|v| !matches!(v, LoopCommand::Eval))
        .map(<LoopCommand as Into<&'static str>>::into)
        .map(String::from)
        .collect::<Vec<String>>();
    cmd_vec.sort();
    let cmd_list = "eval or one of: ".to_owned() + &cmd_vec.join(", ") + " or help";
    #[allow(unused_variables)]
    // let outer_prompt = || {
    //     println!(
    //         "{}",
    //         nu_resolve_style(MessageLevel::OuterPrompt)
    //             .paint(format!("Enter {}", cmd_list))
    //     );
    // };
    // outer_prompt();
    let context = Context {
        options,
        proc_flags,
        build_state,
        start: &start,
    };
    let mut repl = Repl::new(context)
        .with_name("REPL")
        // .with_version("v0.1.0")
        .with_description(
            "REPL mode lets you type or paste a Rust expression to be evaluated.
Start by choosing the eval option and entering your expression. Expressions between matching braces,
brackets, parens or quotes may span multiple lines.
If valid, the expression will be converted into a Rust program, and built and run using Cargo.
Dependencies will be inferred from imports if possible using a Cargo search, but the overhead
of doing so can be avoided by placing them in Cargo.toml format at the top of the expression in a
comment block of the form
/*[toml]
[depedencies]
...
*/
From here they will be extracted to a dedicated Cargo.toml file.
In this case the whole expression must be enclosed in curly braces to include the TOML in the expression.
At any stage before exiting the REPL, or at least as long as your TMPDIR is not cleared, you can
go back and edit your expression or its generated Cargo.toml file and copy or save them from the
editor or their temporary disk locations.
Outside of the expression evaluator, use the tab key to show selections and to complete partial
matching selections.",
        )
        .with_banner(&format!(
            "{}",
            nu_resolve_style(MessageLevel::OuterPrompt)
                .paint(&format!("Enter {}", cmd_list)),
        ))
        .with_quick_completions(true)
        .with_partial_completions(true)
        // .with_on_after_command(display_banner)

        .with_command(
            ReplCommand::new("eval")
                .about("Enter/paste and evaluate a Rust expression.
This is the convenient option to use for snippets or even short programs.")
                .subcommand(ReplCommand::new("quit")),
            eval,
        )
        .with_command(
            ReplCommand::new("edit").about("Edit Rust expression in editor"),
            edit
        )
        .with_command(
            ReplCommand::new("run").about("Attempt to build and run Rust expression"),
            run_expr
        )
        .with_command(
            ReplCommand::new("toml").about("Edit generated Cargo.toml"),
            toml
        )
        .with_command(ReplCommand::new("list").about("List temporary files"), list)
            .with_command(
                ReplCommand::new("delete")
                    .about("Delete all temporary files for this eval (see list)"),
                delete,
            )
        .with_command(
            ReplCommand::new("quit").about("Exit the REPL"),
            // .aliases(["q", "exit"]), // don't work
            quit,
        )
        .with_command(ReplCommand::new("history").about("Edit history."), history)
        // .with_error_handler(|ref _err, _repl| Ok(()))
        .with_stop_on_ctrl_c(true);

    repl.run()?;
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
fn history(_args: ArgMatches, context: &mut Context) -> Result<Option<String>, BuildRunError> {
    edit::edit_file(context.build_state.working_dir_path.join(HISTORY_FILE))?;
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

    debug!("In run_expr: build_state={build_state:#?}");
    let result = gen_build_run(options, proc_flags, build_state, None::<Ast>, start);
    if result.is_err() {
        println!("{result:?}");
    }
    Ok(Some(String::from("End of run")))
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

    let history_file = build_state.cargo_home.join(HISTORY_FILE);
    let history = Box::new(
        FileBackedHistory::with_file(25, history_file)
            .expect("Error configuring history with file"),
    );

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(nu_resolve_style(MessageLevel::Ghost)),
        ))
        .with_history(history);

    let prompt = EvalPrompt("expr");

    disp_eval_banner();
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
        let lc = rs_source.to_lowercase();
        if lc == "q" || lc == "quit" {
            break;
        }

        let rs_manifest = extract_manifest(rs_source, Instant::now())
            .map_err(|_err| BuildRunError::FromStr("Error parsing rs_source".to_string()))?;
        build_state.rs_manifest = Some(rs_manifest);

        let maybe_ast = extract_ast(rs_source);

        if let Ok(expr_ast) = maybe_ast {
            code_utils::process_expr(
                &expr_ast,
                build_state,
                rs_source,
                options,
                proc_flags,
                start,
            )?;
        } else {
            nu_color_println!(
                nu_resolve_style(MessageLevel::Error),
                "Error parsing code: {maybe_ast:#?}"
            );
        }

        disp_eval_banner();
    }

    Ok(Some("Back in main REPL".to_string()))
}

fn disp_eval_banner() {
    nu_color_println!(
        nu_resolve_style(MessageLevel::InnerPrompt),
        r"Enter an expression (e.g., 2 + 3), or Ctrl-C/D to go back. Expressions in matching braces, brackets or quotes may span multiple lines.
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