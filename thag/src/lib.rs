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
// Specify `pub mod termbg;` or `pub use termbg;` depending whether we want to use our termbg module or the `termbg crate`.
// Crate should be good as from 0.6.1.
// pub mod termbg;
pub use termbg;

pub mod tui_editor;

// Re-export commonly used items for convenience
pub use builder::{execute, gen_build_run, process_expr};
pub use cmd_args::{get_args, get_proc_flags, validate_args, Cli, ProcFlags};
pub use code_utils::{
    create_temp_source_file, disentangle, extract_ast_expr, extract_manifest,
    modified_since_compiled,
};
pub use colors::{
    coloring, Ansi16DarkStyle, Ansi16LightStyle, ColorSupport, Lvl, MessageLevel, TermTheme,
    Xterm256DarkStyle, Xterm256LightStyle,
};
pub use config::{
    load, maybe_config, Colors, Config, Context, Dependencies, FeatureOverride, Logging, Misc,
    ProcMacros,
};
pub use errors::{BuildError, BuildResult};
pub use keys::KeyCombination;
pub use log;
pub use logging::{get_verbosity, Verbosity, V};
#[cfg(debug_assertions)]
pub use shared::debug_timings;
pub use shared::{
    display_timings, escape_path_for_windows, Ast, BuildState, CrosstermEventReader, EventReader,
    KeyDisplayLine, MockEventReader, ScriptState,
};
pub use thag_proc_macros::repeat_dash;

// Common constants and statics
pub const BUILT_IN_CRATES: [&str; 7] = [
    "std",
    "core",
    "alloc",
    "collections",
    "fmt",
    "crate",
    "self",
];
pub const DYNAMIC_SUBDIR: &str = "rs_dyn";
pub const FLOWER_BOX_LEN: usize = 70;
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const REPL_SCRIPT_NAME: &str = "repl_script.rs";
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
