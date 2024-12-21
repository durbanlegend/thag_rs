#![warn(clippy::pedantic)]
//-----------------------------------------------------------------------------
/// Core functionality (minimal feature)
/// Required for basic script operations
//-----------------------------------------------------------------------------
#[cfg(feature = "minimal")]
pub mod errors; // Error handling
#[cfg(feature = "minimal")]
pub mod logging; // Basic logging
#[cfg(feature = "minimal")]
pub mod profiling; // Performance profiling
#[cfg(feature = "minimal")]
pub mod shared; // Core shared utilities

//-----------------------------------------------------------------------------
/// Build system functionality
/// Handles script compilation and execution
//-----------------------------------------------------------------------------
#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub mod code_utils; // Shared build utilities

#[cfg(feature = "full_build")]
pub mod builder; // Script building
#[cfg(feature = "full_build")]
pub mod cmd_args; // Command-line argument handling
#[cfg(feature = "full_build")]
pub mod manifest; // Cargo.toml handling

//-----------------------------------------------------------------------------
/// UI and configuration
/// Terminal-based user interface components
//-----------------------------------------------------------------------------
#[cfg(feature = "color_support")]
pub mod colors; // Terminal color support
#[cfg(any(feature = "color_support", feature = "full_build"))]
pub mod config; // Configuration handling

#[cfg(feature = "tui")]
pub mod file_dialog; // File selection dialog
#[cfg(feature = "tui")]
pub mod stdin; // Standard input handling
#[cfg(feature = "tui")]
pub mod tui_editor; // Terminal UI editor

//-----------------------------------------------------------------------------
/// REPL functionality
/// Interactive command execution
//-----------------------------------------------------------------------------
#[cfg(feature = "repl")]
pub mod repl; // REPL implementation

#[cfg(any(feature = "basic_build", feature = "full_build"))]
pub use code_utils::{find_crates, find_metadata, Ast, CratesFinder, MetadataFinder};

#[cfg(feature = "color_support")]
pub use {
    colors::{
        coloring, Ansi16DarkStyle, Ansi16LightStyle, ColorSupport, Lvl, MessageLevel, TermTheme,
        Xterm256DarkStyle, Xterm256LightStyle,
    },
    config::{
        load, maybe_config, Colors, Config, Context, Dependencies, FeatureOverride, Logging, Misc,
        ProcMacros,
    },
    termbg,
};

#[cfg(feature = "full_build")]
pub use {
    builder::{
        debug_timings, display_timings, execute, gen_build_run, process_expr, BuildState,
        ScriptState,
    },
    cmd_args::{get_args, get_proc_flags, validate_args, Cli, ProcFlags},
    code_utils::modified_since_compiled,
    manifest::extract,
};

#[cfg(feature = "minimal")]
pub use {
    errors::{ThagError, ThagResult},
    log,
    logging::{get_verbosity, Verbosity, V},
    profiling::Profile,
    shared::escape_path_for_windows,
    thag_proc_macros::repeat_dash,
};

#[cfg(feature = "tui")]
pub use {
    crokey::KeyCombination,
    tui_editor::{CrosstermEventReader, EventReader, KeyDisplayLine, MockEventReader},
};

use std::path::PathBuf;
use std::{env, sync::LazyLock};

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
