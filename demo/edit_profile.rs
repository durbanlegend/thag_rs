/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling"] }
*/

#[allow(unused_doc_comments)]
/// Profiled version of published example from edit crate readme.
///
/// Will use the editor specified in VISUAL or EDITOR environment variable.
///
/// E.g. `EDITOR="zed --wait" thag demo/edit_profile.rs`
//# Purpose: Demo of edit crate to invoke preferred editor.
//# Categories: crates, profiling, technique
use std::env;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::io::Result;
use std::path::{Path, PathBuf};

use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn get_full_editor_cmd(s: String) -> Result<(PathBuf, Vec<String>)> {
    let (path, args) = string_to_cmd(s);
    match get_full_editor_path(&path) {
        Ok(result) => Ok((result, args)),
        Err(_) if path.exists() => Ok((path, args)),
        Err(_) => Err(std::io::Error::from(ErrorKind::NotFound)),
    }
}

#[profiled]
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

#[profiled]
fn string_to_cmd(s: String) -> (PathBuf, Vec<String>) {
    let mut args = s.split_ascii_whitespace();
    (
        args.next().unwrap().into(),
        args.map(String::from).collect(),
    )
}

// #[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // GUI editors
    "code.cmd -n -w",
    "atom.exe -w",
    "subl.exe -w",
    // notepad++ does not block for input
    // Installed by default
    "notepad.exe",
    // Generic "file openers"
    "cmd.exe /C start",
];

static ENV_VARS: &[&str] = &["VISUAL", "EDITOR"];

#[enable_profiling]
fn main() -> Result<()> {
    let editor = ENV_VARS
        .iter()
        .filter_map(env::var_os)
        .filter(|v| !v.is_empty())
        .filter_map(|v| v.into_string().ok())
        .next();
    println!("editor={editor:?}");
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
    Ok(())
}
