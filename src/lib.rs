use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;

// Re-export the modules you need
pub mod builder;
pub mod cmd_args;
pub mod code_utils;
pub mod colors;
pub mod config;
pub mod errors;
pub mod logging;
pub mod manifest;
pub mod repl;
pub mod shared;
pub mod stdin;

// Re-export commonly used items for convenience
pub use builder::{execute, gen_build_run};
pub use cmd_args::{get_args, get_proc_flags, validate_args, Cli, ProcFlags};
pub use code_utils::{
    create_temp_source_file, extract_ast_expr, extract_manifest, modified_since_compiled,
    process_expr,
};
pub use colors::{nu_resolve_style, MessageLevel};
pub use config::load;
pub use config::MAYBE_CONFIG;
pub use errors::ThagError;
pub use repl::run_repl;
pub use shared::{debug_timings, escape_path_for_windows, Ast, BuildState, ScriptState};
pub use stdin::{edit, read};

// Re-export specific items if they are defined in the respective modules
// pub use crate::{gen_build_run, BuildState, DYNAMIC_SUBDIR, REPL_SUBDIR, TEMP_SCRIPT_NAME, TMPDIR};
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

lazy_static! {
    #[derive(Debug)]
    pub static ref TMPDIR: PathBuf = env::temp_dir();
}
