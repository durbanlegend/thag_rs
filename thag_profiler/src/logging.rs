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
    std::env::var("THAG_PROFILER_DEBUG").map_or(DEBUG_LEVEL_NONE, |value| {
        match value.parse::<u8>() {
            Ok(level) if level > 0 => level,
            // If parsing fails or level is 0, but the var exists, default to level 1
            _ => DEBUG_LEVEL_LOG,
        }
    })
}

static_lazy! {
    DebugLogger: Option<Mutex<BufWriter<File>>> = {
        #[cfg(feature = "full_profiling")]
        {
            // For memory profiling, we must use the system allocator
            crate::with_allocator(crate::AllocatorType::SystemAlloc, || {
                create_debug_logger()
            })
        }

        #[cfg(not(feature = "full_profiling"))]
        {
            create_debug_logger()
        }
    }
}

// Helper function to create the debug logger
fn create_debug_logger() -> Option<Mutex<BufWriter<File>>> {
    // Get debug level
    let debug_level = get_debug_level();

    // Only proceed if debugging is enabled
    if debug_level > DEBUG_LEVEL_NONE {
        // Get the debug log path from ProfilePaths
        let log_path = PathBuf::from(&ProfilePaths::get().debug_log);

        // Print the log path if we're in verbose mode
        if debug_level >= DEBUG_LEVEL_VERBOSE {
            eprintln!("THAG Profiler debug log: {}", log_path.display());
        }

        // Try to open the file with a buffered writer
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(log_path) {
            // Add a header to the log file with information about the run
            // Increase buffer capacity to 64KB to reduce flush frequency
            let mut header_writer = BufWriter::with_capacity(65536, file);
            let _ = writeln!(header_writer, "--- Thag Profiler Debug Log ---");
            let _ = writeln!(
                header_writer,
                "Executable: {}",
                ProfilePaths::get().executable_stem
            );
            let _ = writeln!(
                header_writer,
                "Timestamp: {}",
                ProfilePaths::get().timestamp
            );
            let _ = writeln!(header_writer, "Debug level: {debug_level}");
            let _ = writeln!(
                header_writer,
                "Start time: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            let _ = writeln!(header_writer, "---------------------------");

            // Make sure to flush the header before returning
            if let Err(e) = header_writer.flush() {
                eprintln!("Failed to flush debug log header: {e}");
            }

            return Some(Mutex::new(header_writer));
        }
    }
    None
}

// Update the helper function to include debug level check
#[must_use]
pub fn get_debug_log_path() -> Option<String> {
    #[cfg(feature = "full_profiling")]
    {
        // Always use system allocator for getting log path
        crate::with_allocator(crate::AllocatorType::SystemAlloc, || {
            if get_debug_level() > DEBUG_LEVEL_NONE {
                Some(ProfilePaths::get().debug_log.clone())
            } else {
                None
            }
        })
    }

    #[cfg(not(feature = "full_profiling"))]
    {
        if get_debug_level() > DEBUG_LEVEL_NONE {
            Some(ProfilePaths::get().debug_log.clone())
        } else {
            None
        }
    }
}

// Define a function to flush the log buffer - can be called at strategic points
pub fn flush_debug_log() {
    #[cfg(feature = "full_profiling")]
    {
        // Always use system allocator for logging operations to prevent circular dependencies
        crate::with_allocator(crate::AllocatorType::SystemAlloc, || {
            if let Some(logger) = DebugLogger::get() {
                let flush_result = {
                    let mut locked_writer = logger.lock();
                    locked_writer.flush()
                };

                if let Err(e) = flush_result {
                    // Use eprintln for direct console output without going through our logger
                    eprintln!("Error flushing debug log: {e}");
                }
            }
        });
    }

    #[cfg(not(feature = "full_profiling"))]
    {
        if let Some(logger) = DebugLogger::get() {
            let flush_result = {
                let mut locked_writer = logger.lock();
                locked_writer.flush()
            };

            if let Err(e) = flush_result {
                // Use eprintln for direct console output without going through our logger
                eprintln!("Error flushing debug log: {}", e);
            }
        }
    }
}

// Improved debug_log macro that uses the lazy static
#[cfg(feature = "full_profiling")]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        // Always use system allocator for logging to prevent circular dependencies
        $crate::with_allocator($crate::AllocatorType::SystemAlloc, || {
            static mut LOG_COUNT: usize = 0;
            if let Some(logger) = $crate::DebugLogger::get() {
                use std::io::Write;
                let _write_result = {
                    let mut locked_writer = logger.lock();
                    writeln!(locked_writer, "{}", format!($($arg)*))
                };

                // Auto-flush periodically
                unsafe {
                    LOG_COUNT += 1;
                    if LOG_COUNT % 1000 == 0 {
                        $crate::flush_debug_log();
                    }
                }
            }
        })
    };
}

#[cfg(not(feature = "full_profiling"))]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::DebugLogger::get() {
            use std::io::Write;
            let _write_result = {
                let mut locked_writer = logger.lock();
                writeln!(locked_writer, "{}", format!($($arg)*))
            };

            // Auto-flush periodically
            static mut LOG_COUNT: usize = 0;
            unsafe {
                LOG_COUNT += 1;
                if LOG_COUNT % 1000 == 0 {
                    $crate::flush_debug_log();
                }
            }
        }
    };
}
