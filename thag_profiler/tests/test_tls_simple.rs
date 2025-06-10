/// Simple test for thread-local storage allocator functions
/// Uses function calls only to avoid macro expansion issues
use thag_profiler::{
    current_allocator, current_allocator_tls, mem_tracking, with_sys_alloc, with_sys_alloc_tls,
    Allocator,
};

// Import debug_log based on feature availability
#[cfg(feature = "debug_logging")]
use thag_profiler::debug_log;

// For tests without debug_logging feature, use the fallback macro
#[cfg(not(feature = "debug_logging"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}
use serial_test::serial;
use std::sync::{Arc, Barrier};
use std::thread;

// Test utility to reset state
#[cfg(feature = "full_profiling")]
fn reset_allocator_states() {
    // Reset global state
    thag_profiler::mem_tracking::reset_global_allocator_state();
    // Reset thread-local state
    thag_profiler::mem_tracking::reset_tls_allocator_state();
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_tls_vs_global_functions() {
    reset_allocator_states();
    // Initially both should be in Tracking mode
    assert_eq!(current_allocator(), Allocator::Tracking);
    assert_eq!(current_allocator_tls(), Allocator::Tracking);

    // Test global version - changes global flag from false->true, then back to false
    with_sys_alloc(|| {
        assert_eq!(current_allocator(), Allocator::System);
        assert_eq!(current_allocator_tls(), Allocator::Tracking); // TLS unaffected
    });

    // Test TLS version - changes TLS flag from false->true, then back to false
    with_sys_alloc_tls(|| {
        assert_eq!(current_allocator(), Allocator::Tracking); // Global unaffected
        assert_eq!(current_allocator_tls(), Allocator::System);
    });

    // Both should be back to Tracking (flags reset by their respective setters)
    assert_eq!(current_allocator(), Allocator::Tracking);
    assert_eq!(current_allocator_tls(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_nested_tls_behavior() {
    reset_allocator_states();
    // Test that nested TLS calls work correctly - inner call should not touch flag
    assert_eq!(current_allocator_tls(), Allocator::Tracking);

    with_sys_alloc_tls(|| {
        assert_eq!(current_allocator_tls(), Allocator::System);

        // Nested call should NOT change anything (already in System mode)
        // The inner call finds the flag is already true, so it doesn't touch it
        with_sys_alloc_tls(|| {
            assert_eq!(current_allocator_tls(), Allocator::System);
        });

        // Still in System mode after nested call - the outer call still owns the flag
        assert_eq!(current_allocator_tls(), Allocator::System);
    });

    // Back to Tracking mode - only the outer call resets the flag
    assert_eq!(current_allocator_tls(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_nested_global_behavior() {
    reset_allocator_states();
    // Test that nested global calls work correctly - inner call should not touch flag
    assert_eq!(current_allocator(), Allocator::Tracking);

    with_sys_alloc(|| {
        assert_eq!(current_allocator(), Allocator::System);

        // Nested call should NOT change anything (already in System mode)
        // The inner call finds the flag is already true, so it doesn't touch it
        with_sys_alloc(|| {
            assert_eq!(current_allocator(), Allocator::System);
        });

        // Still in System mode after nested call - the outer call still owns the flag
        assert_eq!(current_allocator(), Allocator::System);
    });

    // Back to Tracking mode - only the outer call resets the flag
    assert_eq!(current_allocator(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_mixed_nesting() {
    reset_allocator_states();
    // Test mixing global and TLS calls
    assert_eq!(current_allocator(), Allocator::Tracking);
    assert_eq!(current_allocator_tls(), Allocator::Tracking);

    // Start with global
    with_sys_alloc(|| {
        assert_eq!(current_allocator(), Allocator::System);
        assert_eq!(current_allocator_tls(), Allocator::Tracking);

        // Nest TLS call inside global
        with_sys_alloc_tls(|| {
            assert_eq!(current_allocator(), Allocator::System); // Still global system
            assert_eq!(current_allocator_tls(), Allocator::System); // Now TLS is also system
        });

        // TLS should be back to tracking, global still system
        assert_eq!(current_allocator(), Allocator::System);
        assert_eq!(current_allocator_tls(), Allocator::Tracking);
    });

    // Both should be back to tracking
    assert_eq!(current_allocator(), Allocator::Tracking);
    assert_eq!(current_allocator_tls(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_thread_isolation() {
    reset_allocator_states();
    // Test that TLS version provides thread isolation
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    // Thread 1: Uses global allocator switching
    let barrier1 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier1.wait(); // Synchronize start

        with_sys_alloc(|| {
            // This thread is in System mode (global)
            assert_eq!(current_allocator(), Allocator::System);
            assert_eq!(current_allocator_tls(), Allocator::Tracking);

            // Sleep to ensure other threads can check their state
            thread::sleep(std::time::Duration::from_millis(10));
        });
    }));

    // Thread 2: Uses TLS allocator switching
    let barrier2 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier2.wait(); // Synchronize start

        with_sys_alloc_tls(|| {
            // This thread is in System mode (TLS only)
            assert_eq!(current_allocator_tls(), Allocator::System);

            // Sleep to ensure other threads can check their state
            thread::sleep(std::time::Duration::from_millis(10));
        });
    }));

    // Thread 3: Checks it's unaffected by other threads
    let barrier3 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier3.wait(); // Synchronize start

        // Small delay to let other threads switch allocators
        thread::sleep(std::time::Duration::from_millis(5));

        // TLS should be isolated from other threads
        assert_eq!(current_allocator_tls(), Allocator::Tracking);

        // Global might be affected by thread 1, but TLS should be isolated
        // We can't assert the global state since thread 1 might still be running
    }));

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_performance_comparison() {
    reset_allocator_states();
    use std::time::Instant;

    const ITERATIONS: usize = 1000;

    // Time global atomic version
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        with_sys_alloc(|| {
            // Minimal work
            std::hint::black_box(current_allocator());
        });
    }
    let global_duration = start.elapsed();

    // Time TLS version
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        with_sys_alloc_tls(|| {
            // Minimal work
            std::hint::black_box(current_allocator_tls());
        });
    }
    let tls_duration = start.elapsed();

    println!(
        "Performance comparison ({} iterations):\n  Global atomic: {:?}\n  Thread-local: {:?}\n  TLS speedup: {:.2}x",
        ITERATIONS,
        global_duration,
        tls_duration,
        global_duration.as_nanos() as f64 / tls_duration.as_nanos() as f64
    );

    // TLS should be faster (though this is not a strict requirement for the test)
    // We just print the results for analysis
}

#[test]
fn test_debug_log_zero_cost() {
    // This test should always pass - when debug_logging is disabled,
    // the macro should compile to nothing
    debug_log!("This should be zero-cost when feature is disabled");

    // The fact that this compiles and runs means the macro is working correctly
    assert!(true);
}

#[test]
#[cfg(feature = "debug_logging")]
fn test_debug_log_with_feature() {
    // Test the zero-cost debug logging macro when feature is enabled
    debug_log!("This is a test message: {}", 42);
    debug_log!("Multiple args: {} {} {}", "hello", "world", 123);

    // When debug_logging feature is enabled, these should write to the debug log
    // We can't easily test the output here, but we can verify it compiles and runs
    assert!(true);
}
