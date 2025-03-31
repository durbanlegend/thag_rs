use crate::profiling::ProfilePaths;
use crate::static_lazy;
use chrono::Local;
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

// Define constants for clarity
const DEBUG_LEVEL_NONE: u8 = 0;
const DEBUG_LEVEL_LOG: u8 = 1;
const DEBUG_LEVEL_VERBOSE: u8 = 2;

// Helper function to get the current debug level
fn get_debug_level() -> u8 {
    match std::env::var("THAG_PROFILER_DEBUG") {
        Ok(value) => {
            // Try to parse the value as a number
            match value.parse::<u8>() {
                Ok(level) if level > 0 => level,
                // If parsing fails or level is 0, but the var exists, default to level 1
                _ => DEBUG_LEVEL_LOG,
            }
        }
        // If env var isn't set, return 0 (no debugging)
        Err(_) => DEBUG_LEVEL_NONE,
    }
}

static_lazy! {
    DebugLogger: Option<Mutex<BufWriter<File>>> = {
        // Check if debug logging is enabled via environment variable
        if std::env::var("THAG_PROFILER_DEBUG").is_ok() {
            // Get the debug log path from ProfilePaths
            let log_path = PathBuf::from(&ProfilePaths::get().debug_log);

            // Try to open the file with a buffered writer
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                // Add a header to the log file with information about the run
                let mut header_writer = BufWriter::with_capacity(8192, file);
                let _ = writeln!(header_writer, "--- THAG Profiler Debug Log ---");
                let _ = writeln!(header_writer, "Executable: {}", ProfilePaths::get().executable_stem);
                let _ = writeln!(header_writer, "Timestamp: {}", ProfilePaths::get().timestamp);
                let _ = writeln!(header_writer, "Start time: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
                let _ = writeln!(header_writer, "---------------------------");
                let _ = header_writer.flush();

                return Some(Mutex::new(header_writer));
            }
        }
        None
    }
}

// #[must_use]
// pub fn get_platform_log_path() -> Option<PathBuf> {
//     // Platform-specific paths
//     #[cfg(target_os = "windows")]
//     {
//         if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
//             let mut path = PathBuf::from(local_app_data);
//             path.push("thag_profiler");
//             std::fs::create_dir_all(&path).ok()?;
//             path.push("debug.log");
//             return Some(path);
//         }
//     }

//     #[cfg(target_os = "macos")]
//     {
//         if let Some(home) = std::env::var_os("HOME") {
//             let mut path = PathBuf::from(home);
//             path.push("Library");
//             path.push("Logs");
//             path.push("thag_profiler");
//             std::fs::create_dir_all(&path).ok()?;
//             path.push("debug.log");
//             return Some(path);
//         }
//     }

//     #[cfg(target_os = "linux")]
//     {
//         if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
//             let mut path = PathBuf::from(xdg_data_home);
//             path.push("thag_profiler");
//             path.push("logs");
//             std::fs::create_dir_all(&path).ok()?;
//             path.push("debug.log");
//             return Some(path);
//         } else if let Some(home) = std::env::var_os("HOME") {
//             let mut path = PathBuf::from(home);
//             path.push(".local");
//             path.push("share");
//             path.push("thag_profiler");
//             path.push("logs");
//             std::fs::create_dir_all(&path).ok()?;
//             path.push("debug.log");
//             return Some(path);
//         }
//     }

//     // Fallback to temporary directory
//     let mut path = std::env::temp_dir();
//     path.push("thag_profiler");
//     std::fs::create_dir_all(&path).ok()?;
//     path.push("debug.log");
//     Some(path)
// }

#[must_use]
pub fn get_debug_log_path() -> Option<String> {
    if std::env::var("THAG_PROFILER_DEBUG").is_ok() {
        Some(ProfilePaths::get().debug_log.clone())
    } else {
        None
    }
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
