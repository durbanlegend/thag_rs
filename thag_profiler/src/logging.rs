use crate::static_lazy;
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

static_lazy! {
    DebugLogger: Option<Mutex<BufWriter<File>>> = {
        // Check if debug logging is enabled via environment variable
        if std::env::var("THAG_PROFILER_DEBUG").is_ok() {
            // Get the platform-specific log path
            if let Some(path) = get_platform_log_path() {
                // Try to open the file with a buffered writer
                if let Ok(file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    // Wrap the file in a buffered writer with a reasonable buffer size
                    // and protect with a mutex for thread safety
                    return Some(Mutex::new(BufWriter::with_capacity(8192, file)));
                }
            }
        }
        None
    }
}

#[must_use]
pub fn get_platform_log_path() -> Option<PathBuf> {
    // Platform-specific paths
    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let mut path = PathBuf::from(local_app_data);
            path.push("thag_profiler");
            std::fs::create_dir_all(&path).ok()?;
            path.push("debug.log");
            return Some(path);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push("Library");
            path.push("Logs");
            path.push("thag_profiler");
            std::fs::create_dir_all(&path).ok()?;
            path.push("debug.log");
            return Some(path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
            let mut path = PathBuf::from(xdg_data_home);
            path.push("thag_profiler");
            path.push("logs");
            std::fs::create_dir_all(&path).ok()?;
            path.push("debug.log");
            return Some(path);
        } else if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".local");
            path.push("share");
            path.push("thag_profiler");
            path.push("logs");
            std::fs::create_dir_all(&path).ok()?;
            path.push("debug.log");
            return Some(path);
        }
    }

    // Fallback to temporary directory
    let mut path = std::env::temp_dir();
    path.push("thag_profiler");
    std::fs::create_dir_all(&path).ok()?;
    path.push("debug.log");
    Some(path)
}

// Define a function to flush the log buffer - can be called at strategic points
pub fn flush_debug_log() {
    if let Some(logger) = DebugLogger::get() {
        // let mut writer = logger.lock();
        let _ = logger.lock().flush();
    }
}

// Improved debug_log macro that uses the lazy static
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::DebugLogger::get() {
            use std::io::Write;
            let _ = writeln!(logger.lock(), "{}", format!($($arg)*));
        }
    };
}
