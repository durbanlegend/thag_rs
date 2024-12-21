#![warn(clippy::pedantic)]
use std::path::PathBuf;
use std::{env, sync::LazyLock};

// Re-export the modules
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub mod builder;
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub mod cmd_args;
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub mod code_utils;
#[cfg(feature = "color_support")]
pub mod colors;
#[cfg(any(
    feature = "basic_color",
    feature = "color_support",
    feature = "full_build"
))]
pub mod config;
#[cfg(feature = "minimal")]
pub mod errors;
#[cfg(feature = "tui")]
pub mod file_dialog;
#[cfg(feature = "minimal")]
pub mod logging;
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub mod manifest;
#[cfg(feature = "minimal")]
pub mod profiling;
#[cfg(feature = "repl")]
pub mod repl;
#[cfg(feature = "minimal")]
pub mod shared;
#[cfg(feature = "tui")]
pub mod stdin;
// Specify `pub mod termbg;` or `pub use termbg;` depending whether we want to use our termbg module or the `termbg crate`.
// Crate should be good as from 0.6.1.
// pub mod termbg;
#[cfg(feature = "color_support")]
pub use termbg;
#[cfg(feature = "tui")]
pub mod tui_editor;

// Re-export commonly used items for convenience
#[cfg(any(feature = "basic_build", feature = "full_build"))]
#[cfg(debug_assertions)]
pub use builder::debug_timings;
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub use builder::{display_timings, execute, gen_build_run, process_expr, BuildState, ScriptState};
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub use cmd_args::{get_args, get_proc_flags, validate_args, Cli, ProcFlags};
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub use code_utils::{create_temp_source_file, extract_ast_expr, modified_since_compiled, Ast};
#[cfg(any(feature = "basic_color", feature = "color_support"))]
pub use colors::{
    coloring, Ansi16DarkStyle, Ansi16LightStyle, ColorSupport, Lvl, MessageLevel, TermTheme,
    Xterm256DarkStyle, Xterm256LightStyle,
};
#[cfg(any(feature = "basic_color", feature = "color_support"))]
pub use config::{
    load, maybe_config, Colors, Config, Context, Dependencies, FeatureOverride, Logging, Misc,
    ProcMacros,
};
#[cfg(feature = "tui")]
pub use crokey::KeyCombination;
#[cfg(feature = "minimal")]
pub use errors::{ThagError, ThagResult};
#[cfg(feature = "minimal")]
pub use log;
// #[cfg(any(feature = "simplelog", feature = "env_logger"))]
#[cfg(feature = "minimal")]
pub use logging::{get_verbosity, Verbosity, V};
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub use manifest::extract;
#[cfg(feature = "minimal")]
pub use profiling::Profile;
pub use shared::disentangle;
#[cfg(feature = "minimal")]
pub use shared::escape_path_for_windows;
#[cfg(feature = "minimal")]
pub use thag_proc_macros::repeat_dash;
#[cfg(feature = "tui")]
pub use tui_editor::{CrosstermEventReader, EventReader, KeyDisplayLine, MockEventReader};

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

// pub mod __private {
//     pub use crokey::key;
//     pub use crossterm;
//     pub use strict::OneToThree;

//     pub use crate::KeyCombination;
//     use crossterm::event::KeyModifiers;
//     pub const MODS: KeyModifiers = KeyModifiers::NONE;
//     pub const MODS_CTRL: KeyModifiers = KeyModifiers::CONTROL;
//     pub const MODS_ALT: KeyModifiers = KeyModifiers::ALT;
//     pub const MODS_SHIFT: KeyModifiers = KeyModifiers::SHIFT;
//     pub const MODS_CTRL_ALT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::ALT);
//     pub const MODS_ALT_SHIFT: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::SHIFT);
//     pub const MODS_CTRL_SHIFT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);
//     pub const MODS_CTRL_ALT_SHIFT: KeyModifiers = KeyModifiers::CONTROL
//         .union(KeyModifiers::ALT)
//         .union(KeyModifiers::SHIFT);
// }
