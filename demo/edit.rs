/*[toml]
[dependencies]
#edit = { version = "0.1.5", features = ["better-path"] }
edit = "0.1.5"
*/

#[allow(unused_doc_comments)]
/// Published example from edit crate readme.
///
/// Will use the editor specified in VISUAL or EDITOR environment variable.
///
/// E.g. `EDITOR=vim thag_rs demo/edit.rs`
//# Purpose: Demo of edit crate to invoke preferred editor.
// eprintln!("VISUAL={:?}", std::env::var("VISUAL"));
use std::env;
use std::error::Error;
use std::io::Result;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

env::set_var("VISUAL", "cmd.exe /C type");
env::set_var("EDITOR", "cmd.exe /C type");

fn get_full_editor_cmd(s: String) -> Result<(PathBuf, Vec<String>)> {
    let (path, args) = string_to_cmd(s);
    match get_full_editor_path(&path) {
        Ok(result) => Ok((result, args)),
        Err(_) if path.exists() => Ok((path, args)),
        Err(_) => Err(std::io::Error::from(ErrorKind::NotFound))
    }
}

fn get_full_editor_path<T: AsRef<OsStr> + AsRef<Path>>(binary_name: T) -> Result<PathBuf> {
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            if dir.join(&binary_name).is_file() {
                return Ok(dir.join(&binary_name));
            }
        }
    }

    Err(std::io::Error::from(ErrorKind::NotFound))
}

fn string_to_cmd(s: String) -> (PathBuf, Vec<String>) {
    let mut args = s.split_ascii_whitespace();
    (
        args.next().unwrap().into(),
        args.map(String::from).collect(),
    )
}

#[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // GUI editors
    "code.cmd -n -w", "atom.exe -w", "subl.exe -w",
    // notepad++ does not block for input
    // Installed by default
    "notepad.exe",
    // Generic "file openers"
    "cmd.exe /C start",
];

static ENV_VARS: &[&str] = &["VISUAL", "EDITOR"];
let x = ENV_VARS
        .iter()
        .filter_map(env::var_os)
        .filter(|v| !v.is_empty())
        .filter_map(|v| v.into_string().ok()).next();
println!("x={x:?}");
let editor_cmd = ENV_VARS
        .iter()
        .filter_map(env::var_os)
        .filter(|v| !v.is_empty())
        .filter_map(|v| v.into_string().ok())
        .filter_map(|s| get_full_editor_cmd(s).ok())
        .next()
        .or_else(|| {
            HARDCODED_NAMES
                .iter()
                .map(|s| s.to_string())
                .filter_map(|s| get_full_editor_cmd(s).ok())
                .next()
        });
        // .ok_or_else(|| Error::from(ErrorKind::NotFound));
println!("editor_cmd={editor_cmd:?}");

let template = "Fill in the blank: Hello, _____!";
let edited = edit::edit(template)?;
println!("after editing: '{}'", edited);
// after editing: 'Fill in the blank: Hello, world!'
