//! Introducing `thag_rs` (command `thag`) - a set of creative solutions to enhance your Rust development experience.
//!
//! `thag` combines a script runner, expression evaluator, and REPL into a single command, with an array of smart features.
//!
//! `thag`'s mission is to remove obstacles to productivity by giving you a selection of tools
//! and examples to make it as quick and easy as possible to explore coding solutions without tedious setup.
//!
//! 🧱 **Core Features:**
//!
//! - Run Rust code straight from the command line
//!
//! - Evaluate expressions on the fly
//!
//! - Interactive REPL mode for rapid prototyping
//!
//! - Uses AST analysis to understand your code
//!
//! - Optionally embed custom Cargo manifest settings in "toml block" comments
//!
//! - Shebang support for true scripting (but `thag`'s command building is much better still!)
//!
//! - Loop-filter mode for data processing
//!
//! 🕶️ **Smart Features:**
//!
//! - Toml-free by default: dependency inference from imports and Rust paths (`x::y::z`)
//!
//! - You're in control: dependency inference (max/min/config/none) and/or toml block
//!
//! - Beyond shebangs: build instant commands from your snippets and programs
//!
//! - Execute scripts directly from URLs (`GitHub`, `GitLab`, `BitBucket`, `Rust Playground`)
//!
//! - Paste-and-run with built-in TUI editor
//!
//! - An evolution path for your code from REPL to edit-submit loop to saved scripts
//!
//! - Edit-submit standard input
//!
//! - Integrate your favourite editor (VS Code, Helix, Zed, vim, nano etc.)
//!
//! - Run any Cargo command (clippy, tree, test) against your scripts.
//!   (Yes, you can even include unit tests in your scripts)
//!
//! - View macro expansions side-by-side with your base script
//!
//! - Proc macro development support, including proc macro starter kit and a `maybe_expand...` option to show an expanded view of your proc macro
//!
//! - Automated, configurable inclusion of `derive` or other features of specific dependencies
//!
//! 💡 **Getting Started:**
//!
//! Jump into `thag`'s collection of 330+ sample scripts in [demo/README.md](https://github.com/durbanlegend/thag_rs/blob/master/demo/README.md) to see what's possible.
//! Contributions will be considered under MIT/Apache 2 license.
//!
//! Whether you're prototyping, learning, or building tools, `thag_rs` adapts to your style - from quick one-liners to full-featured programs.
//!
//! ## Feature flags
//!  `thag_rs` is a full-featured binary, but it is also a library so that you can call `thag_rs`
//!  functionality from your code. When you do so, the script build time can be greatly reduced
//!  by only specifying the features you need. See the demo script library for examples.
//!
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
#![warn(clippy::pedantic, missing_docs)]
//-----------------------------------------------------------------------------
// Core functionality (core feature):
// Required for basic script operations
//-----------------------------------------------------------------------------
/// Core: Error handling
#[cfg(feature = "core")]
pub mod errors;

/// Core: Basic logging
#[cfg(feature = "core")]
pub mod logging;

//-----------------------------------------------------------------------------
// AST Analysis:
//-----------------------------------------------------------------------------
#[cfg(any(feature = "ast", feature = "build"))]
pub mod ast;
/// Operations on code
#[cfg(any(feature = "ast", feature = "build"))]
pub mod code_utils;

//-----------------------------------------------------------------------------
// Build System
//-----------------------------------------------------------------------------
#[cfg(feature = "build")]
pub mod builder;
/// Command-line argument and processing flags handling
#[cfg(feature = "build")]
pub mod cmd_args;
/// Manifest processing and Cargo.toml generation for the script
#[cfg(feature = "build")]
pub mod manifest;

//-----------------------------------------------------------------------------
// UI and configuration:
// Terminal-based user interface components
//-----------------------------------------------------------------------------
/// Configuration loader
#[cfg(feature = "config")]
pub use thag_common::config;

/// TUI file dialog
#[cfg(feature = "tui")]
pub mod file_dialog;
#[cfg(feature = "tui")]
/// TUI key handling and combinations
pub mod keys;
/// Paste-and-run and standard input handling
#[cfg(feature = "tui")]
pub mod stdin;
/// TUI editor for paste-and-run, stdin processing and REPL expression promotion.
#[cfg(feature = "tui")]
pub mod tui_editor;

//-----------------------------------------------------------------------------
// REPL functionality:
// Interactive command execution
//-----------------------------------------------------------------------------
/// REPL implementation
#[cfg(feature = "repl")]
pub mod repl;

//-----------------------------------------------------------------------------
// Tools
//-----------------------------------------------------------------------------
/// Error types for command-line tools
#[cfg(feature = "tools")]
pub mod tool_errors;

