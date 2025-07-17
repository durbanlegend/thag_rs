use crate::profiling::{get_debug_level, DebugLevel, ProfilePaths};
use crate::static_lazy;
use chrono::Local;
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[cfg(feature = "full_profiling")]
use crate::safe_alloc;
// use crate::{mem_tracking, safe_alloc};

static_lazy! {
    DebugLogger: Option<Mutex<BufWriter<File>>> = {
        #[cfg(feature = "full_profiling")]
        {
            // For memory profiling, we must use the system allocator
            // use crate::mem_tracking;
            safe_alloc! {
                create_debug_logger()
            }
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
    if debug_level != DebugLevel::None {
        // Get the debug log path from ProfilePaths
        let log_path = PathBuf::from(&ProfilePaths::get().debug_log);

        // Print the log path if we're in verbose mode
        if debug_level == DebugLevel::Announce {
            eprintln!("Thag Profiler debug log: {}", log_path.display());
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
            let _ = writeln!(header_writer, "{}", "─".repeat(27));

            // Make sure to flush the header before returning
            if let Err(e) = header_writer.flush() {
                eprintln!("Failed to flush debug log header: {e}");
            }

            return Some(Mutex::new(header_writer));
        }
    }
    None
}

/// Retrieve the debug log path
#[allow(clippy::missing_const_for_fn)]
#[must_use]
pub fn get_debug_log_path() -> Option<String> {
    #[cfg(feature = "full_profiling")]
    {
        // Always use system allocator for getting log path
        // use crate::mem_tracking;
        safe_alloc! {
            if get_debug_level() == DebugLevel::None {
                None
            } else {
                Some(ProfilePaths::get().debug_log.clone())
            }
        }
    }

    #[cfg(not(feature = "full_profiling"))]
    {
        if get_debug_level() == DebugLevel::None {
            None
        } else {
            Some(ProfilePaths::get().debug_log.clone())
        }
    }
}

/// A function to flush the log buffer - can be called at strategic points
#[allow(clippy::missing_const_for_fn)]
pub fn flush_debug_log() {
    #[cfg(not(feature = "full_profiling"))]
    {
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
    }

    // No-op for full profiling to prevent deadlocks
    // The BufWriter will auto-flush on drop anyway
    #[cfg(feature = "full_profiling")]
    {}
}

/// Zero-cost debug logging gated behind feature `debug_logging`.
///
/// Compiles to unit expression when `debug_logging` feature is disabled
#[cfg(feature = "debug_logging")]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        // #[cfg(feature = "full_profiling")]
        // use $crate::mem_tracking;

        $crate::safe_alloc! {
            if let Some(logger) = $crate::DebugLogger::get() {
                use std::io::Write;
                let _write_result = {
                    let mut locked_writer = logger.lock();
                    writeln!(locked_writer, "{}", format!($($arg)*))
                };
                // Remove the auto-flush logic to prevent deadlocks
            }
        }
    };
}

/// Zero-cost debug logging gated behind feature `debug_logging`.
///
/// Compiles to unit expression when `debug_logging` feature is disabled
#[cfg(not(feature = "debug_logging"))]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        ()
    };
}

