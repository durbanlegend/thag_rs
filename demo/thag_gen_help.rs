/*[toml]
[dependencies]
thag_common = { version = "1, thag-auto", features = ["color_detect"] }
*/
/// Minimal prospective `thag` tool to generate help text for a Rust source file from its Doc comments.
/// Very basic - for now the Rust source file's location is hard-coded.
///
/// Usage: `thag demo/thag_gen_help.rs
//# Purpose: demo and test `egui_extras` code block formatting
//# Categories: technique, tools
use std::{fs, path::PathBuf};
use thag_common::help_system::{self, HelpSystem};

let thag_dev_path = env::var("THAG_DEV_PATH")?;
let mut file_pat PathBuf::from(thag_dev_path);
file_path.push("src");
file_path.push("bin");
file_path.push("thag_md_view.rs");

let contents = fs::read_to_string(&file_path)?;

let help_system = HelpSystem::from_source(&contents);
eprintln!("{help_system}");