#[cfg(feature = "core")]
pub use {
    errors::{ThagError, ThagResult},
    log, // re-export log crate for debug_log
    thag_common::{
        debug_log, debug_timings, escape_path_for_windows, get_home_dir, get_home_dir_string,
        get_verbosity, init_verbosity, lazy_static_var, re, set_global_verbosity, set_verbosity,
        set_verbosity_from_env, static_lazy, thousands, vprtln, ColorSupport, TermBgLuma,
        Verbosity, OUTPUT_MANAGER, V,
    },
    thag_styling::{
        display_theme_details, display_theme_roles, find_closest_color, paint_for_role, sprtln,
        svprtln, AnsiStyleExt, Color, ColorInfo, ColorInitStrategy, ColorValue, HowInitialized,
        PaletteConfig, Role, Style, Styleable, Styled, StyledPrint, StyledString, Styler, Theme,
    },
};

#[cfg(feature = "tools")]
pub use thag_styling::themed_inquire_config;

pub use thag_common::{auto_help, help_system};

pub use thag_proc_macros::{file_navigator, repeat_dash};

#[cfg(any(feature = "ast", feature = "build"))]
pub use {
    ast::{find_crates, find_metadata, Ast, CratesFinder, MetadataFinder},
    code_utils::to_ast,
};

#[cfg(feature = "build")]
pub use {
    builder::{display_timings, execute, gen_build_run, process_expr, BuildState, ScriptState},
    cmd_args::{get_args, get_proc_flags, set_verbosity, validate_args, Cli, ProcFlags},
    code_utils::modified_since_compiled,
    logging::configure_log,
    manifest::extract,
    ratatui::crossterm,
};

#[cfg(feature = "color_detect")]
pub use termbg;

#[cfg(feature = "color_detect")]
pub use thag_common::terminal;

#[cfg(all(feature = "color_detect", feature = "tools"))]
pub use thag_styling::inquire_theming;

#[cfg(feature = "config")]
pub use config::{
    load, maybe_config, Config, Context, Dependencies, FeatureOverride, Logging, Misc, ProcMacros,
    Styling,
};

#[cfg(feature = "tui")]
pub use {
    keys::KeyCombination,
    tui_editor::{CrosstermEventReader, EventReader, KeyDisplayLine, MockEventReader},
};

use std::path::PathBuf;
use std::{env, sync::LazyLock};

// Common constants and statics
/// Built-in crates that are always available in Rust
pub const BUILT_IN_CRATES: [&str; 7] = [
    "std",
    "core",
    "alloc",
    "collections",
    "fmt",
    "crate",
    "self",
];
/// Subdirectory name for dynamic/temporary Rust files
pub const DYNAMIC_SUBDIR: &str = "rs_dyn";
/// Subdirectory name for shared build target (all scripts share dependencies)
pub const SHARED_TARGET_SUBDIR: &str = "thag_rs_shared_target";
/// Subdirectory name for executable cache (stores built script executables)
pub const EXECUTABLE_CACHE_SUBDIR: &str = "thag_rs_bins";
/// Length of decorative flower box borders for output formatting
pub const FLOWER_BOX_LEN: usize = 70;
/// Package name from Cargo.toml
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
/// Default filename for REPL scripts
pub const REPL_SCRIPT_NAME: &str = "repl_script.rs";
/// Subdirectory name for REPL files
pub const REPL_SUBDIR: &str = "rs_repl";
/// Rust source file extension
pub const RS_SUFFIX: &str = ".rs";
/// Temporary directory name
pub const TEMP_DIR_NAME: &str = "temp";
/// Default filename for temporary scripts
pub const TEMP_SCRIPT_NAME: &str = "temp.rs";
/// Cargo manifest filename
pub const TOML_NAME: &str = "Cargo.toml";
/// Package version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// System temporary directory path
pub static TMPDIR: LazyLock<PathBuf> = LazyLock::new(env::temp_dir);
/// Copied from `crokey` under MIT licence.
/// Copyright (c) 2022 Canop
///
#[doc(hidden)] // Makes it not appear in documentation
#[cfg(feature = "tui")]
pub mod __private {
    pub use ratatui::crossterm;
    pub use strict::OneToThree;
    pub use thag_proc_macros::key;

    pub use crate::KeyCombination;
    use ratatui::crossterm::event::KeyModifiers;
    /// No modifier keys pressed
    pub const MODS: KeyModifiers = KeyModifiers::NONE;
    /// Control key modifier
    pub const MODS_CTRL: KeyModifiers = KeyModifiers::CONTROL;
    /// Alt key modifier
    pub const MODS_ALT: KeyModifiers = KeyModifiers::ALT;
    /// Shift key modifier
    pub const MODS_SHIFT: KeyModifiers = KeyModifiers::SHIFT;
    /// Control and Alt key modifiers combined
    pub const MODS_CTRL_ALT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::ALT);
    /// Alt and Shift key modifiers combined
    pub const MODS_ALT_SHIFT: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::SHIFT);
    /// Control and Shift key modifiers combined
    pub const MODS_CTRL_SHIFT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);
    /// Control, Alt, and Shift key modifiers combined
    pub const MODS_CTRL_ALT_SHIFT: KeyModifiers = KeyModifiers::CONTROL
        .union(KeyModifiers::ALT)
        .union(KeyModifiers::SHIFT);
}
