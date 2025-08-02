#![allow(clippy::uninlined_format_args)]
use std::sync::atomic::{AtomicBool, Ordering};
use thag_profiler::profiled;

#[cfg(feature = "simplelog")]
use {
    crate::vprtln,
    crate::V,
    simplelog::{
        ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
    },
    std::fs::File,
    std::sync::Once,
};

#[cfg(feature = "env_logger")]
use env_logger::{Builder, Env};

static DEBUG_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "simplelog")]
static LOGGING_INIT: Once = Once::new();

/// Enables debug logging by setting the global debug flag to true.
#[allow(clippy::module_name_repetitions)]
#[profiled]
pub fn enable_debug_logging() {
    DEBUG_LOG_ENABLED.store(true, Ordering::SeqCst);
}

/// Returns whether debug logging is currently enabled.
#[profiled]
pub fn is_debug_logging_enabled() -> bool {
    DEBUG_LOG_ENABLED.load(Ordering::SeqCst)
}

/// Configure log level
#[cfg(feature = "env_logger")]
#[profiled]
pub fn configure_log() {
    use log::info;

    let env = Env::new().filter("RUST_LOG");
    eprintln!("env={env:?}");
    let builder = Builder::new().parse_env(env).init();
    eprintln!("builder={builder:?}");
    info!("Initialized env_logger");
}

/// Configure log level
///
/// # Panics
///
/// Panics if it can't create the log file app.log in the current working directory.
#[cfg(feature = "simplelog")]
#[profiled]
pub fn configure_log() {
    LOGGING_INIT.call_once(|| {
        configure_simplelog();
    });
    // info!("Initialized simplelog");  // interferes with testing
    vprtln!(V::V, "Initialized simplelog");
}

/// Configure log level
///
/// # Panics
///
/// Panics if it can't create the log file app.log in the current working directory.
#[cfg(feature = "simplelog")]
#[profiled]
fn configure_simplelog() {
    if let Err(e) = CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("app.log").unwrap(),
        ),
    ]) {
        // Logger already initialized, which is fine
        eprintln!("Logger already initialized: {}", e);
    }
}
