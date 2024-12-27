//! Introducing `thag_rs` (command `thag`) - a Swiss Army knife of productivity tools for Rust development.
//! //! `thag` combines a script runner, expression evaluator, and REPL into one tool,
//! then adds an array of smart features.
//!
//! `thag`'s mission is to remove obstacles to productivity by giving you a selection of tools
//! and examples to make it as quick and easy as possible to figure stuff out without tedious setup.
//!
//! 🚀 **Core Powers:**
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
//! - Shebang support for true scripting (but you can do better: read on...)
//!
//! - Loop-filter mode for data processing
//!
//! 🎯 **Smart Features:**
//!
//! - Toml-free by default: dependency inference from imports and Rust paths (`x::y::z`)
//!
//! - You're in control: dependency inference (max/min/config/none) and/or toml block
//!
//! - Beyond shebangs: build instant commands from your snippets and programs
//!
//! - Execute scripts directly from URLs (GitHub, GitLab, BitBucket, Rust Playground)
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
//! (Yes, you can even include unit tests in your scripts)
//!
//! - View macro expansions side-by-side with your base script
//!
//! - Proc macro development support, including proc macro starter kit and an "intercept-and-debug" option to show an expanded view of your proc macro
//!
//! - Automated inclusion of `derive` or other dependency features
//!
//! 💡 **Getting Started:**
//!
//! Jump into `thag`'s collection of 230+ sample scripts in [demo/README.md](https://github.com/durbanlegend/thag_rs/blob/master/demo/README.md) to see what's possible. Got a cool script to share? We'd love to see it (under MIT/Apache 2 license)!
//!
//! Whether you're prototyping, learning, or building tools, `thag_rs` adapts to your style - from quick one-liners to full-featured programs.
//!
//! ## Feature flags
//!  `thag_rs` is a full-featured binary, but it is also a library so that you can call `thag_rs`
//!  functionality from your code. When you do so, the script build time can be greatly reduced
//!  by only specifying the features you need. See the demo script library for examples.
#![doc = document_features::document_features!()]
#![warn(clippy::pedantic)]
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
/// Core: Performance profiling
#[cfg(feature = "core")]
pub mod profiling;
/// Core: Shared functionality
#[cfg(feature = "core")]
pub mod shared;

//-----------------------------------------------------------------------------
// AST Analysis:
//-----------------------------------------------------------------------------
#[cfg(any(feature = "ast", feature = "build"))]
/// Abstract Syntax Tree parsing and dependency inference
pub mod ast;
/// Operations on code
#[cfg(any(feature = "ast", feature = "build"))]
pub mod code_utils;

//-----------------------------------------------------------------------------
// Build System
//-----------------------------------------------------------------------------
/// Script building and execution
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
/// Message coloring tailored to terminal capabilities and current theme
#[cfg(feature = "color_support")]
pub mod colors;
/// Configuration loader
#[cfg(any(feature = "color_support", feature = "build"))]
pub mod config;
#[cfg(feature = "color_support")]
pub mod log_color; // Alternative lightweight logging

/// TUI file dialog
#[cfg(feature = "tui")]
pub mod file_dialog;
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

// #[cfg(feature = "core")]
pub use {
    errors::{ThagError, ThagResult},
    logging::{get_verbosity, Verbosity, V},
    profiling::Profile,
    shared::{debug_timings, escape_path_for_windows, get_home_dir, get_home_dir_string},
    thag_proc_macros::repeat_dash,
};

#[cfg(any(feature = "ast", feature = "build"))]
pub use {
    ast::{find_crates, find_metadata, Ast, CratesFinder, MetadataFinder},
    code_utils::to_ast,
};

#[cfg(feature = "build")]
pub use {
    builder::{display_timings, execute, gen_build_run, process_expr, BuildState, ScriptState},
    cmd_args::{get_args, get_proc_flags, validate_args, Cli, ProcFlags},
    code_utils::modified_since_compiled,
    manifest::extract,
};

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