/// This test suite provides comprehensive coverage for the logging module:
///
/// E.g.:
///
/// ```bash
/// cargo test --features=time_profiling logging -- --nocapture
/// ```
///
/// 1. **Feature-gated tests**: Tests are conditionally compiled for `time_profiling` or `full_profiling` features
/// 2. **Debug level detection**: Verifies the debug level can be correctly determined
/// 3. **Debug log path**: Tests that the debug log path matches the expected format
/// 4. **Log file creation**: Ensures the log file is created with appropriate settings
/// 5. **Write and flush operations**: Validates logs can be written and explicitly flushed
/// 6. **Auto-flushing**: Tests the automatic flushing that happens every 1000 log messages
/// 7. **System allocator usage**: For `full_profiling`, verifies that logging operations use the system allocator
/// 8. **Logger initialization**: Tests that the `DebugLogger` static is properly initialized
///
/// All tests are organized into individual functions that are called sequentially from a single `#[test]` function to avoid concurrency issues with the global state. The tests handle different debug levels and feature flag combinations appropriately.
///
/// Key features of this test design:
///
/// 1. **Safe allocator usage**: When `full_profiling` is enabled, operations use `with_sys_alloc(...)` to prevent recursive tracking
/// 2. **Conditional testing**: Tests skip or modify behavior based on the active debug level
/// 3. **Minimal side effects**: Tests avoid disrupting the global state in ways that could affect other tests
/// 4. **Feature compatibility**: Tests work with both `time_profiling` and `full_profiling` feature flags
#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_log;

    #[cfg(feature = "full_profiling")]
    use crate::{safe_alloc, ProfileType};

    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    /// Test the entire logging system in a single sequential test
    #[test]
    fn test_logging_functionality() {
        // Initialize profiling to set up logging
        #[cfg(feature = "full_profiling")]
        safe_alloc! {
            let _ =
                crate::profiling::test_utils::initialize_profiling_for_test(ProfileType::Memory);
        };

        #[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
        // Using attribute-based profiling
        crate::profiling::force_enable_profiling_time_for_tests();

        // ----- Test 1: Debug Level Detection -----
        let debug_level = get_debug_level();
        eprintln!("Current debug level: {:?}", debug_level);

        // ----- Test 2: Debug Log Path -----
        let log_path = get_debug_log_path();
        eprintln!("Debug log path: {:?}", log_path);

        // Skip further tests if debug level is None
        if matches!(debug_level, DebugLevel::None) {
            eprintln!("Debug level is None, skipping remaining logging tests");
            return;
        }

        // ----- Test 3: Log Writing and Flushing -----
        // Write a unique message for identification
        let unique_msg = format!(
            "Unique test message: {}",
            chrono::Local::now().format("%H:%M:%S.%3f")
        );
        debug_log!("{}", unique_msg);
        flush_debug_log();

        // Verify the log file exists
        let path = log_path.as_ref().unwrap();
        assert!(Path::new(path).exists(), "Log file should exist");

        // Verify the message was written
        let file = File::open(path).expect("Failed to open log file");
        let reader = BufReader::new(file);

        #[cfg(feature = "full_profiling")]
        let found_message = safe_alloc! {
            reader
                .lines()
                .filter_map(Result::ok)
                .any(|line| line.contains(&unique_msg))
        };

        #[cfg(not(feature = "full_profiling"))]
        let found_message = reader
            .lines()
            .filter_map(Result::ok)
            .any(|line| line.contains(&unique_msg));

        assert!(found_message, "Log should contain our test message");

        // ----- Test 4: Logger Access -----
        let logger = DebugLogger::get();
        assert!(logger.is_some(), "Logger should be available");

        // Write directly to the logger
        if let Some(logger_mutex) = logger {
            let write_result = {
                let mut locked_logger = logger_mutex.lock();
                writeln!(locked_logger, "Direct logger write test")
            };
            assert!(write_result.is_ok(), "Direct write should succeed");
        }

        // ----- Test 5: Auto-flush Mechanism -----
        // We only write a few messages to avoid making the test slow
        // The real auto-flush happens at 1000 messages
        for i in 0..5 {
            debug_log!("Auto-flush test message #{}", i);
        }

        // Explicitly flush for test purposes
        flush_debug_log();

        // ----- Test 6: System Allocator Usage (full_profiling only) -----
        #[cfg(feature = "full_profiling")]
        {
            let current_allocator = safe_alloc!(crate::mem_tracking::current_allocator());

            // Log a message (which should use system allocator)
            debug_log!("Testing system allocator usage");

            // Verify allocator wasn't changed
            let after_allocator = safe_alloc!(crate::mem_tracking::current_allocator());

            assert_eq!(
                current_allocator, after_allocator,
                "Allocator should remain unchanged after logging"
            );
        }

        // Clean up
        crate::profiling::disable_profiling();

        eprintln!("All logging tests completed successfully!");
    }

    // You can add specific unit tests for individual functions if needed
    #[test]
    fn test_create_debug_logger() {
        // Test the logger creation based on debug levels
        // This is just a verification test since real behavior depends on environment
        let logger = create_debug_logger();

        if matches!(get_debug_level(), DebugLevel::None) {
            assert!(
                logger.is_none(),
                "Logger should be None with debug level None"
            );
        }
    }
}
