#![allow(clippy::uninlined_format_args)]
use env_logger::{Builder, Env, WriteStyle};

use log::{debug, log_enabled, Level::Debug};
use rs_script::builder::gen_build_run;
use rs_script::cmd_args::{get_opt, get_proc_flags, Cli, ProcFlags};
use rs_script::code_utils::{self, extract_ast, extract_manifest};
use rs_script::errors::BuildRunError;
use rs_script::nu_color_println;
use rs_script::repl;
use rs_script::shared::{debug_timings, Ast, BuildState, ScriptState};
use rs_script::stdin;
use rs_script::term_colors::{nu_resolve_style, MessageLevel};
use rs_script::{
    DYNAMIC_SUBDIR, PACKAGE_NAME, REPL_SUBDIR, RS_SUFFIX, TEMP_SCRIPT_NAME, TMPDIR, VERSION,
};


use std::error::Error;

use std::path::{Path, PathBuf};

use std::time::Instant;


//      TODO:
//       1.  Consider supporting alternative TOML embedding keywords so we can run examples/regex_capture_toml.rs.
//       2.  Consider history support for stdin.
//       3.  Paste event in Windows slow or not happening?
//       4.  TUI editor as an option on stdin.
//       5.  How to navigate reedline history entry by entry instead of line by line.
//       6.  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
//       8.  Cat files before delete.
//       9.  DONE Consider making script name optional, with -s/stdin parm as per my runner changes?
//      10.  Decide if it's worth passing the wrapped syntax tree to gen_build_run from eval just to avoid
//           re-parsing it for that specific use case.
//      11.  Clean up debugging
//      12.  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
//      13.  Clap aliases not working in REPL.
//      14.  Get rid of date and time in RHS of REPL? - doesn't seem to be an option.
//      15.  Help command in eval, same as quit and q
//      16.  Work on examples/reedline_clap_repl_gemini.rs
//      17.
//      18.  How to set editor in Windows.
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
    // println!("Temporary directory: {:?}", *TMP_DIR);

    let is_repl = proc_flags.contains(ProcFlags::REPL);

    let working_dir_path = if is_repl {
        TMPDIR.join(REPL_SUBDIR)
    } else {
        std::env::current_dir()?.canonicalize()?
    };

    validate_options(&options, &proc_flags)?;

    // Normal REPL with no named script
    let repl_source_path = if is_repl && options.script.is_none() {
        Some(code_utils::create_next_repl_file())
    } else {
        None
    };

    let is_expr = proc_flags.contains(ProcFlags::EXPR);
    let is_stdin = proc_flags.contains(ProcFlags::STDIN);
    let is_dynamic = is_expr | is_stdin;

    if is_dynamic {
        code_utils::create_temp_source_file();
    }

    // Reusable source path for expressions and stdin evaluation
    // let temp_source_path = if is_expr {
    //     Some(code_utils::create_temp_source_file())
    // } else {
    //     None
    // };

    let script_dir_path = if is_repl {
        if let Some(ref script) = options.script {
            // REPL with repeat of named script
            let source_stem = script
                .strip_suffix(RS_SUFFIX)
                .expect("Failed to strip extension off the script name");
            working_dir_path.join(source_stem)
        } else {
            // Normal REPL with no script name
            repl_source_path
                .as_ref()
                .expect("Missing path of newly created REPL souece file")
                .parent()
                .expect("Could not find parent directory of repl source file")
                .to_path_buf()
        }
    } else if is_dynamic {
        debug!("^^^^^^^^ is_dynamic={is_dynamic}");
        <std::path::PathBuf as std::convert::AsRef<Path>>::as_ref(&TMPDIR)
            .join(DYNAMIC_SUBDIR)
            .clone()
    } else {
        // Normal script file prepared beforehand
        let script = options
            .script
            .clone()
            .expect("Problem resolving script path");
        let script_path = PathBuf::from(script);
        let script_dir_path = script_path
            .parent()
            .expect("Problem resolving script parent path");
        // working_dir_path.join(PathBuf::from(script.clone()))
        script_dir_path
            .canonicalize()
            .expect("Problem resolving script dir path")
    };

    let script_state: ScriptState = if let Some(ref script) = options.script {
        let script = script.to_owned();
        ScriptState::Named {
            script,
            script_dir_path,
        }
    } else if is_repl {
        let script = repl_source_path
            .expect("Missing newly created REPL source path")
            .display()
            .to_string();
        ScriptState::NamedEmpty {
            script,
            script_dir_path,
        }
    } else {
        assert!(is_dynamic);
        ScriptState::NamedEmpty {
            script: String::from(TEMP_SCRIPT_NAME),
            script_dir_path,
        }
    };

    // debug!("script_state={script_state:?}");

    let mut build_state = BuildState::pre_configure(&proc_flags, &options, &script_state)?;
    if is_repl {
        debug!("build_state.source_path={:?}", build_state.source_path);
    }

    if is_repl {
        repl::run_repl(&mut options, &proc_flags, &mut build_state, start)?;
    } else if is_dynamic {
        let rs_source = if is_expr {
            let Some(rs_source) = options.expression.clone() else {
                return Err(Box::new(BuildRunError::Command(
                    "Missing expression for --expr option".to_string(),
                )));
            };
            rs_source
        } else {
            assert!(is_stdin);
            debug!("About to call read_stdin()");
            let vec = stdin::read_stdin()?;
            debug!("vec={vec:#?}");
            vec.join("\n")
        };
        let rs_manifest = extract_manifest(&rs_source, Instant::now())
            .map_err(|_err| BuildRunError::FromStr("Error parsing rs_source".to_string()))?;
        build_state.rs_manifest = Some(rs_manifest);

        let maybe_ast = extract_ast(&rs_source);

        if let Ok(expr_ast) = maybe_ast {
            code_utils::process_expr(
                &expr_ast,
                &mut build_state,
                &rs_source,
                &mut options,
                &proc_flags,
                &start,
            )?;
        } else {
            nu_color_println!(
                nu_resolve_style(MessageLevel::Error),
                "Error parsing code: {:#?}",
                maybe_ast
            );
        }
    } else {
        gen_build_run(
            &mut options,
            &proc_flags,
            &mut build_state,
            None::<Ast>,
            &start,
        )?;
    }

    Ok(())
}

fn validate_options(options: &Cli, proc_flags: &ProcFlags) -> Result<(), Box<dyn Error>> {
    if let Some(ref script) = options.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name must end in {RS_SUFFIX}"
            ))));
        }
        if proc_flags.contains(ProcFlags::EXPR) {
            return Err(Box::new(BuildRunError::Command(
                "Incompatible options: --expr option and script name".to_string(),
            )));
        }
    } else if !proc_flags.contains(ProcFlags::EXPR)
        && !proc_flags.contains(ProcFlags::REPL)
        && !proc_flags.contains(ProcFlags::STDIN)
    {
        return Err(Box::new(BuildRunError::Command(
            "Missing script name".to_string(),
        )));
    }
    Ok(())
}

fn debug_print_config() {
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    debug!("REPL_SUBDIR={REPL_SUBDIR}");
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
