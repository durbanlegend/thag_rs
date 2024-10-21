#![warn(clippy::pedantic)]
use std::path::PathBuf;
use std::{env, sync::LazyLock};

// Re-export the modules
pub mod builder;
pub mod cmd_args;
pub mod code_utils;
pub mod colors;
pub mod config;
pub mod errors;
pub mod file_dialog;
pub mod keys;
pub mod logging;
pub mod manifest;
pub mod repl;
pub mod shared;
pub mod stdin;
// #[cfg(not(target_os = "windows"))]
pub mod termbg;
pub mod tui_editor;

// Re-export commonly used items for convenience
pub use builder::{execute, gen_build_run};
pub use cmd_args::{get_args, get_proc_flags, validate_args, Cli, ProcFlags};
pub use code_utils::{
    create_temp_source_file, extract_ast_expr, extract_manifest, modified_since_compiled,
    process_expr,
};
pub use colors::{
    Ansi16DarkStyle, Ansi16LightStyle, Lvl, MessageLevel, Xterm256DarkStyle, Xterm256LightStyle,
};
pub use config::{load, maybe_config};
pub use errors::{ThagError, ThagResult};
pub use keys::KeyCombination;
pub use shared::{debug_timings, escape_path_for_windows, Ast, BuildState, ScriptState};
pub use termbg;
pub use thag_proc_macros::{DeriveCustomModel, IntoStringHashMap};

// Common constants and statics
pub const DYNAMIC_SUBDIR: &str = "rs_dyn";
pub const FLOWER_BOX_LEN: usize = 70;
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const REPL_SCRIPT_NAME: &str = "repl.rs";
pub const REPL_SUBDIR: &str = "rs_repl";
pub const RS_SUFFIX: &str = ".rs";
pub const TEMP_DIR_NAME: &str = "temp";
pub const TEMP_SCRIPT_NAME: &str = "temp.rs";
pub const TOML_NAME: &str = "Cargo.toml";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub static TMPDIR: LazyLock<PathBuf> = LazyLock::new(env::temp_dir);

pub mod __private {
    pub use crossterm;
    pub use strict::OneToThree;
    pub use thag_proc_macros::key;

    pub use crate::KeyCombination;
    use crossterm::event::KeyModifiers;
    pub const MODS: KeyModifiers = KeyModifiers::NONE;
    pub const MODS_CTRL: KeyModifiers = KeyModifiers::CONTROL;
    pub const MODS_ALT: KeyModifiers = KeyModifiers::ALT;
    pub const MODS_SHIFT: KeyModifiers = KeyModifiers::SHIFT;
    pub const MODS_CTRL_ALT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::ALT);
    pub const MODS_ALT_SHIFT: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::SHIFT);
    pub const MODS_CTRL_SHIFT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);
    pub const MODS_CTRL_ALT_SHIFT: KeyModifiers = KeyModifiers::CONTROL
        .union(KeyModifiers::ALT)
        .union(KeyModifiers::SHIFT);
}
